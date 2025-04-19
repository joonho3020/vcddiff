pub mod vcdparser;

use structopt::StructOpt;
use vcdparser::*;

#[derive(StructOpt)]
struct Opts {
    /// Path to the first VCD file
    #[structopt(parse(from_os_str))]
    vcd1: std::path::PathBuf,

    /// Path to the second VCD file
    #[structopt(parse(from_os_str))]
    vcd2: std::path::PathBuf,

    /// Optional scope to restrict comparison (e.g., top.module1)
    #[structopt(long)]
    scope: Option<String>,
}

fn main() {
    let opts = Opts::from_args();

    // Load VCD files using the wrapper
    let mut vcd1 = WaveformDB::new(&opts.vcd1.to_string_lossy().to_string());
    let mut vcd2 = WaveformDB::new(&opts.vcd2.to_string_lossy().to_string());

    // Get the signal values at cycle 0 (or you could iterate through cycles if needed)
    let signals1 = vcd1.signal_values_at_cycle(0);
    let signals2 = vcd2.signal_values_at_cycle(0);

    // Filter signals based on scope if provided
    let scope_prefix = opts.scope.as_deref().unwrap_or("");

    // Compare signals and report differences
    let mut found_differences = false;

    for (signal1, value1) in signals1.iter() {
        let signal_path = signal1.to_string();
        if !signal_path.starts_with(scope_prefix) {
            continue;
        }

        match signals2.get(signal1) {
            Some(value2) => {
                if value1 != value2 {
                    found_differences = true;
                    println!("Signal '{}' differs:", signal_path);
                    println!("  File 1: {:?}", value1);
                    println!("  File 2: {:?}", value2);
                }
            }
            None => {
                found_differences = true;
                println!("Signal '{}' only exists in first file", signal_path);
            }
        }
    }

    // Check for signals that only exist in the second file
    for (signal2, _) in signals2.iter() {
        let signal_path = signal2.to_string();
        if !signal_path.starts_with(scope_prefix) {
            continue;
        }
        if !signals1.contains_key(signal2) {
            found_differences = true;
            println!("Signal '{}' only exists in second file", signal_path);
        }
    }

    if !found_differences {
        println!("No differences found between the VCD files");
        if !scope_prefix.is_empty() {
            println!("(within scope '{}')", scope_prefix);
        }
    }
}
