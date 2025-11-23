use libc::pause;

fn main() {
    unsafe {
        println!("Process is pausing. Press Ctrl+C to terminate.");
        pause();
    }
}