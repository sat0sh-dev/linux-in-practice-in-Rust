use std::ffi::CString;
use libc::{_exit, execve, fork};

fn main() {
    unsafe {
        let pid = fork();
        if pid < 0 {
            eprint!("fork failed");
            return;
        }
        if pid == 0 {
            // child process
            let path = CString::new("/bin/echo").unwrap();
            let arg0 =CString::new("echo").unwrap();
            let arg1 = CString::new("Hello from the child process!").unwrap();

            let args = [arg0.as_ptr(), arg1.as_ptr(), std::ptr::null()];
            let env = [std::ptr::null()];

            execve(path.as_ptr(), args.as_ptr(), env.as_ptr());

            // If execve failed, reach here
            _exit(1)
        }
        else {
            libc::waitpid(pid, std::ptr::null_mut(), 0);
        }
    }
}