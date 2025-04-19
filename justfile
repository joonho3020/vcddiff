test_inputs_dir := "test-data"

[group: 'run']
run_example: run_rtl_sim
  cargo run --release -- \
    --vcd1 {{test_inputs_dir}}/gcd.vcd \
    --vcd2 {{test_inputs_dir}}/gcd_buggy.vcd \
    --clock "gcd_tb.clk" \
    --reset "gcd_tb.reset"

[group: 'run']
run_digitaltop:
  cargo run --release -- \
    --vcd1 test-data/hello.golden.vcd \
    --vcd2 test-data/hello.impl.vcd \
    --clock TestDriver.testHarness.chiptop0.system.auto_chipyard_prcictrl_domain_reset_setter_clock_in_member_allClocks_uncore_clock \
    --reset TestDriver.testHarness.chiptop0.system.auto_chipyard_prcictrl_domain_reset_setter_clock_in_member_allClocks_uncore_reset \
    --scope TestDriver.testHarness.chiptop0.system

[group: 'run']
run_rtl_sim:
  cd {{test_inputs_dir}} && make

[group: 'clean']
clean:
  cd {{test_inputs_dir}} && make clean

[group: 'clean']
clean_build:
  cargo clean

[group: 'clean']
clean_all: clean clean_build

[group: 'profile']
flamegraph:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin vcddiff -- \
        --vcd1 test-data/hello.golden.vcd \
        --vcd2 test-data/hello.impl.vcd \
        --clock TestDriver.testHarness.chiptop0.system.auto_chipyard_prcictrl_domain_reset_setter_clock_in_member_allClocks_uncore_clock \
        --reset TestDriver.testHarness.chiptop0.system.auto_chipyard_prcictrl_domain_reset_setter_clock_in_member_allClocks_uncore_reset \
        --scope TestDriver.testHarness.chiptop0.system