use std::time::Instant;
use std::hint::black_box;

fn main() {
    const N_LOOPS: u64 = 1_000_000_000;

    let start = Instant::now();

    for i in 0..N_LOOPS {
        black_box(i);
    }

    let elapsed = start.elapsed();
    println!("Elapsed: {:.3?}", elapsed.as_secs_f64());
}