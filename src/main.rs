pub mod vcdparser;

use clap::Parser;
use vcdparser::*;
use std::{cmp::min, io::Read, collections::HashMap};
use gag::BufferRedirect;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    /// Path to the first VCD file
    #[arg(long)]
    vcd1: std::path::PathBuf,

    /// Path to the second VCD file
    #[arg(long)]
    vcd2: std::path::PathBuf,

    /// Optional scope to restrict comparison (e.g., top.module1)
    #[arg(long)]
    scope: Option<String>,

    /// Top level clock signal
    #[arg(long)]
    clock: String,

    /// Top level reset signal
    #[arg(long)]
    reset: String,

    #[arg(long)]
    print_hier: bool,
}

fn get_vcd(opts: &Opts) -> (WaveformDB, WaveformDB) {
    let vcd1 = WaveformDB::new(&opts.vcd1.to_string_lossy().to_string());
    let vcd2 = WaveformDB::new(&opts.vcd2.to_string_lossy().to_string());
    (vcd1, vcd2)
}

fn main() {
    let opts = Opts::parse();

    println!("{:#?}", opts);
    let (mut vcd1, mut vcd2) = {
        let mut buf = BufferRedirect::stdout().unwrap();

        let ret = get_vcd(&opts);

        // Redirect stdout while parsing VCD
        let mut out = String::new();
        buf.read_to_string(&mut out).unwrap();

        ret
    };

    if opts.print_hier {
        vcd1.print_hierarchy();
    }

    // Pre-load signals for both waveforms
    println!("Pre-loading signals from VCD files...");
    vcd1.preload_signals();
    vcd2.preload_signals();

    let time1 = vcd1.clock_cycles(&opts.clock);
    let time2 = vcd2.clock_cycles(&opts.clock);

    let total_cycles = min(time1.tot_cycles, time2.tot_cycles);

    println!("time1 {:?}", time1);
    println!("time2 {:?}", time2);
    println!("total_cycle {:?}", total_cycles);

    // Setup progress bar
    let pb = ProgressBar::new(total_cycles);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} cycles ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    // Track first divergence point for each signal
    let mut first_divergence: HashMap<WaveformSignal, (u64, FourStateBit, FourStateBit)> = HashMap::new();
    let scope_prefix = opts.scope.as_deref().unwrap_or("");

    // Compare signals and report differences
    for cycle in 0..total_cycles {
        pb.inc(1);
        let step1 = time1.offset + cycle * time1.per_cycle_steps;
        let step2 = time2.offset + cycle * time2.per_cycle_steps;

        let signals1 = vcd1.signal_values_at_cycle(step1 as u32);
        let signals2 = vcd2.signal_values_at_cycle(step2 as u32);

        for (signal1, value1) in signals1.iter() {
            let signal_path = signal1.to_string();
            if !signal_path.starts_with(scope_prefix) {
                continue;
            }

            match signals2.get(signal1) {
                Some(value2) => {
                    if value1 != value2 && !first_divergence.contains_key(signal1) {
                        first_divergence.insert(
                            signal1.clone(),
                            (cycle, value1.clone(), value2.clone())
                        );
                    }
                }
                None => {
                    if !first_divergence.contains_key(signal1) {
                        println!("Signal '{}' only exists in first file", signal_path);
                        first_divergence.insert(
                            signal1.clone(),
                            (cycle, value1.clone(), FourStateBit::X)
                        );
                    }
                }
            }
        }

        if first_divergence.len() > 10 {
            break;
        }
    }
    pb.finish();

    // Report all divergences
    if first_divergence.is_empty() {
        println!("Success, no differences found!");
    } else {
        println!("\nFound differences in {} signals:", first_divergence.len());
        for (signal, (cycle, val1, val2)) in first_divergence.iter() {
            println!("Signal '{}' first diverged at cycle {}: {:?} vs {:?}", 
                signal.to_string(), cycle, val1, val2);
        }
        std::process::exit(1);
    }
}
