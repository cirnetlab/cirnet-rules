`timescale 1ns/1ps
module tb;
  reg clk = 0;
  reg reset_n = 0;
  reg request = 0;
  reg permit = 0;
  reg watchdog_kick = 0;
  reg fault = 0;
  wire active;

  command_gate dut (
    .clk(clk), .reset_n(reset_n), .request(request), .permit(permit),
    .watchdog_kick(watchdog_kick), .fault(fault), .active(active)
  );
  always #5 clk = ~clk;

  task expect_active(input string phase);
    if (active !== 1'b1) $fatal(1, "%s: expected an active command", phase);
  endtask

  task expect_inactive(input string phase);
    if (active !== 1'b0) $fatal(1, "%s: output did not return to safe state", phase);
  endtask

  task wait_for_active;
    integer elapsed;
    begin
      for (elapsed = 0; elapsed < 6 && active !== 1'b1; elapsed = elapsed + 1) begin
        @(posedge clk);
        #1;
      end
      expect_active("startup timeout");
    end
  endtask

  `include "sequence.svh"

  initial begin
    repeat (2) @(negedge clk);
    expect_inactive("initial reset");
    reset_n = 1;
    #1;
    expect_inactive("before permit");
    run_sequence();
    expect_active("startup");

    @(negedge clk);
    permit = 0;
    @(posedge clk);
    #1;
    expect_inactive("permit loss");
    $display("permit loss PASS");

    @(negedge clk);
    permit = 1;
    request = 1;
    watchdog_kick = 1;
    @(posedge clk);
    #1;
    expect_active("before watchdog expiry");
    @(negedge clk);
    watchdog_kick = 0;
    repeat (4) @(posedge clk);
    #1;
    expect_inactive("watchdog expiry");
    $display("watchdog expiry PASS");

    @(negedge clk);
    watchdog_kick = 1;
    @(posedge clk);
    #1;
    expect_active("before fault");
    @(negedge clk);
    fault = 1;
    @(posedge clk);
    #1;
    expect_inactive("fault");
    $display("fault PASS");

    @(negedge clk);
    fault = 0;
    @(posedge clk);
    #1;
    expect_active("before reset");
    @(negedge clk);
    reset_n = 0;
    @(posedge clk);
    #1;
    expect_inactive("reset");
    $display("reset PASS");
    $display("simulation PASS");
    $finish;
  end

  initial begin
    #1000;
    $fatal(1, "global testbench deadline");
  end
endmodule
