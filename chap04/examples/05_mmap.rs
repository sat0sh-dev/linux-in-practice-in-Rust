use std::process::{self, Command};

fn show_memory_map(label: &str) {
    let pid = process::id();
    println!("\n*** {} ***", label);

    let output = Command::new("cat")
        .arg(format!("/proc/{}/maps", pid))
        .output()
        .expect("Failed to execute cat command");

    print!("{}", String::from_utf8_lossy(&output.stdout));
}

fn main() {
    const ALLOC_SIZE: usize = 1024 * 1024 * 1024;

    // let pid = process::id();

    show_memory_map("Memory map before allocating new memory region");

    // mmap() system call to allocate 1GB memory region
    let data = unsafe {
        libc::mmap(
            std::ptr::null_mut(), 
            ALLOC_SIZE, 
            libc::PROT_READ | libc::PROT_WRITE, 
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE, 
            -1, 
            0
        )
    };

    if data == libc::MAP_FAILED {
        eprintln!("mmap() failed");
        std::process::exit(1);
    }

    println!();
    println!("*** New Memory region address = {:p}, size = {:#x} ***", data, ALLOC_SIZE);
    println!();

    show_memory_map("Memory map after allocating new memory region");

    // Clean up (optional, OS will do this on exit)
    unsafe {
        libc::munmap(data, ALLOC_SIZE);
    }

}