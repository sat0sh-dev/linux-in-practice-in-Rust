use std::thread::sleep;
use std::time::Duration;

fn main() {
    // Ignore the SIGINT signal (Ctrl+C)
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_IGN);
    }

    println!("SIGINT signal is now ignored. Try pressing Ctrl+C.");
    println!("To exeit: kill -9 {}", std::process::id());

    loop {
        println!("Working...");
        sleep(Duration::from_secs(1));
    }
}