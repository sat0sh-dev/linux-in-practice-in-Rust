fn main() {
    let mut data = 1000;

    println!("Before create child process: {}", data);

    let pid = unsafe {
      libc::fork()  
    };

    if pid < 0 {
        eprintln!("Failed to fork()");
        std::process::exit(1);
    } else if pid == 0 {
        data *= 2; 
        std::process::exit(0);
    } else {
        unsafe {
            libc::waitpid(pid, std::ptr::null_mut(), 0);
        };
        println!("After exit child process: {}", data);
    }
}