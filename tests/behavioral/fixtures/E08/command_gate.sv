`timescale 1ns/1ps
module command_gate (
  input wire clk, reset_n, request, permit, watchdog_kick, fault,
  output wire active
);
  reg [2:0] age;

  always @(posedge clk) begin
    if (!reset_n || watchdog_kick)
      age <= 0;
    else if (age < 4)
      age <= age + 1'b1;
  end

  assign active = reset_n && request && permit && !fault && age < 4;
endmodule
