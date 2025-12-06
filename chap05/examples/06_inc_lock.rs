use std::process::Command;

fn main() {
    let output = Command::new("flock")
        .arg("lock")
        .arg("./target/release/examples/04_inc")
        .status()
        .expect("Failed to execute 04_inc");

    if !output.success() {
        eprintln!("Failed to 04_inc");
        std::process::exit(1);
    }
}