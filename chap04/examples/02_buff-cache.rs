use std::process::Command;

fn main() {
    println!("Measure system memory usage before create cache file");
    let output = Command::new("free")
        .output()
        .expect("Failed to execute free command");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("Create the 1[GiB] chche file");
    Command::new("dd")
        .args(["if=/dev/zero", "of=testfile", "bs=1M", "count=1K"])
        .output()
        .expect("Failed to execute dd command");

    println!("Measure system memory usage after create chche file");
    let output = Command::new("free")
        .output()
        .expect("Failed to execute free command");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("Remove chche file");
    Command::new("rm")
        .arg("testfile")
        .output()
        .expect("Failed to execute rm command");

    println!("Measure system memory usage after remove chche file");
    let output = Command::new("free")
        .output()
        .expect("Failed to execute free command");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}