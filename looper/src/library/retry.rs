use rand::Rng;
use std::thread;
use std::time;

pub fn go<F: Fn() -> anyhow::Result<()>>(in_: In<F>) -> anyhow::Result<()> {
    let start = time::Instant::now();
    let mut random = rand::thread_rng();

    loop {
        let result = (in_.run)();

        if result.is_ok() || start.elapsed() >= in_.total_duration_limit {
            break result;
        }

        thread::sleep(in_.expected_retry_pause.mul_f32(random.gen_range(0.0..2.0)))
    }
}

pub struct In<F> {
    pub total_duration_limit: time::Duration,
    pub expected_retry_pause: time::Duration,
    pub run: F,
}
