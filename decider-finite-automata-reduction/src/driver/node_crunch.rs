use super::DeciderProgress;
use crate::core::{BadProof, Side, DFA};
use crate::io::{Database, DeciderVerificationFile, Index, MachineID, OutputFile};
use crate::provers::{prover_by_name, ProverBox};
use crate::DeciderArgs;
use indicatif::ProgressBar;
use node_crunch::{
    NCConfiguration, NCError, NCJobStatus, NCNode, NCNodeStarter, NCServer, NCServerStarter, NodeID,
};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

type DeciderResult = Result<(Side, DFA), BadProof>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewData {
    prover_name: String,
    ids: Vec<MachineID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    results: Vec<(MachineID, DeciderResult)>,
}

struct WorkStats {
    target_size: usize,
    last_send: Instant,
    todo: Option<NewData>,
}

impl Default for WorkStats {
    fn default() -> Self {
        Self {
            target_size: 1,
            last_send: Instant::now(),
            todo: None,
        }
    }
}

fn update(s: &mut WorkStats) {
    s.last_send = Instant::now();
}

struct Server {
    index: Index,
    progress: DeciderProgress,
    prover_names: VecDeque<String>,
    out: OutputFile,
    dvf: DeciderVerificationFile,
    current_prover: String,
    ids: VecDeque<MachineID>,
    retries: Vec<NewData>,
    bar: ProgressBar,
    stats: HashMap<NodeID, WorkStats>,
    tms_in_flight: usize,
}

impl NCServer for Server {
    type InitialDataT = ();
    type NewDataT = NewData;
    type ProcessedDataT = ProcessedData;
    type CustomMessageT = ();

    fn prepare_data_for_node(
        &mut self,
        node_id: NodeID,
    ) -> Result<NCJobStatus<Self::NewDataT>, NCError> {
        let entry = self.stats.entry(node_id).and_modify(update).or_default();
        if let Some(new_data) = self.retries.pop() {
            entry.todo = Some(new_data.clone());
            return Ok(NCJobStatus::Unfinished(new_data));
        }
        while self.ids.is_empty() {
            if let Some(prover) = self.prover_names.pop_front() {
                self.current_prover = prover;
                self.index.read_decided()?;
                self.ids = self.index.iter().collect();
                self.bar = self
                    .progress
                    .prover_progress(self.ids.len(), self.current_prover.clone());
                self.progress.set_solved(self.index.len_solved());
            } else if self.tms_in_flight == 0 {
                return Ok(NCJobStatus::Finished);
            } else {
                return Ok(NCJobStatus::Waiting);
            }
        }
        let len = min(self.ids.len(), entry.target_size);
        self.bar.inc(len as u64);
        self.tms_in_flight += len;
        let new_data = NewData {
            prover_name: self.current_prover.clone(),
            ids: self.ids.drain(0..len).collect(),
        };
        entry.todo = Some(new_data.clone());
        Ok(NCJobStatus::Unfinished(new_data))
    }

    fn process_data_from_node(
        &mut self,
        node_id: NodeID,
        node_data: &Self::ProcessedDataT,
    ) -> Result<(), NCError> {
        self.stats.entry(node_id).and_modify(|s| {
            let sent = std::mem::take(&mut s.todo);
            // Adapt the batch size with target of 1s. (Exact value barely matters.)
            let size_out = match sent {
                Some(data) => data.ids.len(),
                None => s.target_size,
            };
            s.target_size = match s.last_send.elapsed().as_millis() {
                0..=250 => max(s.target_size, size_out * 4),
                251..=500 => max(s.target_size, size_out * 2),
                501..=2000 => s.target_size,
                2001..=4000 => min(s.target_size, size_out / 2),
                _ => min(s.target_size, size_out / 4),
            };
            s.target_size = max(1, min(s.target_size, 8192));
            self.tms_in_flight -= size_out;
        });
        for (i, result) in node_data.results.iter() {
            match &result {
                Ok((direction, dfa)) => {
                    self.out.insert(*i)?;
                    self.dvf.insert(*i, *direction, dfa)?;
                    self.progress.solve(1);
                }
                Err(e) => {
                    let msg = format!("Rejected proof of {}: {:?}", i, e);
                    self.progress.println(msg)?;
                }
            }
        }
        if self.ids.is_empty() && self.prover_names.is_empty() {
            let msg = format!("Awaiting results for {} TMs + 60s shutdown.", self.tms_in_flight);
            self.progress.println(msg)?;
        }
        Ok(())
    }

    fn heartbeat_timeout(&mut self, nodes: Vec<NodeID>) {
        for id in nodes {
            self.stats.entry(id).and_modify(|s| {
                if let Some(new_data) = std::mem::take(&mut s.todo) {
                    let msg = format!("{:?} died. Retrying {} TMs.", id, new_data.ids.len());
                    let _ = self.progress.println(msg);
                    self.retries.push(new_data);
                }
            });
        }
    }

    fn finish_job(&mut self) {}
}

struct Node {
    db: Database,
    current_prover: Option<ProverBox>,
    current_prover_name: String,
}

impl NCNode for Node {
    type InitialDataT = ();
    type NewDataT = NewData;
    type ProcessedDataT = ProcessedData;
    type CustomMessageT = ();

    fn process_data_from_server(
        &mut self,
        data: &Self::NewDataT,
    ) -> Result<Self::ProcessedDataT, NCError> {
        if self.current_prover_name != data.prover_name {
            self.current_prover_name = data.prover_name.clone();
            self.current_prover = prover_by_name(&data.prover_name);
        }
        let prover = match &mut self.current_prover {
            Some(p) => p,
            None => return Err(NCError::ServerMsgMismatch),
        };
        let results = self
            .db
            .read(data.ids.iter().copied())
            .filter_map(|(i, tm)| {
                prover
                    .prove(&tm)
                    .map(|proof| {
                        proof
                            .validate(&tm)
                            .map(|()| (proof.automaton.direction, proof.automaton.dfa))
                    })
                    .map(|r| (i, r))
            })
            .collect();
        Ok(ProcessedData { results })
    }
}

fn config_from_args(args: DeciderArgs) -> NCConfiguration {
    NCConfiguration {
        address: args.ip,
        port: args.port,
        compress: true,
        ..Default::default()
    }
}

pub fn run_node(args: DeciderArgs, db: Database) {
    let current_prover = None;
    let current_prover_name = String::new();
    NCNodeStarter::new(config_from_args(args))
        .start(Node {
            db,
            current_prover,
            current_prover_name,
        })
        .expect("Quit due to node error!");
}

pub fn process_remote(
    args: DeciderArgs,
    index: Index,
    progress: DeciderProgress,
    prover_names: VecDeque<String>,
    out: OutputFile,
    dvf: DeciderVerificationFile,
) {
    NCServerStarter::new(config_from_args(args))
        .start(Server {
            index,
            progress,
            prover_names,
            out,
            dvf,
            current_prover: String::new(),
            ids: VecDeque::new(),
            retries: Vec::new(),
            bar: ProgressBar::hidden(),
            stats: HashMap::new(),
            tms_in_flight: 0,
        })
        .expect("Quit due to server error!");
}
