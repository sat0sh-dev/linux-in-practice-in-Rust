use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::mem;
use std::time::Instant;
use std::hint::black_box;
use plotters::prelude::*;

const NLOOP_FOR_ESTIMATION: u64 = 10_00_000_000;
const NLOOP_PROGRESS: usize = 100;

fn usage(prog_name: &str) {
    eprintln!("Usage: {} <concurrency", prog_name);
    eprintln!();
    eprintln!("  Visualize scheduler behavior with <concurrency> process on CPU 0");
    std::process::exit(1);
}

/// Estimate loop count per 1 milli second
fn estimate_loops_per_msec() -> u64 {
    let start = Instant::now();
    for i in 0..NLOOP_FOR_ESTIMATION {
        // Busy loop
        black_box(i);
    }
    let elapsed_ms = start.elapsed().as_millis() as u64;
    if elapsed_ms == 0 {
        return NLOOP_FOR_ESTIMATION;
    }
    NLOOP_FOR_ESTIMATION / elapsed_ms
}

/// Set CPU affinity to CPU 0
fn set_affinity_to_cpu0() {
    unsafe {
        let mut set: libc::cpu_set_t = mem::zeroed();
        libc::CPU_ZERO(&mut set);
        libc::CPU_SET(0, &mut set);
        libc::sched_setaffinity(0, mem::size_of::<libc::cpu_set_t>(), &set);
    }
}

/// Child process: Record progress and output to file
fn child_fn(id: usize, nloop_per_msec: u64, start: Instant) {
    let mut progress = vec![0f64; NLOOP_PROGRESS];

    for i in 0..NLOOP_PROGRESS {
        for j in 0..nloop_per_msec {
            //Busy loop
            black_box(j);
        }
        progress[i] = start.elapsed().as_secs_f64() * 1000.0; // milli seconds
    }

    // Write data to file
    let filename = format!("{}.data", id);
    let mut file = File::create(&filename).expect("Failed to create data file");
    for (i, &elapsed_ms) in progress.iter().enumerate() {
        writeln!(file, "{}\t{}", elapsed_ms, i).expect("Failed to write data");
    }
}

/// Read data file
fn load_data(id: usize) -> Vec<(f64, f64)> {
    let filename = format!("{}.data", id);
    let file = File::open(&filename).expect("Failed to open data file");
    let reader = BufReader::new(file);

    reader
    .lines()
    .filter_map(|line| {
        let line = line.ok()?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() == 2 {
            let x: f64 = parts[0].parse().ok()?;
            let y: f64 = parts[1].parse().ok()?;
            Some((x, y))
        } else {
            None
        }
    })
    .collect()
}

/// Draw the graph
fn plot_sched(concurrency: usize) -> Result<(), Box<dyn std::error::Error>> {
    let filename = format!("sched-{}.png", concurrency);
    let root = BitMapBackend::new(&filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Load all data and get maximum value of X axis
    let mut all_data: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut max_x: f64 = 0.0;

    for i in 0..concurrency {
        let data = load_data(i);
        if let Some(&(x, _)) = data.last() {
            if x > max_x {
                max_x = x;
            }
        }
        all_data.push(data);
    }

    let mut chart = ChartBuilder::on(&root)
    .caption(
        format!("Scheduler visualization (concurrency={})", concurrency),
        ("sans-serif", 20),
    )
    .margin(10)
    .x_label_area_size(40)
    .y_label_area_size(50)
    .build_cartesian_2d(0.0..max_x, 0.0..100.0)?;

    chart
        .configure_mesh()
        .x_desc("Elapsed Time[ms]")
        .y_desc("Progress [%]")
        .draw()?;

    // Plot each process data
    let colors = [RED, BLUE, GREEN, MAGENTA, CYAN, YELLOW];

    for (i, data) in all_data.iter().enumerate() {
        let color = colors[i % colors.len()];

        chart
            .draw_series(PointSeries::of_element(
                data.iter().map(|&(x, y)| (x, y)),
                1,
                color,
                &|coord, size, style| {
                    EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
                },
            ))?
            .label(format!("Process {}", i))
            .legend(move |(x, y)| Circle::new((x, y), 3, color.filled()));
    }

    chart
        .configure_series_labels()
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    println!("Graph saved to {}", filename);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog_name = &args[0];

    if args.len() < 2 {
        usage(prog_name);
    }

    let concurrency: usize = args[1].parse().unwrap_or_else(|_| {
        usage(prog_name);
        9
    });

    if concurrency < 1 {
        eprintln!("concurrency must be >= 1");
        usage(prog_name);
    }

    // Fixed CPU 0
    set_affinity_to_cpu0();

    // Estimate loop count per millisecond
    println!("Estimating loops per millisecond...");
    let nloop_per_msec = estimate_loops_per_msec();
    println!("Estimated: {} loops/ms", nloop_per_msec);

    // Record start time
    let start = Instant::now();

    // Fork child process
    for i in 0..concurrency {
        let pid = unsafe {
            libc::fork()
        };

        if pid < 0 {
            eprintln!("fork failed");
            std::process::exit(1);
        } else if pid == 0 {
            // Child process
            child_fn(i, nloop_per_msec, start);
            std::process::exit(0);
        } else {
            // Parent process
            println!("Started process {} with PID {}", i, pid);
        }
    }

    // Wait for all child process
    for _ in 0..concurrency {
        unsafe {
            libc::wait(std::ptr::null_mut());
        }
    }

    println!("\nAll process finished.");
    
    // Plot
    if let Err(e) = plot_sched(concurrency) {
        eprintln!("Failed to plot: {}", e);
    }
}