use std::os::unix::process::parent_id;

fn main() {
    loop {
        let _ppid = parent_id();        
    }
}