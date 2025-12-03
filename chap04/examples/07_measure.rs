use std::process::Command;
use std::thread;
use std::time::Duration;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <process_name", args[0]);
        eprintln!("Example: {} 06_demand-paging", args[0]);
        std::process::exit(1);
    }

    let process_name = &args[1];

    // Find prcess ID using pgrep
    let output = Command::new("pgrep")
        .arg("-o")
        .arg("-f")
        .arg(process_name)
        .output()
        .expect("Failed to execute pgrep");

    if !output.status.success() {
        eprintln!("{} process not found. Please run it first.", process_name);
        std::process::exit(1);
    }

    let pid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    if pid.is_empty() {
        eprintln!("{} process not found. Please run it first.", process_name);
        std::process::exit(1);
    }

    println!("Monitoring process {} (PID: {})", process_name, pid);
    println!("Format: DATE: VSZ RSS MAJ_FLT MIN_FLT");
    println!();

    loop {
        // Get current timestamp
        let date_output = Command::new("date")
            .output()
            .expect("Failed to execute date");
        let date = String::from_utf8_lossy(&date_output.stdout)
            .trim()
            .to_string();

        // Get Process information
        // -h: no header
        // -o output format (vsz, rss, maj_flt, min_flt)
        // -p process ID
        let ps_output = Command::new("ps")
            .arg("-h")
            .arg("-o")
            .arg("vsz,rss,maj_flt,min_flt")
            .arg("-p")
            .arg(&pid)
            .output()
            .expect("Failed to execute ps");

        if !ps_output.status.success() {
            eprintln!("{}: {} process terminated.", date, process_name);
            std::process::exit(1);
        }

        let info = String::from_utf8_lossy(&ps_output.stdout)
            .trim()
            .to_string();

        println!("{}: {}", date, info);

        thread::sleep(Duration::from_secs(1));
    }
}