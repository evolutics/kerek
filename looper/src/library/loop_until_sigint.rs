use anyhow::Context;
use std::sync::mpsc;

pub fn go<F: Fn() -> anyhow::Result<()>, G: Fn() -> anyhow::Result<()>, H: Fn()>(
    in_: In<F, G, H>,
) -> anyhow::Result<()> {
    let result = (in_.set_up)().and_then(|_| {
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
            let result = (in_.iterate)();
            if result.is_err() {
                break result;
            }
        }
    });

    (in_.tear_down)();
    result
}

pub struct In<F, G, H> {
    pub set_up: F,
    pub iterate: G,
    pub tear_down: H,
}
