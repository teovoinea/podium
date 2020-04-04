use crate::query_executor::QueryResponse;
use crossbeam::channel::{Receiver, Sender};

pub struct AppState {
    pub query_sender: Sender<String>,
    pub result_receiver: Receiver<QueryResponse>,
}
