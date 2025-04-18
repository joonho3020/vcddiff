module gcd_buggy (
    input wire clk,
    input wire reset,
    input wire start,
    input wire [31:0] a,
    input wire [31:0] b,
    output reg [31:0] result,
    output reg done
);
    reg [31:0] a_reg, b_reg;
    reg working;

    always @(posedge clk or posedge reset) begin
        if (reset) begin
            a_reg <= 0;
            b_reg <= 0;
            result <= 0;
            done <= 0;
            working <= 0;
        end else begin
            if (start && !working) begin
                a_reg <= a;
                b_reg <= b;
                working <= 1;
                done <= 0;
            end else if (working) begin
                if (a_reg == 0) begin
                    result <= b_reg;
                    done <= 1;
                    working <= 0;
                end else if (b_reg == 0) begin
                    result <= a_reg + 1; // Bug: Incorrectly adds 1
                    done <= 1;
                    working <= 0;
                end else begin
                    if (a_reg >= b_reg)
                        a_reg <= a_reg - b_reg;
                    else
                        b_reg <= b_reg - a_reg;
                end
            end
        end
    end
endmodule
