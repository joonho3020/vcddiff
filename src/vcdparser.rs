use indexmap::IndexMap;
use indicatif::ProgressStyle;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use wellen::*;
use itertools::Itertools;

pub type Bit = u8;

pub type SignalMap = IndexMap<WaveformSignal, FourStateBit>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TimeStampInfo {
    pub tot_cycles: wellen::Time,
    pub per_cycle_steps: wellen::Time,
    pub offset: wellen::Time
}

impl TimeStampInfo {
    pub fn new(tot_cycles: wellen::Time, per_cycle_steps: wellen::Time, offset: wellen::Time) -> Self {
        Self { tot_cycles, per_cycle_steps, offset }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum FourStateBit {
    #[default]
    ZERO,
    ONE,
    X,
    Z,
    MultiHex(String),  // For multi-bit hex values (0s and 1s)
    MultiX(String),    // For multi-bit X values
    MultiZ(String),    // For multi-bit Z values
}

impl FourStateBit {
    pub fn from_char(c: char) -> Self {
        match c {
            '0' => Self::ZERO,
            '1' => Self::ONE,
            'x' => Self::X,
            'z' => Self::Z,
            _ => Self::X,
        }
    }

    fn bin_to_hex(bin: &str) -> String {
        // Pad the binary string to make its length a multiple of 4
        let padding = (4 - (bin.len() % 4)) % 4;
        let padded_bin = format!("{:0>width$}", bin, width = bin.len() + padding);

        // Convert each group of 4 bits to a hex digit
        let hex_value: String = padded_bin.chars()
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|chunk| {
                let bin_digit: String = chunk.iter().rev().collect();
                let decimal = u8::from_str_radix(&bin_digit, 2).unwrap_or(0);
                format!("{:X}", decimal)
            })
            .collect();

        format!("0x{}", hex_value)
    }

    pub fn from_string(s: String) -> Self {
        if s.len() == 1 {
            return Self::from_char(s.chars().next().unwrap());
        }

        // Check if the string contains only specific characters
        let contains_only = |s: &str, chars: &[char]| -> bool {
            s.chars().all(|c| chars.contains(&c))
        };

        if contains_only(&s, &['0', '1']) {
            Self::MultiHex(Self::bin_to_hex(&s))
        } else if contains_only(&s, &['x', 'X']) {
            Self::MultiX(s)
        } else if contains_only(&s, &['z', 'Z']) {
            Self::MultiZ(s)
        } else {
            // If mixed, prioritize in order: X > Z > Hex
            if s.chars().any(|c| c == 'x' || c == 'X') {
                Self::MultiX(s)
            } else if s.chars().any(|c| c == 'z' || c == 'Z') {
                Self::MultiZ(s)
            } else {
                Self::MultiHex(Self::bin_to_hex(&s))
            }
        }
    }

    pub fn to_bit(self: &Self) -> Option<Bit> {
        match self {
            Self::ZERO => Some(0),
            Self::ONE => Some(1),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct WaveformSignal {
    path: Vec<String>,
}

impl WaveformSignal {
    pub fn new(path_: Vec<String>) -> Self {
        Self {
            path: path_
        }
    }

    pub fn hier(self: &Self) -> Vec<String> {
        let len = self.path.len();
        return self.path[..len-1].to_vec();
    }

    pub fn name(self: &Self) -> String {
        assert!(self.path.len() > 0, "WaveformSignal path is empty");
        return self.path.last().unwrap().to_string();
    }

    pub fn append(self: &mut Self, sig: String) {
        self.path.push(sig);
    }

    pub fn to_string(self: &Self) -> String {
        return self.path.join(".");
    }
}

impl From<String> for WaveformSignal {
    fn from(value: String) -> Self {
        let path_: Vec<String> = value.split('.').map(|s| s.to_string()).collect();
        Self {
            path: path_
        }
    }
}

const LOAD_OPTS: LoadOptions = LoadOptions {
    multi_thread: true,
    remove_scopes_with_empty_name: false,
};

pub struct WaveformDB {
    pub header: viewers::HeaderResult,
    pub body: viewers::BodyResult,
}

impl WaveformDB {
    pub fn new(vcd_file: &String) -> WaveformDB {
        let header = viewers::read_header(&vcd_file, &LOAD_OPTS).expect("Failed to load file!");
        let hierarchy = header.hierarchy;
        let body = header.body;

        // create body progress indicator
        let body_len = header.body_len;
        let (body_progress, progress) = if body_len == 0 {
            (None, None)
        } else {
            let p = Arc::new(AtomicU64::new(0));
            let p_out = p.clone();
            let done = Arc::new(AtomicBool::new(false));
            let done_out = done.clone();
            let ten_millis = std::time::Duration::from_millis(10);
            let t = thread::spawn(move || {
                let bar = indicatif::ProgressBar::new(body_len);
                bar.set_style(
                    ProgressStyle::with_template(
                        "[{elapsed_precise}] {bar:40.cyan/blue} {decimal_bytes} ({percent_precise}%)",
                    )
                    .unwrap(),
                );
                loop {
                    // always update
                    let new_value = p.load(Ordering::SeqCst);
                    bar.set_position(new_value);
                    thread::sleep(ten_millis);
                    // see if we are done
                    let now_done = done.load(Ordering::SeqCst);
                    if now_done {
                        bar.finish_and_clear();
                        break;
                    }
                }
            });

            (Some(p_out), Some((done_out, t)))
        };

        let body_ =
            viewers::read_body(body, &hierarchy, body_progress).expect("Failed to load body!");
        if let Some((done, t)) = progress {
            done.store(true, Ordering::SeqCst);
            t.join().unwrap();
        }

        // This is kind of stupid:
        // a way to get around the fact that body cannot be read w/o moving values out from the
        // "header", and read_body doesn't take borrowed types.
        let header2 = viewers::read_header(&vcd_file, &LOAD_OPTS).expect("Failed to load file!");

        return WaveformDB {
            header: header2,
            body: body_,
        };
    }

    /// Returns a signal name to bit value map for all signals at `cycle`
    pub fn signal_values_at_cycle(self: &mut Self, cycle: u32) -> SignalMap {
        let mut ret: SignalMap = SignalMap::new();

        let hierarchy = &self.header.hierarchy;
        for var in hierarchy.iter_vars() {
            let _signal_name: String = var.full_name(&hierarchy);
            let ids = [var.signal_ref(); 1];
            let loaded = self
                .body
                .source
                .load_signals(&ids, &hierarchy, LOAD_OPTS.multi_thread);
            let (_, loaded_signal) = loaded.into_iter().next().unwrap();

            let offset = loaded_signal.get_offset(cycle as u32);
            match offset {
                Some(idx) => {
                    for elemidx in 0..idx.elements {
                        let signal_path: Vec<String> = _signal_name.split('.').map(|s| s.to_string()).collect();
                        let sig_val = loaded_signal.get_value_at(&idx, elemidx);
                        let numbits = match sig_val.bits() {
                            Some(x) => x,
                            _ => {
                                continue;
                            },
                        };
                        let bits = match sig_val.to_bit_string() {
                            Some(bits_as_string) => bits_as_string,
                            _ => "".to_string(),
                        };
                        let bits_array: Vec<char> = bits.chars().rev().collect();
                        assert!(numbits == bits_array.len() as u32);

                        // Store the entire signal value as one entry
                        let val = if numbits == 1 {
                            FourStateBit::from_char(bits_array[0])
                        } else {
                            FourStateBit::from_string(bits.chars().rev().collect())
                        };
                        ret.insert(WaveformSignal::new(signal_path), val);
                    }
                }
                _ => {}
            }
        }
        return ret;
    }

    pub fn signal_values_at_cycle_rebase_top(self: &mut Self, cycle: u32, instance_path: String) -> IndexMap<String, FourStateBit> {
        let ref_signals = self.signal_values_at_cycle(cycle);
        let instance_depth = instance_path.split(".").collect_vec().len();

        let mut ret: IndexMap<String, FourStateBit> = IndexMap::new();
        for (signal_path, four_state_bit) in ref_signals.iter() {
            let name = signal_path.name();
            let mut hier = signal_path.hier();

            if hier.len() >= instance_depth {
                let hier_depth = &hier[..instance_depth];
                let hier_str = hier_depth.join(".");
                if hier_str == instance_path {
                    hier.drain(0..instance_depth);
                    hier.push(name.clone());
                } else {
                    continue;
                }
            }
            ret.insert(hier.join("."), four_state_bit.clone());
        }
        return ret;
    }

    pub fn print_hierarchy(&self) {
        let hierarchy = &self.header.hierarchy;
        for var in hierarchy.iter_vars() {
            let signal_name: String = var.full_name(&hierarchy);
            println!("signal_name {:?}", signal_name);
        }
    }

    /// Returns the number of clock cycles in the waveform by counting rising edges of the clock signal
    /// The clock_path should be the full hierarchical path to the clock signal (e.g., "top.clk" or "gcd_tb.clk")
    pub fn clock_cycles(&mut self, clock_path: &str) -> TimeStampInfo {
        let hierarchy = &self.header.hierarchy;

        // Find the clock signal
        for var in hierarchy.iter_vars() {
            let signal_name = var.full_name(&hierarchy);
            if signal_name == clock_path {
                let ids = [var.signal_ref(); 1];
                let loaded = self
                    .body
                    .source
                    .load_signals(&ids, &hierarchy, LOAD_OPTS.multi_thread);

                if let Some((_, loaded_signal)) = loaded.into_iter().next() {
                    let mut cycles = 0;
                    let mut last_value = FourStateBit::X;
                    let mut current_offset = 0;
                    let mut per_cycle_steps = 0;
                    let mut last_posedge = 0;
                    let mut first_posedge = 0;
                    let mut found_first_posedge = false;

                    // Count rising edges (transitions from 0 to 1)
                    while let Some(idx) = loaded_signal.get_offset(current_offset) {
                        if let Some(sig_val) = loaded_signal.get_value_at(&idx, 0).to_bit_string() {
                            let current_value = FourStateBit::from_char(sig_val.chars().next().unwrap_or('x'));

                            // Check for rising edge (0 to 1 transition)
                            if last_value == FourStateBit::ZERO && current_value == FourStateBit::ONE {
                                cycles += 1;
                                per_cycle_steps = current_offset as wellen::Time - last_posedge;
                                last_posedge = current_offset as wellen::Time;

                                if !found_first_posedge {
                                    first_posedge = current_offset as wellen::Time;
                                    found_first_posedge = true;
                                }
                            }
                            last_value = current_value;
                        }
                        current_offset += 1;
                        if idx.next_index.is_none() {
                            break;
                        }
                    }
                    return TimeStampInfo::new(cycles, per_cycle_steps, first_posedge);
                }
            }
        }
        TimeStampInfo::default()
    }
}
