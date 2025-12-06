use std::process::Command;

const ACCESS_SIZE: usize = 100 * 1024 * 1024;
const PAGE_SIZE: usize = 4096;

fn access(data: *mut u8) {
    for i in (0..ACCESS_SIZE).step_by(PAGE_SIZE) {
        unsafe {
            *data.add(i) = 0;
        }
    }
}

fn show_meminfo(msg: &str, process: &str) {
    println!("{}", msg);
    println!("free command result");
    let output = Command::new("free")
        .output()
        .expect("Failed to execute free command");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("{} memory information", process);
    
    let pid = std::process::id();
    let output = Command::new("ps")
        .arg("-orss,maj_flt,min_flt")
        .arg(format!("{}", pid))
        .output()
        .expect("Failed to execute ps command");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn main() {
    let data = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            ACCESS_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0)
    };

    let ptr = data as *mut u8;
    access(ptr);
    show_meminfo("Before create child process", "Parent process");

    let pid = unsafe {
        libc::fork()
    };

    if pid < 0 {
        eprintln!("Failed to fork()");
        std::process::exit(1);
    } else if pid == 0 {
        show_meminfo("After create child process", "Child process");
        access(ptr);
        show_meminfo("After memory access by child process", "Child process");
        std::process::exit(1);
    } else {
        unsafe {
            libc::waitpid(pid, std::ptr::null_mut(), 0);
        };
    }
}

