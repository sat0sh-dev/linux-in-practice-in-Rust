use std::env;
use std::mem;
use std::process::{Command, Child};
use std::time::Instant;

fn usage(program_namee: &str) {
    eprintln!("Usage: {} <num_processes>", program_namee);
    eprintln!();
    eprintln!("  Run <concurrency> load processes and wait for all to finish.");
    eprintln!("  By default, all processes run on CPU 0 only.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -m: Allow processes to run on multiple CPUs.");
    std::process::exit(1);
}

fn timeval_to_secs(tv: &libc::timeval) -> f64 {
    (tv.tv_sec as f64) + (tv.tv_usec as f64) / 1_000_000.0
}

fn wait_with_rusage(pid: u32) -> (i32, libc::rusage) {
    unsafe {
        let mut status: i32 = 0;
        let mut rusage: libc::rusage = mem::zeroed();
        libc::wait4(pid as i32, &mut status, 0, &mut rusage);
        (status, rusage)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog_name = &args[0];

    // Analyze command-line arguments
    let mut multiple_cpu = false;
    let mut concurrency: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-m" {
            multiple_cpu = true;
        } else {
            concurrency = Some(args[i].parse().unwrap_or_else(|_| {
                usage(prog_name);
                0
            }));
        }
        i += 1;
    }

    let concurrency = match concurrency {
        Some(n) if n > 0 => n,
        _ => {
            usage(prog_name);
            return;
        }
    };

    // Get the path to the load process executable
    let load_program = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("01_load");
    
    println!("Starting {} processes (muliti_cpu={})...", concurrency, multiple_cpu);

    // Execute child processes
    let mut children: Vec<(Child, Instant)> = Vec::new();

    for i in 0..concurrency {
        let child = if multiple_cpu {
            // Run with mutiplr CPUs
            Command::new(&load_program)
                .spawn()
                .expect("Failed to start load process")
        } else {
            // Run on CPU 0 inly
            Command::new("taskset")
                .args(["-c", "0", load_program.to_str().unwrap()])
                .spawn()
                .expect("Failed to start load process")
        };

        println!("Started process {} with PID {}", i, child.id());
        let start_time = Instant::now();
        children.push((child, start_time));
    }

    // Wait for all child processes to finish
    let start = Instant::now();

    for (i, (child, start_time)) in children.into_iter().enumerate() {
        let pid = child.id();

        let (_status, rusage) = wait_with_rusage(pid);

        let real = start_time.elapsed().as_secs_f64();
        let user = timeval_to_secs(&rusage.ru_utime);
        let sys = timeval_to_secs(&rusage.ru_stime);

        println!("Process {} (PID {}) finished:", i, pid);
        println!("  Real time: {:.3?} seconds", real);
        println!("  User time: {:.3?} seconds", user);
        println!("  Sys  time: {:.3?} seconds", sys);
    }

    let total_elapsed = start.elapsed();
    println!("\nTotal elapsed time: {:.3?} seconds", total_elapsed.as_secs_f64());
}