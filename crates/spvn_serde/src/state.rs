use crate::ASGIResponse;
use bytes::Bytes;

use futures::lock::Mutex;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum StateKeys {
    HTTPResponseBody,
    HTTPResponseStart,
}
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Default)]
pub struct StateMap(pub BTreeSet<ASGIResponse>);

pub type State = Arc<Mutex<StateMap>>;
pub type HeaderState = Arc<Mutex<HashMap<String, Bytes>>>;
pub type Sending = Arc<Mutex<Sender<Bytes>>>;
pub type Polling = Arc<Mutex<Receiver<Bytes>>>;
