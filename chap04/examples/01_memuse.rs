use std::process::Command;
use std::hint::black_box;

fn main() {
    const MEM_SIZE: usize = 100_000_000;

    println!("Before allocation:");
    let output = Command::new("free")
        .output()
        .expect("Failed to execute free command");
    print!("{}", String::from_utf8_lossy(&output.stdout));

    let mut array = vec![0u8; MEM_SIZE];

    println!("\nAfter allocation (but before access):");
    let output = Command::new("free")
        .output()
        .expect("Failed to execute free command");
    print!("{}", String::from_utf8_lossy(&output.stdout));

    // Touch each page to allocate physical memory
    for i in (0..MEM_SIZE).step_by(4096) { 
        array[i] = black_box(i as u8);
    }
    black_box(&array);

    println!("\nAfter memory access:");
    let output = Command::new("free")
        .output()
        .expect("Failed to execute free command");
    print!("{}", String::from_utf8_lossy(&output.stdout));

}