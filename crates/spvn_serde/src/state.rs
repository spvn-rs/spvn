use crate::ASGIResponse;
use bytes::Bytes;
use bytes_expand::BytesMut;
use futures::lock::Mutex;


use std::collections::HashMap;
use std::sync::Arc;
pub enum StateKeys {
    HTTPResponseBody,
    HTTPResponseStart,
}
use tokio::sync::{
    mpsc::{Sender, Receiver},

};



pub type State = Arc<Mutex<HashMap<StateKeys, ASGIResponse>>>;
pub type HeaderState = Arc<Mutex<HashMap<StateKeys, Bytes>>>;
pub type Sending = Arc<Mutex<Sender<Bytes>>>;
pub type Polling = Arc<Mutex<Receiver<Bytes>>>;
