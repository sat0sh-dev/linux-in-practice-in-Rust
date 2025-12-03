use std::io::{self};
use ::std::thread;
use std::time::{Duration, SystemTime};

fn show_message(msg: &str) {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs% 60;

    println!("[{:02}:{:02}:{:02}] {}", hours, mins, secs, msg);

}

fn main() {
    const ALLOC_SIZE: usize = 100 * 1024 * 1024;
    const ACCESS_UNIT: usize = 10 * 1024 * 1024;

    show_message("Allocating new memory region using mmap() system call. Press Enter to access the allocated memory region (10MiB at a time, 100MiB in total):)");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let memregion = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            ALLOC_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0)
    };

    if memregion == libc::MAP_FAILED {
        eprintln!("mmap() failed");
        std::process::exit(1);
    }

    show_message("Allocated new memory region. Press Enter to access the memory region. (10MiB at a time, 100MiB in total");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    // Access memory in chunks
    let ptr = memregion as *mut u8;
    for i in 0..ALLOC_SIZE {
        unsafe {
            *ptr.add(i) = 0;
        }

        //Print progress every ACCESS_UNIT (10MiB)
        if i % ACCESS_UNIT == 0 && i != 0 {
            show_message(&format!("{} MiB accessed", i / (1024 * 1024)));
            thread::sleep(Duration::from_secs(1));
        }
    }

    show_message("Accessed all of allocated memory region. Press Enter to exit:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    // Clean up
    unsafe {
        libc::munmap(memregion, ALLOC_SIZE);
    }

}