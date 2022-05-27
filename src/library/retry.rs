use std::thread;
use std::time;

pub fn go<F: Fn() -> anyhow::Result<()>>(in_: In<F>) -> anyhow::Result<()> {
    let start = time::Instant::now();

    loop {
        let result = (in_.run)();

        if result.is_ok() || start.elapsed() >= in_.total_duration_limit {
            break result;
        }

        thread::sleep(in_.retry_pause)
    }
}

pub struct In<F> {
    pub total_duration_limit: time::Duration,
    pub retry_pause: time::Duration,
    pub run: F,
}
