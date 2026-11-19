# Synthetic sequencer

This simulation models a generic command gate. It contains no board mapping,
electrical limits or device programming steps.

Run `node simulate.mjs` with `iverilog` and `vvp` on PATH. The script compiles
the RTL and bench into a fresh temporary directory and returns the simulator's
exit status. `sequence.svh` supplies the feature stimulus; `tb.sv` drives
the negative safety tests.
