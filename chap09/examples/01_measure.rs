use plotters::prelude::*;
use serde_json::Value;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;

/// COnfiguration structure for block device benchmark
struct Config {
    device: String,
    device_name: String,
}

/// Load configuration from config fie
fn load_config() -> Result<Config, Box<dyn Error>> {
    let config_str = fs::read_to_string("config")?;
    let lines: Vec<&str> = config_str.lines().collect();

    if lines.len() < 2 {
        return Err("Invalid config file format".into());
    }

    Ok(Config {
        device: lines[0].trim().to_string(),
        device_name: lines[1].trim().to_string(),
     })
}

/// Check if the device exists and is a block device
fn validate_device(device: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(device);

    if !path.exists() {
        return Err(format!("Device {} does not exist", device).into());
    }

    // Check if it's a block device using stat
    let output = Command::new("stat")
        .arg("-c")
        .arg("%F")
        .arg(device)
        .output()?;

    let file_type = String::from_utf8_lossy(&output.stdout);

    if !file_type.contains("block special file") && !file_type.contains("regular file") {
        return Err(format!("{} is not a block device", device).into());
    }

    println!("Device type: {}", file_type.trim());

    Ok(())
}

/// Get current I/O scheduler for the device
fn get_scheduler(device: &str) -> Result<String, Box<dyn Error>> {
    // Check if it's a regular file
    let output = Command::new("stat")
        .arg("-c")
        .arg("%F")
        .arg(device)
        .output()?;

    let file_type = String::from_utf8_lossy(&output.stdout);

    if file_type.contains("regular file") {
        return Ok("none".to_string());  // Regular files don't have schedulers
    }

    // Extract device name (e.g., sda from /dev/sda)
    let dev_name = device.trim_start_matches("/dev/");
    let scheduler_path = format!("/sys/block/{}/queue/scheduler", dev_name);

    let content = fs::read_to_string(&scheduler_path)?;

    // Extract current scheduler (matked wwith brackets)
    for part in content.split_whitespace() {
        if part.starts_with('[') && part.ends_with(']') {
            return Ok(part.trim_matches(|c| c == '[' || c== ']').to_string());
        }
    }

    Err("Could not determine current scheduler".into())
}

/// Set I/O scheduler for the device
fn set_scheduler(device: &str, scheduler: &str) -> Result<(), Box<dyn Error>> {
    // Check if it's a regular file
    let output = Command::new("stat")
        .arg("-c")
        .arg("%F")
        .arg(device)
        .output()?;

    let file_type = String::from_utf8_lossy(&output.stdout);

    if file_type.contains("regular file") {
        println!("Skipping scheduler setting for regular file");
        return Ok(());
    }

    let dev_name = device.trim_start_matches("/dev/");
    let scheduler_path = format!("sys/block/{}/queue/scheduler", dev_name);

    let mut file = File::create(&scheduler_path)?;
    file.write_all(scheduler.as_bytes())?;

    println!("Set scheduler to: {}", scheduler);
    Ok(())
}

/// Get current read-ahead value for the device
fn get_read_ahead(device: &str) -> Result<u32, Box<dyn Error>> {
    //Check if it's a regular file
    let output = Command::new("stat")
        .arg("-c")
        .arg("%F")
        .arg(device)
        .output()?;

    let file_type = String::from_utf8_lossy(&output.stdout);
    if file_type.contains("regular file") {
        return Ok(0); // Regular file don't have read-ahead
    }

    let output = Command::new("blockdev")
        .arg("--getra")
        .arg(device)
        .output()?;

    let ra_str = String::from_utf8_lossy(&output.stdout);
    let ra = ra_str.trim().parse::<u32>()?;

    Ok(ra)
}

/// Set read-ahead value for the device
fn set_read_ahead(device: &str, ra_value: u32) -> Result<(), Box<dyn Error>> {
    //Check if it's a regular file
    let output = Command::new("stat")
        .arg("-c")
        .arg("%F")
        .arg(device)
        .output()?;

    let file_type = String::from_utf8_lossy(&output.stdout);
    if file_type.contains("regular file") {
        println!("Skipping read-ahead setting for regular file");
        return Ok(()); // Regular file don't have read-ahead
    }

    let status = Command::new("blockdev")
        .arg("--setra")
        .arg(ra_value.to_string())
        .arg(device)
        .status()?;

    if !status.success() {
        return Err(format!("Failed to set read-ahead to {}", ra_value).into());
    }

    println!("Set read-ahead to: {}", ra_value);
    Ok(())
}

