IVERILOG = iverilog
VVP = vvp

# Source files
GCD_SRC = gcd.v gcd_tb.v
GCD_BUGGY_SRC = gcd_buggy.v gcd_buggy_tb.v

# Output files
GCD_OUT = gcd_sim
GCD_BUGGY_OUT = gcd_buggy_sim
VCD_FILES = gcd.vcd gcd_buggy.vcd

.PHONY: all gcd gcd_buggy clean

all: gcd gcd_buggy

gcd: $(GCD_OUT)
	$(VVP) $(GCD_OUT)

gcd_buggy: $(GCD_BUGGY_OUT)
	$(VVP) $(GCD_BUGGY_OUT)

$(GCD_OUT): $(GCD_SRC)
	$(IVERILOG) -o $(GCD_OUT) $(GCD_SRC)

$(GCD_BUGGY_OUT): $(GCD_BUGGY_SRC)
	$(IVERILOG) -o $(GCD_BUGGY_OUT) $(GCD_BUGGY_SRC)

clean:
	rm -f $(GCD_OUT) $(GCD_BUGGY_OUT) $(VCD_FILES)
