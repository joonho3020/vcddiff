`timescale 1ns / 1ps

module gcd_tb;
    reg clk;
    reg reset;
    reg start;
    reg [31:0] a;
    reg [31:0] b;
    wire [31:0] result;
    wire done;

    // Instantiate GCD module
    gcd uut (
        .clk(clk),
        .reset(reset),
        .start(start),
        .a(a),
        .b(b),
        .result(result),
        .done(done)
    );

    // Clock generation
    initial begin
        clk = 0;
        forever #5 clk = ~clk; // 100 MHz clock
    end

    // Test procedure
    initial begin
        // Initialize VCD dump
        $dumpfile("gcd.vcd");
        $dumpvars(0, gcd_tb);

        // Initialize signals
        reset = 1;
        start = 0;
        a = 0;
        b = 0;
        #20;
        reset = 0;

        // Test case 1: GCD(48, 18) = 6
        a = 48;
        b = 18;
        start = 1;
        #10;
        start = 0;
        wait (done);
        #10;
        if (result != 6) $display("Test 1 failed: GCD(48, 18) = %d, expected 6", result);
        else $display("Test 1 passed");

        // Test case 2: GCD(7, 13) = 1
        #20;
        a = 7;
        b = 13;
        start = 1;
        #10;
        start = 0;
        wait (done);
        #10;
        if (result != 1) $display("Test 2 failed: GCD(7, 13) = %d, expected 1", result);
        else $display("Test 2 passed");

        // Test case 3: GCD(0, 5) = 5
        #20;
        a = 0;
        b = 5;
        start = 1;
        #10;
        start = 0;
        wait (done);
        #10;
        if (result != 5) $display("Test 3 failed: GCD(0, 5) = %d, expected 5", result);
        else $display("Test 3 passed");

        // Test case 4: GCD(0, 0) = 0
        #20;
        a = 0;
        b = 0;
        start = 1;
        #10;
        start = 0;
        wait (done);
        #10;
        if (result != 0) $display("Test 4 failed: GCD(0, 0) = %d, expected 0", result);
        else $display("Test 4 passed");

        // End simulation
        #20;
        $finish;
    end
endmodule
