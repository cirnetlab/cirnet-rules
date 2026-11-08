<!-- doc-id: TOOLS -->
<!-- revision: 2 -->
<a id="commands"></a>

# Rust governance tooling

Use Rust and Cargo 1.99.1 and Git. A WORK task requires only its selected
checkout bindings. Build once and run the binary for subsequent steps.
Exit 0 means PASS, 1 means a policy or input failure, and 2 means blocked or unrun.

~~~sh
cargo build --locked
./target/debug/cirnet-governance workspace-discover
./target/debug/cirnet-governance validate
./target/debug/cirnet-governance check-translations --paths en/routing/README.md
~~~

On Windows the binary has an exe suffix. The Cargo targets are src/main.rs
and src/tests/cli.rs. Schemas stay under en/schemas. See the [local workflow](../references/cgacp-local.md)
for task creation, execution, acceptance and the explicitly selected release mode.
The old context, doctor, version and drift commands remain available for
legacy audits. Use scoped health and task context for ordinary work.
