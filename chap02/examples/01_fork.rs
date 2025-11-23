use libc::fork;
use std::process;
use std::os::unix::process::parent_id;

fn main() {
    unsafe {
        let pid = fork();
        if pid < 0 {
            eprint!("fork failed");
            return;
        }
        if pid == 0 {
            // Child process
            println!("Child: My PID = {}, Parent PID = {}",
                       process::id(), parent_id());
        }
        else {
            // Parent process
            println!("Parent: My PID = {}, Child PID = {}",
                       process::id(), pid);
            libc::waitpid(pid, std::ptr::null_mut(), 0);
        }
    }
}