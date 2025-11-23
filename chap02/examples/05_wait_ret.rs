use std::process::Command;

fn main() {
    // Excute the "false" command(execute as background process by spwan)
    let mut child = Command::new("false")
        .spawn()
        .expect("failed to execute process");

    // Wait for the child process to finish and collect its exit status
    let status = child
        .wait()
        .expect("failed to wait on child");

    // Obtain and print the exit code
    let exit_code = status.code().unwrap_or(-1);
    println!("Child process exited with code: {}", exit_code);
}