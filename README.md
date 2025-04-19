# VCDDiff - VCD Waveform Comparison Tool

VCDDiff is a high-performance tool for comparing VCD (Value Change Dump) waveforms, designed specifically for RTL development and verification workflows. It helps identify differences between a reference implementation and a test implementation by comparing signal values across clock cycles.

## Features

- Fast, cycle-accurate comparison of VCD waveforms
- Scope-based filtering to focus on specific modules
- Efficient signal caching for improved performance
- Progress bar visualization for long-running comparisons
- Detailed reporting of first divergence points for each signal
- Support for multi-bit signals and various signal types (binary, hex)

## Use Cases

VCDDiff is particularly useful when:

1. Developing RTL Compilers:
   - Compare compiler output against a reference implementation
   - Verify correctness of compiler optimizations
   - Debug transformation passes

2. RTL Simulation:
   - Validate simulator implementations against reference simulators
   - Verify timing consistency across different simulation tools
   - Debug race conditions and timing issues

3. Hardware Verification:
   - Compare pre-synthesis vs post-synthesis behavior
   - Verify equivalence after RTL modifications
   - Debug signal mismatches in different implementation stages

## Installation

```bash
# Install dependencies
cargo install flamegraph  # Required for performance profiling

# Build the project
cargo build --release

# Run the tool
cargo run --release -- \
    --vcd1 path/to/reference.vcd \
    --vcd2 path/to/test.vcd \
    --clock "top.clk" \
    --reset "top.reset"

# Print hierarchy
cargo run --release -- \
    --vcd1 path/to/reference.vcd \
    --vcd2 path/to/test.vcd \
    --clock "top.clk" \
    --reset "top.reset" \
    --print-hier
```

## Example output

```
Found differences in 3 signals:
Signal 'top.module1.signal_a' first diverged at cycle 42: 0x1 vs 0x0
Signal 'top.module1.signal_b' first diverged at cycle 57: 0x3F vs 0x2F
Signal 'top.module2.flag' first diverged at cycle 103: 1 vs 0
```

## Flamegraph profiling

```
just flamegraph
```