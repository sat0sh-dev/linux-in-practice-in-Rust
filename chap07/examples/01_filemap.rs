use std::process::{Command};
use std::fs::{OpenOptions};
use std::os::unix::io::AsRawFd;
use std::io::Write;

fn show_memory_map(label: &str) {
    let pid = std::process::id();
    println!("\n*** {} ***", label);

    let output = Command::new("cat")
        .arg(format!("/proc/{}/maps", pid))
        .output()
        .expect("Failed to executge cat command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn main() {
    show_memory_map("Process virtual address space before mapping testifile");

    // Open testfile
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("testfile")
        .expect("Failed to open testfile");

    // Write some data to ensure file has content
    file.write_all(b"Hello").expect("Failed to write to file");
    file.sync_all().expect("Failed to sync file");

    // mmap() system call to map 5 bytes of memory
    let data = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            5,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            0
        )
    };

    if data == libc::MAP_FAILED {
        eprintln!("mmap() failed");
        std::process::exit(1);
    }

    println!();
    println!("testfile mapped address: {:p}", data);
    println!();

    show_memory_map("Process virtual address space after mapping testfile");

    // Cleanup
    unsafe {
        libc::munmap(data, 5);
    }
}
