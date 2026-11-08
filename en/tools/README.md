<!-- doc-id: TOOLS -->
<!-- revision: 1 -->
<a id="commands"></a>

# Rust governance tooling

Use Rust and Cargo 1.99.1, Git and a product workspace containing six readable
sibling checkouts. The Rust language MSRV in Cargo metadata is separate from
the exact toolchain. The commands return JSON. Exit 0 means the requested
check passed, 1 means a policy or input failure, and 2 means blocked or unrun.

~~~sh
cargo run --locked -- validate
cargo run --locked -- check-translations
cargo run --locked -- doctor --workspace ..
cargo run --locked -- context --workspace .. --repo cirnet-remote --task audit
cargo run --locked -- check-versions --workspace ..
cargo run --locked -- check-drift --workspace ..
cargo run --locked -- check-release --workspace ..
~~~

Run from the governance checkout or pass its path with the governance option.
Read-only product commands do not fetch or build products. Context records
the actual commit and dirty state, and hashes normalized English bytes.
Set Cargo's target and cache locations externally for isolated runs. See
[adapters](../adapters/README.md) and [release schema](../schemas/README.md).
