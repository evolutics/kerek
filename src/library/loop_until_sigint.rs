use anyhow::Context;
use std::sync::mpsc;

pub fn go<F: Fn() -> anyhow::Result<()>, G: Fn() -> anyhow::Result<()>, H: Fn()>(
    set_up: F,
    iterate: G,
    tear_down: H,
) -> anyhow::Result<()> {
    let result = set_up().and_then(|_| {
        let (sender, receiver) = mpsc::channel();

        ctrlc::set_handler(move || {
            eprintln!("SIGINT acknowledged.");
            sender.send(()).expect("Unable to send to channel.");
        })?;

        loop {
            match receiver.try_recv() {
                Err(mpsc::TryRecvError::Empty) => (),
                result => break result.context("Unable to receive from channel."),
            }
            let result = iterate();
            if result.is_err() {
                break result;
            }
        }
    });

    tear_down();
    result
}
