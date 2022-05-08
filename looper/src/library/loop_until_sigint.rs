use anyhow::Context;
use std::sync::mpsc;

pub fn go<F: Fn() -> anyhow::Result<()>, G: Fn()>(iterate: F, clean_up: G) -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::channel();

    ctrlc::set_handler(move || {
        eprintln!("SIGINT acknowledged.");
        sender.send(()).expect("Unable to send to channel.");
    })?;

    let result = loop {
        match receiver.try_recv() {
            Err(mpsc::TryRecvError::Empty) => (),
            result => break result.context("Unable to receive from channel."),
        }
        let result = iterate();
        if result.is_err() {
            break result;
        }
    };

    clean_up();
    result
}
