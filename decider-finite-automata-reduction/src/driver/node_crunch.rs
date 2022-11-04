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

struct ProofStage {
    prover: ProverBox,
    out: OutputFile,
    dvf: DeciderVerificationFile,
    ids: VecDeque<MachineID>,
    bar: ProgressBar,
}

struct Server {
    index: Index,
    progress: DeciderProgress,
    prover_names: VecDeque<String>,
    stage: Option<ProofStage>,
    in_flight: usize,
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
        loop {
            if let Some(stage) = self.stage.as_mut() {
                // TODO: Fixed batch size of 100 is stupid!
                let len = min(stage.ids.len(), 10000);
                if len > 0 {
                    self.in_flight += 1;
                    stage.bar.inc(len as u64);
                    return Ok(NCJobStatus::Unfinished(NewData {
                        prover_name: stage.prover.name(),
                        ids: stage.ids.drain(0..len).collect(),
                    }));
                } else if self.in_flight > 0 {
                    return Ok(NCJobStatus::Waiting);
                }
            }
            if let Some(prover) = self.prover_names.pop_front().and_then(prover_by_name) {
                let out = OutputFile::append(format!("output/{}.ind", prover.name()))?;
                let dvf = DeciderVerificationFile::append(format!("output/{}.dvf", prover.name()))?;
                self.index.read_decided("output", false)?;
                let ids: VecDeque<MachineID> = self.index.iter().collect();
                let bar = self.progress.prover_progress(ids.len(), prover.name());
                self.progress.set_solved(self.index.len_solved());
                self.stage = Some(ProofStage {
                    prover,
                    out,
                    dvf,
                    ids,
                    bar,
                });
            } else {
                self.stage = None;
                return Ok(NCJobStatus::Finished);
            }
        }
    }

    fn process_data_from_node(
        &mut self,
        _id: NodeID,
        node_data: &Self::ProcessedDataT,
    ) -> Result<(), NCError> {
        if let Some(stage) = self.stage.as_mut() {
            self.in_flight -= 1;
            for (i, result) in node_data.results.iter() {
                match &result {
                    Ok((direction, dfa)) => {
                        stage.out.insert(*i)?;
                        stage.dvf.insert(*i, *direction, dfa)?;
                        self.progress.solve(1);
                    }
                    Err(e) => {
                        let name = stage.prover.name();
                        let msg = format!("Rejected {} proof of {}: {:?}", name, i, e);
                        self.progress.println(msg)?;
                    }
                }
            }
            Ok(())
        } else {
            Err(NCError::NodeMsgMismatch)
        }
    }

    fn heartbeat_timeout(&mut self, nodes: Vec<NodeID>) {
        if !nodes.is_empty() {
            let oh_no = format!("Heartbeat timeout from nodes: {:?}", nodes);
            let _ = self.progress.println(oh_no); // TODO: Retry logic
            self.in_flight -= nodes.len();
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
) {
    NCServerStarter::new(config_from_args(args))
        .start(Server {
            index,
            progress,
            prover_names,
            stage: None,
            in_flight: 0,
        })
        .expect("Quit due to server error!");
}
