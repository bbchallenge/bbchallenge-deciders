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
use std::cmp::min;
use std::collections::VecDeque;

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

struct Server {
    index: Index,
    progress: DeciderProgress,
    prover_names: VecDeque<String>,
    out: OutputFile,
    dvf: DeciderVerificationFile,
    current_prover: String,
    ids: VecDeque<MachineID>,
    bar: ProgressBar,
}

impl NCServer for Server {
    type InitialDataT = ();
    type NewDataT = NewData;
    type ProcessedDataT = ProcessedData;
    type CustomMessageT = ();

    fn prepare_data_for_node(
        &mut self,
        _id: NodeID,
    ) -> Result<NCJobStatus<Self::NewDataT>, NCError> {
        while self.ids.is_empty() {
            if let Some(prover) = self.prover_names.pop_front() {
                self.current_prover = prover;
                self.index.read_decided()?;
                self.ids = self.index.iter().collect();
                self.bar = self
                    .progress
                    .prover_progress(self.ids.len(), self.current_prover.clone());
                self.progress.set_solved(self.index.len_solved());
            } else {
                return Ok(NCJobStatus::Finished);
            }
        }
        // TODO: Using a fixed batch size is stupid!
        let len = min(self.ids.len(), 10000);
        self.bar.inc(len as u64);
        Ok(NCJobStatus::Unfinished(NewData {
            prover_name: self.current_prover.clone(),
            ids: self.ids.drain(0..len).collect(),
        }))
    }

    fn process_data_from_node(
        &mut self,
        _id: NodeID,
        node_data: &Self::ProcessedDataT,
    ) -> Result<(), NCError> {
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
        if !nodes.is_empty() {
            let oh_no = format!("Heartbeat timeout from nodes: {:?}", nodes);
            let _ = self.progress.println(oh_no); // TODO: Retry logic
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
            bar: ProgressBar::hidden(),
        })
        .expect("Quit due to server error!");
}
