use std::process::Command;

fn main() {
    // Read the count file using cat command
    let output = Command::new("cat")
        .arg("count")
        .output()
        .expect("Failed to execute cat command");

    // Convert output to string
    let count_str = String::from_utf8_lossy(&output.stdout);

    // Parse string to number
    let count: u32 = count_str
        .trim()
        .parse()
        .expect("Failed to parse count");

    let count = count + 1;

    Command::new("sh")
        .arg("-c")
        .arg(format!("echo {} > count", count))
        .output()
        .expect("Failed to execute echo command");
}