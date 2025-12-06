fn main() {
    const PAGE_SIZE: usize = 4096;
    let mut data = 100;
    let share_mem = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            PAGE_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_ANONYMOUS,
            -1,
            0
        )
    };
    let ptr = share_mem as *mut u8;
    unsafe {
        *ptr.add(0) = data;
    };
    println!("Before create child process: {}", data);

    let pid = unsafe {
      libc::fork()  
    };

    if pid < 0 {
        eprintln!("Failed to fork()");
        std::process::exit(1);
    } else if pid == 0 {
        unsafe {
            *ptr.add(0) *= 2;
        }
        std::process::exit(0);
    } else {
        unsafe {
            libc::waitpid(pid, std::ptr::null_mut(), 0);
        };
        unsafe {
            data = *ptr.add(0);
        }
        println!("After exit child process: {}", data);
    }
}