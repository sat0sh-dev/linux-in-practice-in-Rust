use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;
use std::hint::black_box;
use plotters::prelude::*;

const CACHE_LINE_SIZE: usize = 64;
const NACCESS: usize  = 128 * 1024 * 1024; // 128 MiB

fn main() {
    // Remove old output file
    let _ = std::fs::remove_file("out.txt");

    // Open output file
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("out.txt")
        .expect("Failed to open out.txt");

    let mut results: Vec<(f64, f64)> = Vec::new();

    // Test different buffer sizes from 2^2 to 2^16 KB (4KB to 64MB)
    let mut i = 2.0;
    while i <= 16.0 {
        let buf_size = (2.0_f64.powf(i) * 1024.0) as usize;

        // Allocate buffer using mmap (anonymous mapping)
        let data = unsafe {
            libc::mmap(
                std::ptr::null_mut(), 
                buf_size, 
                libc::PROT_READ | libc::PROT_WRITE, 
                libc::MAP_ANONYMOUS | libc::MAP_PRIVATE, 
                -1, 
                0
            )
        };

        if data == libc::MAP_FAILED {
            eprintln!("mmap() failed ");
            std::process::exit(1);
        }

        let ptr = data as *mut u8;

        println!("Collecting data for buffer size 2^{:.2}({}) KB", i, buf_size / 1024);

        let start = Instant::now();

        // Access memory: iterate through entire buffer in cache line steps
        let iterations = NACCESS / (buf_size / CACHE_LINE_SIZE);
        for _ in 0..iterations {
            for j in (0..buf_size).step_by(CACHE_LINE_SIZE) {
                unsafe {
                    *ptr.add(j) = 0;
                    black_box(*ptr.add(j)); // Prevent optimization
                }
            }
        }

        let elapsed = start.elapsed();
        let access_per_ns = NACCESS as f64 / elapsed.as_nanos() as f64;

        // Write result to file
        writeln!(file, "{}\t{}", i, access_per_ns)
            .expect("Failed to write to file");

        // Store for plotting
        results.push((i, access_per_ns));

        // CLeanup
        unsafe {
            libc::munmap(data, buf_size);
        }

        i += 0.25;
    }

    println!("\nData collection complete. Generatin graph...");
    if let Err(e) = plot_cache(&results) {
        eprintln!("Failed to generate graph: {}", e);
        std::process::exit(1);
    }

    println!("Graph generated successfully: cache.png")
}

fn plot_cache(data: &[(f64, f64)]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("cache.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find min/max for axes
    let min_x = data.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
    let max_x = data.iter().map(|(x, _)| *x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
    let max_y = data.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);

    // Add some padding
    let y_padding = (max_y - min_y) * 0.1;

    let mut chart = ChartBuilder::on(&root)
        .caption("Cache Memory Effect Visualization", ("sans-selif", 40).into_font())
        .margin(15)
        .x_label_area_size(50)
        .y_label_area_size(70)
        .build_cartesian_2d(
            min_x..max_x,
            (min_y - y_padding)..(max_y + y_padding)
        )?;

    chart.configure_mesh()
        .x_desc("Buffer Size [2^x KiB")
        .y_desc("Access Speef [accesses / nanosecond]")
        .x_label_style(("sans-serif", 15))
        .y_label_style(("sans-serif", 15))
        .draw()?;
    
    // Draw scatter plot
    chart.draw_series(
        data.iter().map(|(x, y)| Circle::new((*x, *y), 3, BLUE.filled()))
    )?
    .label("Access Speed")
    .legend(|(x, y)| Circle::new((x, y), 3, BLUE.filled()));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    
    Ok(())
}