/// Run fio benchmark and extract results
fn run_fio (
    device: &str,
    rw_type: &str,
    num_jobs: u32,
    output_file: &str,
) -> Result<(f64, f64), Box<dyn Error>> {
    println!("Running fio: {} with {}...", rw_type, num_jobs);

    // Run fio benchmark
    let status = Command::new("fio")
        .arg("--name=test")
        .arg(format!("--filename={}", device))
        .arg("--ioengine=libaio")
        .arg(format!("--rw={}", rw_type))
        .arg("--bs=4k")
        .arg("--direct=1")
        .arg(format!("--numjobs={}", num_jobs))
        .arg("--time_based")
        .arg("--runtime=60")
        .arg("--group_reporting")
        .arg("--output-format=json")
        .arg(format!("--output={}", output_file))
        .status()?;

    if !status.success() {
        return Err("fio command failed".into());
    }

    // Parse JSON output
    let json_str = fs::read_to_string(output_file)?;
    let json: Value = serde_json::from_str(&json_str)?;

    // Extract latency and IPOS based on rw_type
    let (latency, iops) = if rw_type == "read" {
        let lat_ns = json["jobs"][0]["read"]["lat_ns"]["mean"]
            .as_f64()
            .unwrap_or(0.0);
        let iops_val = json["jobs"][0]["read"]["lat_ns"]["mean"]
            .as_f64()
            .unwrap_or(0.0);
        (lat_ns / 1000.0, iops_val)
    } else {
        let lat_ns = json["jobs"][0]["write"]["lat_ns"]["mean"]
            .as_f64()
            .unwrap_or(0.0);
        let iops_val = json["jobs"][0]["write"]["lat_ns"]["mean"]
            .as_f64()
            .unwrap_or(0.0);
        (lat_ns / 1000.0, iops_val)
    };

    println!("  Latency: {:.2} usec, IOPS: {:.2}", latency, iops);

    Ok((latency, iops))
    
}

/// Run read benchmark with different and read-ahead values
fn benchmark_read(device: &str, device_name: &str) -> Result<(), Box<dyn Error>> {
    println!("\n=== Starting Read Benchmarks ===\n");

    let schedules = ["mq-deadline", "none"];
    let read_ahead_values = [0, 256];

    for &scheduler in &schedules {
        set_scheduler(device, &scheduler)?;

        for &ra in &read_ahead_values {
            set_read_ahead(device, ra)?;

            let output_json = format!("read-{}-{}-{}.json", device_name, ra, scheduler);
            let output_txt = format!("read-{}-{}-{}.txt", device_name, ra, scheduler);

            // Run benchmark
            let (latency, iops) = run_fio(device, "read", 1, &output_json)?;

            // Save extracted data to text file
            let mut file = File::create(&output_txt)?;
            writeln!(file, "{} {}", latency, iops)?;

            println!("Saved results to: {}", output_txt);
        }
    }

    Ok(())
}

/// Run random write benchmarks with different schedulers and job counts
fn benchmark_randwrite(device: &str, device_name: &str) -> Result<(), Box<dyn Error>> {
    println!("\n=== Starting Random Write Benchmarks ===\n");

    let schedulers = ["mq-deadline", "none"];
    let num_jobs_list = vec![1, 2, 4, 8, 16, 32, 64];

    for &scheduler in &schedulers {
        set_scheduler(device, scheduler)?;

        for &num_jobs in &num_jobs_list {
            let output_json = format!("randwrite-{}-{}-{}.json", device_name, num_jobs, scheduler);
            let output_txt = format!("randwrite-{}-{}-{}.txt", device_name, num_jobs, scheduler);

            // Run benchmark
            let (latency, iops) = run_fio(device, "randwrite", num_jobs, &output_json)?;

            // Save extracted data to text file
            let mut file = File::create(&output_txt)?;
            writeln!(file, "{} {}", latency, iops)?;

            println!("Saved results to: {}\n", output_txt);
        }
    }

    Ok(())
}

