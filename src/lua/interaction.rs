use tokio::sync::{mpsc, oneshot};

use crate::table::cell::CellPos;

use super::TableValue;

///
/// The maximum number of value request/response messages that can be queued in message channel
/// 10 seems like a reasonalbe number, but any number can be here (it cannot cause deadlocks as
///    the messages are processed in a separate sync thread which is guaranteed to proccess each
///    message without blocking)
///
pub const MESSAGE_BUFFER_SIZE: usize = 10;

///
/// A struct representing a request for a cell's value that is needed for requester's evaluation
/// requester must be the position of the cell that requested the value, wrong value can cause a deadlock (requester is used for tracking dependencies and detecting dependency cycles)
///
#[derive(Debug)]
pub struct ValueRequest {
    pub requester: CellPos,
    pub cell: CellPos,
    pub sender: oneshot::Sender<TableValue>,
}

///
/// A struct representing response with cell's value
/// The name is misleading, as the response goes the same way as the request (from cell value's
/// evaluator to the message processor) TODO: make a better fitting name
///
#[derive(Debug)]
pub struct ValueResponse {
    pub cell: CellPos,
    pub value: TableValue,
}

///
/// ValueMessage that is either request asking by a cell asking for another cell's value, or
/// response from a cell that finished its evaluation and responds with it
///
#[derive(Debug)]
pub enum ValueMessage {
    Req(ValueRequest),
    Res(ValueResponse),
}

impl ValueMessage {
    /// Shortcut method for creating Req message with ValueRequest
    pub fn request(requester: CellPos, cell: CellPos, sender: oneshot::Sender<TableValue>) -> Self {
        Self::Req(ValueRequest {
            requester,
            cell,
            sender,
        })
    }
    /// Shortcut method for creating Res message with ValueResponse
    pub fn response(cell: CellPos, value: TableValue) -> Self {
        Self::Res(ValueResponse { cell, value })
    }
}

pub type MessageSender = mpsc::Sender<ValueMessage>;
pub type MessageReceiver = mpsc::Receiver<ValueMessage>;

/// Function that creates ValueMessage mpsc channel
pub fn message_channel() -> (MessageSender, MessageReceiver) {
    mpsc::channel::<ValueMessage>(MESSAGE_BUFFER_SIZE)
}

pub type ValueSender = oneshot::Sender<TableValue>;
pub type ValueReceiver = oneshot::Receiver<TableValue>;
/// Function that creates TableValue oneshot channel
fn value_channel() -> (ValueSender, ValueReceiver) {
    oneshot::channel()
}

///
/// Helper for sending messages
/// Stores and uses a ValueMessage Sender to send requests (that have the stored position as the
/// requester) and respond with value
///
/// Panic
/// Panics if the request's oneshot channel was closed and not answered. The channel should never
/// be closed. In case of errors TableValue::Error should be sent on the channel
///
#[derive(Clone, Debug)]
pub struct Communicator {
    sender: MessageSender,
    pos: CellPos,
}

impl Communicator {
    ///
    /// Requests cell's value via self.sender
    /// Panic
    /// Panics if the request's oneshot channel was closed and not answered. The channel should never be closed. In case of errors TableValue::Error should be sent on the channel
    ///
    pub async fn request(&mut self, cell: CellPos) -> mlua::Result<TableValue> {
        let (value_sender, reciever) = value_channel();
        let msg = ValueMessage::request(self.pos, cell, value_sender);
        log::debug!("Requested {} by {}", cell, self.pos);
        _ = self.sender.send(msg).await; // We don't really care if the message is delivered (and
        // cannot be sure, as the channel may be closed after the message in in the buffer), as
        // long as there's an answer in the request's channel
        let val = Ok(reciever.await.expect("The channel should never be closed. On errors that do not allow to continue evaluation all requests should be answered with errors instead of closing the channels"));
        log::debug!("Got {}'s value for {}", cell, self.pos);
        val
    }

    ///
    /// Send respond message via self.sender
    /// This method takes ownership as no further commincation after response should be possible
    ///
    pub async fn respond(self, value: TableValue) {
        let msg = ValueMessage::response(self.pos, value);
        _ = self.sender.send(msg).await; // We don't care if the response is delivered
    }

    pub fn new(pos: CellPos, sender: MessageSender) -> Self {
        Self { sender, pos }
    }
}
