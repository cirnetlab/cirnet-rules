task run_sequence;
  begin
    @(negedge clk);
    request = 1;
    permit = 1;
    watchdog_kick = 1;
    wait_for_active();
  end
endtask
