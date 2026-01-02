use std::{
    io::{self, BufRead, Write},
    thread,
};

use crossbeam::channel::{Receiver, Sender, bounded};
use lsp_server::{Connection, Message};
use serde_json::{Number, Value};

pub struct IoThreads {
    reader: thread::JoinHandle<io::Result<()>>,
    writer: thread::JoinHandle<io::Result<()>>,
    dropper: thread::JoinHandle<()>,
}

impl IoThreads {
    pub fn join(self) -> io::Result<()> {
        match self.reader.join() {
            Ok(r) => r?,
            Err(err) => std::panic::panic_any(err),
        }
        match self.dropper.join() {
            Ok(_) => (),
            Err(err) => {
                std::panic::panic_any(err);
            }
        }
        match self.writer.join() {
            Ok(r) => r,
            Err(err) => {
                std::panic::panic_any(err);
            }
        }
    }
}

pub fn io_connection(
    input: impl BufRead + 'static + Send + Sync,
    output: impl Write + 'static + Send + Sync,
) -> (Connection, IoThreads) {
    let (sender, receiver, threads) = io_transport(input, output);
    (Connection { sender, receiver }, threads)
}

/// Creates an LSP connection via stdio.
pub(crate) fn io_transport(
    mut input: impl BufRead + 'static + Send + Sync,
    mut output: impl Write + 'static + Send + Sync,
) -> (Sender<Message>, Receiver<Message>, IoThreads) {
    let (drop_sender, drop_receiver) = bounded::<Message>(0);
    let (writer_sender, writer_receiver) = bounded::<Message>(0);
    let writer = thread::Builder::new()
        .name("LspServerWriter".to_owned())
        .spawn(move || {
            writer_receiver.into_iter().try_for_each(|it| {
                let result = it.write(&mut output);
                let _ = drop_sender.send(it);
                result
            })
        })
        .unwrap();
    let dropper = thread::Builder::new()
        .name("LspMessageDropper".to_owned())
        .spawn(move || drop_receiver.into_iter().for_each(drop))
        .unwrap();
    let (reader_sender, reader_receiver) = bounded::<Message>(0);
    let reader = thread::Builder::new()
        .name("LspServerReader".to_owned())
        .spawn(move || {
            while let Some(msg) = Message::read(&mut input)? {
                let is_exit = matches!(&msg, Message::Notification(n) if n.method == "exit");

                log::debug!("sending message {msg:#?}");
                if let Err(e) = reader_sender.send(msg) {
                    return Err(io::Error::other(e));
                }

                if is_exit {
                    break;
                }
            }
            Ok(())
        })
        .unwrap();
    let threads = IoThreads {
        reader,
        writer,
        dropper,
    };
    (writer_sender, reader_receiver, threads)
}

const CHANGE_SHIFT: u128 = 6;

pub fn transform_client_to_server(mut message: Message) -> Message {
    match &mut message {
        Message::Request(req) => transform_cs(&mut req.params),
        Message::Response(res) => {
            if let Some(res) = res.result.as_mut() {
                transform_cs(res);
            }
        }
        Message::Notification(not) => transform_cs(&mut not.params),
    }
    message
}

fn transform_cs(value: &mut Value) {
    if let Some(obj) = value.as_object_mut()
        && let Some(Value::Number(line)) = obj.get("line")
        && line.as_u64().unwrap() == 0
    {
        if let Some(Value::Number(character)) = obj.get_mut("character") {
            *character = Number::from_u128(character.as_u128().unwrap() + CHANGE_SHIFT).unwrap();
        }
        if let Some(Value::String(line)) = obj.get_mut("text")
            && line.chars().next().is_some_and(|c| c == '=')
        {
            *line = line.replacen('=', "return ", 1);
        }
        if let Some(Value::String(line)) = obj.get_mut("newText")
            && line.chars().next().is_some_and(|c| c == '=')
        {
            *line = line.replacen('=', "return ", 1);
        }
        for (_key, val) in obj.into_iter() {
            transform_cs(val);
        }
    }
}

pub fn transform_server_to_client(message: Message) -> Message {
    message
}
