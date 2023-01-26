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
    stage: usize,
    batch_id: usize,
    ids: Vec<MachineID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    batch_id: usize,
    results: Vec<(MachineID, DeciderResult)>,
}

struct WorkStats {
    target_size: usize,
    last_send: Instant,
    todo: Vec<usize>,
}

impl Default for WorkStats {
    fn default() -> Self {
        Self {
            target_size: 1,
            last_send: Instant::now(),
            todo: Vec::new(),
        }
    }
}

fn update(s: &mut WorkStats) {
    s.last_send = Instant::now();
}

struct Server {
    index: Index,
    progress: DeciderProgress,
    prover_names: Vec<String>,
    out: OutputFile,
    dvf: DeciderVerificationFile,
    stage: usize,
    batch_id: usize,
    ids: VecDeque<MachineID>,
    retry_batches: Vec<usize>,
    bars: Vec<ProgressBar>,
    stats: HashMap<NodeID, WorkStats>,
    batches_out: HashMap<usize, NewData>,
    tms_out_this_stage: usize,
}

impl NCServer for Server {
    type InitialDataT = Vec<String>;
    type NewDataT = NewData;
    type ProcessedDataT = ProcessedData;
    type CustomMessageT = ();

    fn initial_data(&mut self) -> Result<Option<Self::InitialDataT>, NCError> {
        Ok(Some(self.prover_names.clone()))
    }

    fn prepare_data_for_node(
        &mut self,
        node_id: NodeID,
    ) -> Result<NCJobStatus<Self::NewDataT>, NCError> {
        let entry = self.stats.entry(node_id).and_modify(update).or_default();
        if let Some(batch_id) = self.retry_batches.pop() {
            entry.todo.push(batch_id);
            return Ok(NCJobStatus::Unfinished(self.batches_out[&batch_id].clone()));
        }
        while self.ids.is_empty() {
            self.stage += (self.batch_id != 0) as usize;
            self.index.read_decided()?;
            self.progress.set_solved(self.index.len_solved());
            if self.stage < self.prover_names.len() {
                self.ids = self.index.iter().collect();
                self.tms_out_this_stage = 0;
                self.bars.push(
                    self.progress
                        .prover_progress(self.ids.len(), self.prover_names[self.stage].clone()),
                );
            } else if self.batches_out.is_empty() {
                self.progress.println("Done! Worker shutdown takes ~60s.")?;
                self.progress.finish();
                return Ok(NCJobStatus::Finished);
            } else {
                return Ok(NCJobStatus::Waiting);
            }
        }
        let len = min(self.ids.len(), entry.target_size);
        self.tms_out_this_stage += len;
        self.batch_id += 1;
        let new_data = NewData {
            stage: self.stage,
            batch_id: self.batch_id,
            ids: self.ids.drain(0..len).collect(),
        };
        entry.todo.push(self.batch_id);
        self.batches_out.insert(self.batch_id, new_data.clone());
        Ok(NCJobStatus::Unfinished(new_data))
    }

    fn process_data_from_node(
        &mut self,
        node_id: NodeID,
        node_data: &Self::ProcessedDataT,
    ) -> Result<(), NCError> {
        let sent = self
            .batches_out
            .remove(&node_data.batch_id)
            .ok_or(NCError::ServerMsgMismatch)?;
        self.stats.entry(node_id).and_modify(|s| {
            s.todo.retain(|&id| id != node_data.batch_id);
            let size_out = sent.ids.len();
            // Adapt the batch size with target of 1s. (Exact value barely matters.)
            s.target_size = match s.last_send.elapsed().as_millis() {
                0..=250 => max(s.target_size, size_out * 4),
                251..=500 => max(s.target_size, size_out * 2),
                501..=2000 => s.target_size,
                2001..=4000 => min(s.target_size, size_out / 2),
                _ => min(s.target_size, size_out / 4),
            };
            s.target_size = max(1, min(s.target_size, 8192));
            self.bars[sent.stage].inc(size_out as u64);
            if self.bars[sent.stage].length() == Some(self.bars[sent.stage].position()) {
                self.bars[sent.stage].finish();
            }
            if self.stage == sent.stage {
                self.tms_out_this_stage -= size_out;
            } else if self.stage < self.prover_names.len() {
                let done: Vec<MachineID> = node_data
                    .results
                    .iter()
                    .filter_map(|(id, result)| result.is_ok().then_some(*id))
                    .collect();
                self.ids.retain(|id| !done.contains(id));
                self.bars[self.stage].set_length(
                    (self.tms_out_this_stage + self.ids.len()) as u64
                        + self.bars[self.stage].position(),
                );
            }
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
        Ok(())
    }

    fn heartbeat_timeout(&mut self, nodes: Vec<NodeID>) {
        for id in nodes {
            self.stats.entry(id).and_modify(|s| {
                if !s.todo.is_empty() {
                    let msg = format!("{:?} died. Retrying its work.", id);
                    let _ = self.progress.println(msg);
                    self.retry_batches.extend(s.todo.drain(..));
                }
            });
        }
    }

    fn finish_job(&mut self) {}
}

struct Node {
    db: Database,
    prover_names: Vec<String>,
    current_prover: Option<ProverBox>,
    current_stage: usize,
}

impl NCNode for Node {
    type InitialDataT = Vec<String>;
    type NewDataT = NewData;
    type ProcessedDataT = ProcessedData;
    type CustomMessageT = ();

    fn set_initial_data(&mut self, _: NodeID, data: Option<Vec<String>>) -> Result<(), NCError> {
        self.prover_names = data.ok_or(NCError::ServerMsgMismatch)?;
        Ok(())
    }

    fn process_data_from_server(
        &mut self,
        data: &Self::NewDataT,
    ) -> Result<Self::ProcessedDataT, NCError> {
        if self.current_prover.is_none() || self.current_stage != data.stage {
            self.current_stage = data.stage;
            self.current_prover = prover_by_name(&self.prover_names[data.stage]);
        }
        let prover = match &mut self.current_prover {
            Some(p) => p,
            None => return Err(NCError::ServerMsgMismatch),
        };
        let batch_id = data.batch_id;
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
        Ok(ProcessedData { batch_id, results })
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
    let prover_names = Vec::new();
    let current_prover = None;
    let current_stage = 0;
    NCNodeStarter::new(config_from_args(args))
        .start(Node {
            db,
            prover_names,
            current_prover,
            current_stage,
        })
        .expect("Quit due to node error!");
}

pub fn process_remote(
    args: DeciderArgs,
    index: Index,
    progress: DeciderProgress,
    prover_names: Vec<String>,
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
            stage: 0,
            batch_id: 0,
            ids: VecDeque::new(),
            retry_batches: Vec::new(),
            bars: Vec::new(),
            stats: HashMap::new(),
            batches_out: HashMap::new(),
            tms_out_this_stage: 0,
        })
        .expect("Quit due to server error!");
}
