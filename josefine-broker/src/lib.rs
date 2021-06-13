#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use josefine_raft::rpc::Request;
use josefine_raft::rpc::Response;
use josefine_core::error::Result;

mod entry;
mod index;
mod log;
mod partition;
mod segment;
mod server;
pub mod fsm;

pub struct JosefineBroker {}

impl JosefineBroker {
    pub fn new() -> Self {
        JosefineBroker {}
    }

    pub fn run(self, client_tx: UnboundedSender<(Request, oneshot::Sender<Result<Response>>)>) -> Result<()> {
        unimplemented!()
    }
}
