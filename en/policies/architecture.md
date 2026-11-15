# Architecture compliance and Lab-First priority

## Select authority by task

Use the [repository routing table](../references/architecture-routing.md) to find
the owner and relevant architecture. Read the English sections that govern the
change and the owner's current instructions, contracts, artifacts and tests.
Use an approved checkout or a pinned artifact with verifiable provenance. An
installed agent bundle supplies navigation. Read architecture from that checkout
or artifact as needed.

| Scope | Authority |
| --- | --- |
| Organization boundaries, shared services and integration | [Ecosystem Architecture](../../docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-en.md), especially §§4–5 for ownership. |
| Virtual and Physical Lab, electrical safety and experiment execution | [Lab-First Architecture](../../docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-en.md). |
| Lab FPGA RTL, instruments, routing and board implementation | [FPGA Architecture](../../docs/cirnetlab-FPGA-architecture-en.md) within the Lab-First safety envelope. |
| Implementation, commands and tests | Applicable repository-local instructions, source and tests. Use the contract owner's published schema and the consumer's selected version. |

For a real conflict, cite both decisions with sections, describe the affected
boundary and ask the owners to review. Continue independent scoped work. Treat
target ownership as a design decision and verify the current writer, issuer,
selected release and evidence before changing a boundary.

Identify the task → select architecture → resolve the repository owner → inspect
contracts and artifacts → plan → implement → verify architecture compliance →
report actual results and limits.

## Development priority

`LAB_FIRST_PRIMARY` is the default. Prioritize `cirnet-interface`, `cirnet-remote`,
`cirnet-contracts`, `cirnet-hardware`, `cirnet-simulator` and `cirnet-fpga`.
First prove the real path Web → Rust Remote → Rust Hardware → C++ Simulator →
Web. Within the platform's production scope, use Rust for Backend, Contracts
runtime and Hardware, C++ for the default Simulator, TypeScript for Web and
Verilog/SystemVerilog for FPGA. Use Python for testing, benchmarks and isolated
historical verification.

- Integrate through owner-published contracts and exact artifact pins. Verify
  compatibility and provenance across the selected set. Keep the
  [VD-01–VD-10 gates in Lab-First §6.5](../../docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-en.md#65-acceptance-gates-virtual-digital-e2e)
  unchanged as the Virtual Digital E2E acceptance criteria.
- Keep Safety, Topology and Functional results distinct. Safety and executability
  gate physical operation. Bind endpoints to observed physical topology. The
  Physical Compiler routes approved infrastructure only, so the student wiring
  remains as assembled. Return `UNAVAILABLE` for unsupported capabilities.
  Never substitute fabricated readings or outcomes.
- Use [Lab-First §19 testing responsibilities](../../docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-en.md#19-verification-strategy-and-definition-of-done).
  Remote owns authorization, planning and operation lifecycle checks. Hardware
  and Simulator verify real worker Stop and negative paths. Remote, Hardware
  and FPGA verify observed bindings and infrastructure-only routing at their
  boundaries. Hardware and FPGA need board HIL and owner approval for physical
  operation. A static check covers only documentation or source properties.
  Find concrete tests in the [owner evidence](../docs/toolkit-verification.md#lab-first-product-evidence).

Use `PARALLEL_ALLOWED` for coordinated expansion when the owners can point to a
documented, stable Lab-First vertical slice with relevant contract compatibility,
the real execution chain, Stop/cancellation, negative tests and exact artifact
provenance. Record the evidenced scope and limits. This is a development gate.
Record each VD result separately against §6.5 and the selected artifacts.
Use `LAB_FIRST_PRIMARY` for the currently inspected product set.
Independent IDE, LMS, Studio, scientific or other Ecosystem work may proceed
within its own bounds while Lab-First remains primary, provided it keeps the
Lab path intact, respects existing owners and preserves one authority per domain.

## Canonical sources and specialist variants

The EN/VI Ecosystem and Lab-First pairs under `docs/tai-lieu-kien-tuc-he-thong/`
are canonical for this toolkit. See the [source revision and checksums](../../docs/architecture-sources.tsv).
Preserve the four same-named EN/VI variants
found under `docs/tai-lieu-kien-tuc-chuyen-biet/` in the source checkout. The
variants share core decisions and record different levels of specification.
Use the canonical policy for routing and refer proposed changes to the owners.

| Decision | Canonical system copy | Specialist copy |
| --- | --- | --- |
| Ecosystem ownership migration | §§4–5 and §21 assign target owners, require a tested migration and keep Remote's Lab lease and safety domain. | §3 specifies standalone and integrated authorities by tenant and record type, shadow-read reconciliation, legacy write fencing and rollback. |
| Ecosystem release composition | §16 keeps Remote's Lab release-set selection and leaves the ecosystem-wide manifest owner open. | §10 names Platform as a possible owner of a cross-application deployment catalog that references the Lab lock. |
| Lab scan and protection | §§8.2 and 10 require a verified safe state, bounded test excitation, observed evidence and protected operation. | §6.2 gives scan excitation its own bounded permit, with DUT supply and experiment stimuli disabled and a separate pre-check for residual and external energy. |
| Lab Virtual Digital evidence | §§6.4–6.5 define exact pins and the full VD-01–VD-10 conditions, separating Windows integration from Linux Mini PC qualification. | §§5 and 12 restate the gates and give the AND/LED reference sequence and target-Linux acceptance sequence explicit operational detail. |

Keep these decisions separate from observed product implementation. Consult
the [dated six-repository evidence](../references/repository-map.md) for initial
navigation and verify present scope in the owners' checkouts and selected set.
