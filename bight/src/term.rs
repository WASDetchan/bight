pub mod view;

use std::future;

use futures::{Stream, StreamExt};

use crate::key::Key;

pub fn key_event_stream() -> impl Stream<Item = Key> {
    let event_stream = crossterm::event::EventStream::new();
    event_stream.filter_map(|event| {
        let event = event.expect("couldn't find when this can be Err in docs :(");
        future::ready(event.try_into().ok())
    })
}
