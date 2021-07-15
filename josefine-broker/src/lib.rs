#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
#[macro_use]
extern crate serde_derive;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use josefine_core::error::Result;
use josefine_raft::client::RaftClient;
use josefine_raft::rpc::Proposal;
use josefine_raft::rpc::Response;
use server::Broker;
use server::Server;
use sled::Db;

mod entry;
mod index;
mod log;
mod partition;
mod segment;
mod server;
pub mod fsm;
mod tcp;
mod topic;

pub struct JosefineBroker {
    db: &'static Db
}

impl JosefineBroker {
    pub fn new(db: &'static Db) -> Self {
        JosefineBroker {
            db
        }
    }

    pub async fn run(self, client: RaftClient) -> Result<()> {
        let server = Server::new("127.0.0.1:8844".to_string());
        server.run(client).await
    }
}
