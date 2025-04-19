
test_inputs_dir := "test-data"

[group: 'run']
run_rtl_sim:
  cd {{test_inputs_dir}} && make

[group: 'run']
run_example: run_rtl_sim
  cargo run --release -- \
    --vcd1 {{test_inputs_dir}}/gcd.vcd \
    --vcd2 {{test_inputs_dir}}/gcd_buggy.vcd \
    --clock "clock" \
    --reset "reset"

[group: 'clean']
clean:
  cd {{test_inputs_dir}} && make clean

[group: 'clean']
clean_build:
  cargo clean

[group: 'clean']
clean_all: clean clean_build
