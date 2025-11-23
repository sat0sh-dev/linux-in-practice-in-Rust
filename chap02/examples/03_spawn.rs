use std::ffi::CString;
use libc::posix_spawn;

fn main() {
    unsafe {
        let path = CString::new("/bin/echo").unwrap();
        let arg0 = CString::new("echo").unwrap();
        let arg1 = CString::new("Hello from the spawned process!").unwrap();

        let args = [
            arg0.as_ptr() as *mut libc::c_char,
            arg1.as_ptr() as *mut libc::c_char,
            std::ptr::null_mut(),
        ];
        let mut pid: libc::pid_t = 0;

        let result = posix_spawn(
            &mut pid,
            path.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            args.as_ptr(),
            std::ptr::null_mut(),
        );

        if result != 0 {
            eprint!("posix_spawn failed");
            return;
        }

        // Wait for the spawned process to finish
        libc::waitpid(pid, std::ptr::null_mut(), 0);
    }
}