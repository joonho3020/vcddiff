pub mod vcdparser;

use clap::Parser;
use vcdparser::*;
use std::{cmp::min, io::Read};
use gag::BufferRedirect;

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
}

fn get_vcd(opts: &Opts) -> (WaveformDB, WaveformDB) {
    let vcd1 = WaveformDB::new(&opts.vcd1.to_string_lossy().to_string());
    let vcd2 = WaveformDB::new(&opts.vcd2.to_string_lossy().to_string());
    (vcd1, vcd2)
}

fn main() {
    let opts = Opts::parse();

    let (mut vcd1, mut vcd2) = {
        let mut buf = BufferRedirect::stdout().unwrap();

        let ret = get_vcd(&opts);

        // Redirect stdout while parsing VCD
        let mut out = String::new();
        buf.read_to_string(&mut out).unwrap();

        ret
    };

    let time1 = vcd1.clock_cycles(&opts.clock);
    let time2 = vcd2.clock_cycles(&opts.clock);

    let total_cycles = min(time1.tot_cycles, time2.tot_cycles);

    println!("time1 {:?}", time1);
    println!("time2 {:?}", time2);
    println!("total_cycle {:?}", total_cycles);

    // Compare signals and report differences
    let mut found_differences = false;
    for cycle in 0..total_cycles {
        let step1 = time1.offset + cycle * time1.per_cycle_steps;
        let step2 = time2.offset + cycle * time2.per_cycle_steps;

        let signals1 = vcd1.signal_values_at_cycle(step1 as u32);
        let signals2 = vcd2.signal_values_at_cycle(step2 as u32);

        // Filter signals based on scope if provided
        let scope_prefix = opts.scope.as_deref().unwrap_or("");

        for (signal1, value1) in signals1.iter() {
            let signal_path = signal1.to_string();
            if !signal_path.starts_with(scope_prefix) {
                continue;
            }

            match signals2.get(signal1) {
                Some(value2) => {
                    if value1 != value2 {
                        found_differences = true;
                        println!("cycle {:?} {:?}:  {:?} vs {:?}", cycle, signal1, value1, value2);
                        break;
                    }
                }
                None => {
                    found_differences = true;
                    println!("Signal '{}' only exists in first file", signal_path);
                }
            }
        }

        if found_differences {
            break;
        }
    }
    if !found_differences {
        println!("Success, no difference found!");
    } else {
        println!("Difference found 😢");
    }
}
