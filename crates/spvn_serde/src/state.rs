use crate::ASGIResponse;
use bytes::Bytes;

use futures::lock::Mutex;

use std::{collections::HashMap, time::Instant};
use std::sync::Arc;

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum StateKeys {
    HTTPResponseBody,
    HTTPResponseStart,
}
use tokio::sync::mpsc::{Receiver, Sender};

pub type State = Arc<Mutex<HashMap<Instant, Arc<ASGIResponse>>>>;
pub type HeaderState = Arc<Mutex<HashMap<String, Bytes>>>;
pub type Sending = Arc<Mutex<Sender<Bytes>>>;
pub type Polling = Arc<Mutex<Receiver<Bytes>>>;
