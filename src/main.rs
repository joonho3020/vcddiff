use wellen::{self, Waveform};
use std::path::Path;
use structopt::StructOpt;
use std::collections::HashMap;

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
}