/// Process and plot read benchmark data
fn process_read_data(device_name: &str) -> Result<(), Box<dyn Error>> {
    // Data structure: (read_ahead, scheduler, latency, iops)
    let mut data: Vec<(u32, String, f64, f64)> = Vec::new();

    // Read data from the four result files
    for ra in &[0, 256] {
        for sched in &["mq-deadline", "none"] {
            let filename = format!("read-{}-{}-{}.txt", device_name, ra, sched);
            if let Ok(file) = File::open(&filename) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let (Ok(latency), Ok(iops)) = (
                                parts[0].parse::<f64>(),
                                parts[1].parse::<f64>()
                            ) {
                                data.push((*ra, sched.to_string(), latency, iops));
                            }
                        }
                    }
                }
            }
        }
    }

    // Plot 1: Latency comparison
    let output_file = format!("read-{}-latency.png", device_name);
    let root = BitMapBackend::new(&output_file, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_latency = data.iter()
        .map(|(_, _, lat, _)| lat)
        .fold(0.0f64, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Read Latency Comparison ({})", device_name),
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..4f64, 0f64..max_latency * 1.1)?;

    chart
        .configure_mesh()
        .x_labels(4)
        .x_label_formatter(&|x| {
            match  *x as i32 {
                0 => "mq-deadline\nra=0".to_string(),
                1 => "mq-deadline\nra=256".to_string(),
                2 => "none\nra=0".to_string(),
                3 => "none\nra=256".to_string(),
                _ => "".to_string(),
            }
        })
        .y_desc("Latency (usec)")
        .draw()?;

    // Group data by configuration
    let mut deadline_ra0: Vec<f64> = Vec::new();
    let mut deadline_ra256: Vec<f64> = Vec::new();
    let mut none_ra0: Vec<f64> = Vec::new();
    let mut none_ra256: Vec<f64> = Vec::new();

    for (ra, sched, lat, _) in &data {
        match (sched.as_str(), ra) {
            ("mq-deadline", 0) => deadline_ra0.push(*lat),
            ("mq-deadline", 256) => deadline_ra256.push(*lat),
            ("none", 0) => none_ra0.push(*lat),
            ("none", 256) => none_ra256.push(*lat),
            _ => {}
        }
    }

    // Plot scatter points for each configuration
    chart.draw_series(
        deadline_ra0.iter().map(|lat| {
            Circle::new((0.0, *lat), 3, RED.filled())
        })
    )?;
    chart.draw_series(
        deadline_ra256.iter().map(|lat| {
            Circle::new((1.0, *lat), 3, RED.filled())
        })
    )?;
    chart.draw_series(
        none_ra0.iter().map(|lat| {
            Circle::new((2.0, *lat), 3, RED.filled())
        })
    )?;
    chart.draw_series(
        none_ra256.iter().map(|lat| {
            Circle::new((3.0, *lat), 3, RED.filled())
        })
    )?;

    root.present()?;
    println!("Generated: {}", output_file);

    // Plot 2: IOPS comparison
    let output_file = format!("read-{}-iops.png", device_name);
    let root = BitMapBackend::new(&output_file, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_iops = data.iter()
        .map(|(_, _, _, iops)| iops)
        .fold(0.0f64, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Read IOPS Comparison ({})", device_name),
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..4f64,  0f64..max_iops * 1.1)?;

    chart
        .configure_mesh()
        .x_labels(4)
        .x_label_formatter(&|x| {
            match *x as i32 {
                0 => "mq-deadline\nra=0".to_string(),
                1 => "mq-deadline\nra=256".to_string(),
                2 => "none\nra=0".to_string(),
                3 => "none\nra=256".to_string(),
                _ => "".to_string(),
            }
        })
        .y_desc("IOPS")
        .draw()?;

    // Grpup IOPS data
    let mut deadline_ra0_iops: Vec<f64> = Vec::new();
    let mut deadline_ra256_iops: Vec<f64> = Vec::new();
    let mut none_ra0_iops: Vec<f64> = Vec::new();
    let mut none_ra256_iops: Vec<f64> = Vec::new();

    for (ra, sched, _, iops) in &data {
        match (sched.as_str(), ra) {
            ("mq-deadline", 0) => deadline_ra0_iops.push(*iops),
            ("mq-deadline", 256) => deadline_ra256_iops.push(*iops),
            ("none", 0) => none_ra0_iops.push(*iops),
            ("none", 256) => none_ra256_iops.push(*iops),
            _ => {}
        }
    }

    // Plot scatter points
    chart.draw_series(
        deadline_ra0_iops.iter().map(|iops| {
            Circle::new((0.0, *iops), 3, BLUE.filled())
        })
    )?;
    chart.draw_series(
        deadline_ra256_iops.iter().map(|iops| {
            Circle::new((1.0, *iops), 3, BLUE.filled())
        })
    )?;
    chart.draw_series(
        none_ra0_iops.iter().map(|iops| {
            Circle::new((2.0, *iops), 3, BLUE.filled())
        })
    )?;
    chart.draw_series(
        none_ra256_iops.iter().map(|iops| {
            Circle::new((3.0, *iops), 3, BLUE.filled())
        })
    )?;

    root.present()?;
    println!("Generated: {}", output_file);

    Ok(())
}

/// Process and plot random write benchmark data
fn process_randwrite_data(device_name: &str) -> Result<(), Box<dyn Error>> {
    // Data structure: (num_jobs, scheduler, latency, iops)
    let mut data: Vec<(u32, String, f64, f64)> = Vec::new();
    
    let num_jobs_list = vec![1, 2, 4, 8, 16, 32, 64];
    
    // Read data from result files
    for &nj in &num_jobs_list {
        for sched in &["mq-deadline", "none"] {
            let filename = format!("randwrite-{}-{}-{}.txt", device_name, nj, sched);
            if let Ok(file) = File::open(&filename) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let (Ok(latency), Ok(iops)) = (
                                parts[0].parse::<f64>(),
                                parts[1].parse::<f64>()
                            ) {
                                data.push((nj, sched.to_string(), latency, iops));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Plot 1: Latency vs Number of Jobs
    let output_file = format!("randwrite-{}-latency.png", device_name);
    let root = BitMapBackend::new(&output_file, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_latency = data.iter()
        .map(|(_, _, lat, _)| lat)
        .fold(0.0f64, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Rondom Write Latency ({})", device_name),
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..70f64, 0f64..max_latency * 1.1)?;

    chart
        .configure_mesh()
        .x_desc("Number of Jobs")
        .y_desc("Latency (usec)")
        .draw()?;

    // Separate data by scheduler
    let deadline_data: Vec<(f64, f64)> = data.iter()
        .filter(|(_, sched, _, _)| sched == "mq-deadline")
        .map(|(nj, _, lat, _)| (*nj as f64, *lat))
        .collect();

    let none_data: Vec<(f64, f64)> = data.iter()
        .filter(|(_, sched, _, _)| sched == "none")
        .map(|(nj, _, lat, _)| (*nj as f64, *lat))
        .collect();

    // Plot scatter points for mq-deadline
    chart.draw_series(
        deadline_data.iter().map(|(nj, lat)| {
            Circle::new((*nj, *lat), 3, BLUE.filled())
        })
    )?
    .label("mq-deadline")
    .legend(|(x, y)| Circle::new((x, y), 3, BLUE.filled()));

    // Plot scatter points gor none
    chart.draw_series(
        none_data.iter().map(|(nj, lat)| {
            Circle::new((*nj, *lat), 3, RED.filled())
        })
    )?
    .label("none")
    .legend(|(x, y)| Circle::new((x, y), 3, RED.filled()));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Generated: {}", output_file);

    // Plot 2: IOPS vs Number of Jobs
    let output_file = format!("randwrite-{}-iops.png", device_name);
    let root = BitMapBackend::new(&output_file, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_iops = data.iter()
        .map(|(_, _, _, iops)| iops)
        .fold(0.0f64, |a, &b| a.max(b));

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Random Write IOPS ({})", device_name),
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..70f64, 0f64..max_iops * 1.1)?;

    chart
        .configure_mesh()
        .x_desc("Number of Jobs")
        .y_desc("IOPS")
        .draw()?;

    // Separate IOPS data by scheduler
    let deadline_iops: Vec<(f64, f64)> = data.iter()
        .filter(|(_, sched, _, _)| sched == "mq-deadline")
        .map(|(nj, _, _, iops)| (*nj as f64, *iops))
        .collect();

    let none_iops: Vec<(f64, f64)> = data.iter()
        .filter(|(_, sched, _, _)| sched == "none")
        .map(|(nj, _, _, iops)| (*nj as f64, *iops))
        .collect();

    // Plot scatter points for mq-deadline
    chart.draw_series(
        deadline_iops.iter().map(|(nj, iops)| {
            Circle::new((*nj, *iops), 3, BLUE.filled())
        })
    )?
    .label("mq-deadline")
    .legend(|(x, y)| Circle::new((x, y), 3, BLUE.filled()));

    // Plot scatter points for none
    chart.draw_series(
        none_iops.iter().map(|(nj, iops)| {
            Circle::new((*nj, *iops), 3, RED.filled())
        })
    )?
    .label("none")
    .legend(|(x, y)| Circle::new((x, y), 3, RED.filled()));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Generated: {}", output_file);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Block Device I/O Benchmark ===\n");

    // Load configuration
    let config = load_config()?;
    println!("Device: {}", config.device);
    println!("Device Name: {}", config.device_name);

    // Validate device
    validate_device(&config.device)?;
    println!("Device validated successfully\n");

    // Save original setting
    let original_scheduler = get_scheduler(&config.device)?;
    let original_ra = get_read_ahead(&config.device)?;
    println!("Original scheduler: {}", original_scheduler);
    println!("Original read-ahead: {}", original_ra);

    // Run benchmark
    benchmark_read(&config.device, &config.device_name)?;
    benchmark_randwrite(&config.device, &config.device_name)?;

    // restore original setting
    println!("\n=== Reatoring Original Settings ===\n");
    set_scheduler(&config.device, &original_scheduler)?;
    set_read_ahead(&config.device, original_ra)?;

    // Generate graphs
    process_read_data(&config.device_name)?;
    process_randwrite_data(&config.device_name)?;

    Ok(())
}
