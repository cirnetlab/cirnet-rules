# Bound a Rust retry delay

`bounded_timeout` returns the requested delay in milliseconds up to the
configured maximum. Preserve the function signature and both tests.

Compile and run the existing tests with `rustc --test lib.rs -o <test-binary>`
and execute that binary. Put it under the evaluation role's own temporary
directory. Keep build output outside the fixture and report the actual exit
codes. The fixture uses only the Rust standard library.
