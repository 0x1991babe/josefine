use std::time::Duration;
use std::time::Instant;

use slog::Logger;

use crate::error::RaftError;
use crate::follower::Follower;
use crate::progress::ProgressHandle;
use crate::progress::ReplicationProgress;
use crate::raft::{Apply, NodeId, RaftHandle, RaftRole};
use crate::raft::Command;
use crate::raft::Raft;
use crate::raft::Role;
use crate::rpc::RpcMessage;

///
#[derive(Debug)]
pub struct Leader {
    pub logger: Logger,
    pub progress: ReplicationProgress,
    /// The time of the last heartbeat.
    pub heartbeat_time: Instant,
    /// The timeout since the last heartbeat.
    pub heartbeat_timeout: Duration,
}

impl Raft<Leader> {
    pub(crate) fn heartbeat(&self) -> Result<(), RaftError> {
        for (_, node) in &self.nodes {
            let _ = RpcMessage::Heartbeat(self.state.current_term, self.id);
        };

        Ok(())
    }

    fn _append_entry(&mut self, _node_id: NodeId, handle: ProgressHandle) {
        match handle {
            ProgressHandle::Probe(_) => {},
            ProgressHandle::Replicate(_) => {},
            ProgressHandle::Snapshot(_) => {},
        };
    }

    fn needs_heartbeat(&self) -> bool {
        self.role.heartbeat_time.elapsed() > self.role.heartbeat_timeout
    }

    fn reset_heartbeat_timer(&mut self) {
        self.role.heartbeat_time = Instant::now();
    }
}

impl Role for Leader {
    fn term(&mut self, _term: u64) {
    }

    fn role(&self) -> RaftRole {
        RaftRole::Leader
    }

    fn log(&self) -> &Logger {
        &self.logger
    }
}

impl Apply for Raft<Leader> {
    fn apply(mut self, cmd: Command) -> Result<RaftHandle, RaftError> {
        self.log_command(&cmd);
        match cmd {
            Command::Tick => {
                if self.needs_heartbeat() {
                    if let Err(_err) = self.heartbeat() {
                        panic!("Could not heartbeat")
                    }
                    self.reset_heartbeat_timer();
                }

                for (node_id, node) in &self.nodes {
                    if let Some(mut progress) = self.role.progress.get_mut(*node_id) {
                        match &mut progress {
                            ProgressHandle::Replicate(progress) => {
                                let entries = self.log.get_range(&progress.next, &(progress.next + crate::progress::MAX_INFLIGHT));
                                let len = entries.len();
                                let _ = RpcMessage::Append {
                                    term: self.state.current_term,
                                    leader_id: self.id,
                                    prev_log_index: progress.index,
                                    prev_log_term: 0, // TODO(jcm) need to track in progress?
                                    entries: entries.to_vec(),
                                    leader_commit: 0
                                };

                                progress.next = len as u64;
                            }
                            _ => {}
                        }
                    }
                }

                Ok(RaftHandle::Leader(self))
            }
            Command::AppendResponse { node_id, index, .. } => {
                if let Some(mut progress) = self.role.progress.get_mut(node_id) {
                    match &mut progress {
                        ProgressHandle::Replicate(progress) => {
                            progress.increment(index);
                        }
                        _ => panic!()
                    }
                }

                self.state.commit_index = self.role.progress.committed_index();
                Ok(RaftHandle::Leader(self))
            }
            Command::AppendEntries { term, .. } => {
                if term > self.state.current_term {
                    // TODO(jcm): move term logic into dedicated handler
                    self.term(term);
                    return Ok(RaftHandle::Follower(Raft::from(self)));
                }

                Ok(RaftHandle::Leader(self))
            }
            _ => Ok(RaftHandle::Leader(self))
        }
    }
}

impl From<Raft<Leader>> for Raft<Follower> {
    fn from(val: Raft<Leader>) -> Raft<Follower> {
        Raft {
            id: val.id,
            state: val.state,
            nodes: val.nodes,
            role: Follower { leader_id: None, logger: val.logger.new(o!("role" => "follower"))  },
            logger: val.logger,
            config: val.config,
            log: val.log,
        }
    }
}
