task run_sequence;
  begin
    @(negedge clk);
    request = 1;
    watchdog_kick = 1;
    wait_for_active();
    permit = 1;
  end
endtask
