use std::{
    error::Error,
    io::BufReader,
    process::{Command, Stdio},
};

use bight_lsp::{io_connection, transform_client_to_server, transform_server_to_client};
use crossbeam::channel::TryRecvError;
use lsp_server::Connection;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut child = Command::new("lua-language-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let (server_in, server_out) = (child.stdout.take().unwrap(), child.stdin.take().unwrap());
    let buf_in = BufReader::new(server_in);
    let (server_connection, _server_threads) = io_connection(buf_in, server_out);
    let (client_connection, _client_threads) = Connection::stdio();

    'main_loop: loop {
        // eprintln!("receiving server");
        loop {
            let msg_res = server_connection.receiver.try_recv();
            let msg = match msg_res {
                Ok(msg) => transform_server_to_client(msg),
                Err(TryRecvError::Disconnected) => break 'main_loop,
                Err(TryRecvError::Empty) => break,
            };
            client_connection.sender.send(msg).unwrap();
        }
        // eprintln!("receiving client");
        loop {
            let msg_res = client_connection.receiver.try_recv();
            let msg = match msg_res {
                Ok(msg) => transform_client_to_server(msg),
                Err(TryRecvError::Disconnected) => break 'main_loop,
                Err(TryRecvError::Empty) => break,
            };
            server_connection.sender.send(msg).unwrap();
        }
    }

    Ok(())
}
