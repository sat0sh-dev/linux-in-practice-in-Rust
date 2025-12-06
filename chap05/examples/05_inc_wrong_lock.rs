use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    // Wait until lock file does't exist (wrong implementation)
    loop {
        let output = Command::new("test")
            .arg("-e")
            .arg("lock")
            .status()
            .expect("Failed to execute test command");

        if !output.success() {
            // Lock file does't exist
            break;
        }

        thread::sleep(Duration::from_millis(10));
    }

    // Create lock file
    Command::new("touch")
        .arg("lock")
        .status()
        .expect("Failed to create lock file");

    // Read count
    let output = Command::new("cat")
        .arg("count")
        .output()
        .expect("Failed to execute cat command");

    let count_str = String::from_utf8_lossy(&output.stdout);
    let mut count: u32 = count_str
        .trim()
        .parse()
        .expect("Failed to parse count");

    // Inclrement
    count += 1;

    // Write back
    Command::new("sh")
        .arg("-c")
        .arg(format!("echo {} > count", count))
        .status()
        .expect("Failed to write count file");

    // Remove lock
    Command::new("rm")
        .arg("-f")
        .arg("lock")
        .status()
        .expect("Failed to remove lock file");
    
}