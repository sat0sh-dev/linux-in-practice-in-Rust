use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::Command;
use std::time::Instant;
use plotters::prelude::*;

fn usage(prog_name: &str) -> ! {
    eprintln!("Usage: {} [-m] <max_nproc>", prog_name);
    eprintln!();
    eprintln!("  Measure performance metrics for 1 to <max_nproc> processes.");
    eprintln!("  Saves results to 'cpuperf.data' and generates graphs.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -m: USe multiple CPUs (passed to multiload)");
    std::process::exit(1);
}

/// Run multiload with the specified number of processes and measure execution time
fn measure(nproc: usize, multi_cpu: bool) -> f64 {
    let multiload_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("02_multiload");

    let start = Instant::now();

    let mut cmd = Command::new(&multiload_path);
    if multi_cpu {
        cmd.arg("-m");
    }
    cmd.arg(nproc.to_string());

    let output = cmd.output().expect("Failed to run multiload");

    if !output.status.success() {
        eprintln!("multiload failed for nproc={}", nproc);
        return 0.0;
    }

    start.elapsed().as_secs_f64()
}

/// Create cpuperf.data
fn create_perf_data(max_proc: usize, multi_cpu: bool) -> std::io::Result<()> {
    let mut file = File::create("cpuperf.data")?;

    println!("Running performance tests for 1 to {} processes...", max_proc);

    for nproc in 1..=max_proc {
        let total_real = measure(nproc, multi_cpu);

        let avg_tat = if multi_cpu {
            total_real
        } else {
            let unit_time = total_real /nproc as f64;
            unit_time * (nproc as f64 + 1.0) / 2.0
        };

        let throughput = nproc as f64 / total_real;

        writeln!(file, "{}\t{:.3}\t{:.3}", nproc, avg_tat, throughput)?;
        println!("nproc={}: avg_tat={:.3}/s, throughput={:.3} proc/s",
                    nproc, avg_tat, throughput);
    }

    Ok(())
}

/// Read cpuperf.data
fn load_perf_data() -> Vec<(f64, f64, f64)> {
    let file = File::open("cpuperf.data").expect("Failed to open cpuperf.data");
    let reader = BufReader::new(file);

    reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 3 {
                let nproc: f64 = parts[0].parse().ok()?;
                let avg_tat: f64 = parts[1].parse().ok()?;
                let throughput: f64 = parts[2].parse().ok()?;
                Some((nproc, avg_tat, throughput))
            } else {
                None
            }
        })
        .collect()
}

/// Draw graph of average turnaround time
fn plot_avg_tat(data: &[(f64, f64, f64)], max_nproc: usize) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("avg-tat.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_tat = data.iter().map(|(_, tat, _)| *tat).fold(0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Average Turnaround Time", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..(max_nproc as f64 + 1.0), 0.0..max_tat * 1.1)?;

    chart
        .configure_mesh()
        .x_desc("Number of Processes")
        .y_desc("Average TAT [seconds]")
        .draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().map(|(nproc, tat, _)| (*nproc, *tat)),
        &RED,
    ))?;

    chart.draw_series(PointSeries::of_element(
        data.iter().map(|(nproc, tat, _)| (*nproc, *tat)),
        3,
        &RED,
        &|coord, size, style| {
            EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
        },
    ))?;

    root.present()?;
    println!("Graph saved to: avg-tat.png");
    Ok(())
}

/// Draw graph of throughput
fn plot_throughput(data: &[(f64, f64, f64)], max_nproc: usize) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("throughput.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_throughput = data.iter().map(|(_, _, tp)| *tp).fold(0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Throughput", ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..(max_nproc as f64 + 1.0), 0.0..max_throughput * 1.1)?;

    chart
        .configure_mesh()
        .x_desc("Number of processes")
        .y_desc("Throughput [processes/sec")
        .draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().map(|(nproc, _, tp)| (*nproc, *tp)),
        &BLUE,
    ))?;

    chart.draw_series(PointSeries::of_element(
        data.iter().map(|(nproc, _, tp)| (*nproc, *tp)),
        3,
        &BLACK,
        &|coord, size, style| {
            EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
        },
    ))?;

    root.present()?;
    println!("Graph saved to: thorouput.png");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog_name = &args[0];

    let mut multi_cpu = false;
    let mut max_nproc: Option<usize> = None;

    let mut i = 1;
    while i < args.iter().len() {
        if args[i] == "-m" {
            multi_cpu = true;
        } else {
            max_nproc = Some(args[i].parse().unwrap_or_else(|_| usage(prog_name)));
        }
        i += 1;
    }

    let max_nproc = max_nproc.unwrap_or_else(|| usage(prog_name));

    if max_nproc < 1 {
        eprintln!("max_nproc must be >= 1");
        usage(prog_name);
    }

    // Create data file
    create_perf_data(max_nproc, multi_cpu).expect("Failed to create perf data");

    // Load perf data
    let data = load_perf_data();

    // Draw graph
    if let Err(e) = plot_avg_tat(&data, max_nproc) {
        eprintln!("Failed to plot avg TAT: {}", e);
    }

    if let Err(e) = plot_throughput(&data, max_nproc) {
        eprintln!("Failed to plot throughput: {}", e);
    }
}