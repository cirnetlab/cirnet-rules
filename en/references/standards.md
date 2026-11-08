<!-- doc-id: STANDARDS -->
<!-- revision: 1 -->
<a id="applicability"></a>

# Standards applicability

Checked 2026-11-08 on ISO's official catalogue. All seven editions are
published. The 29148 replacement is a [DIS](https://www.iso.org/standard/94091.html),
so 2018 remains the published reference. Access covered public catalogue
metadata and scope descriptions. Each mapping is an **orientation reference**.
Clause-level conformity and certification are outside the review scope.

| Edition and official source | Scope | CIRNET rule | Artifact or check | Owner role | Open evidence |
| --- | --- | --- | --- | --- | --- |
| [ISO/IEC/IEEE 12207:2026](https://www.iso.org/standard/90219.html) | Software life cycle | REL-01 | Transition, release gate evidence and support policy | release maintainer | Operational acceptance |
| [ISO/IEC/IEEE 15288:2023](https://www.iso.org/standard/81702.html) | Systems integration | ARCH-01 | Product profiles and system combinations | architect | Board and deployment evidence |
| [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) | Architecture description | ARCH-01 | Roles, stakeholder concerns and decision records | architect | Owner review of product viewpoints |
| [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) | Requirements and traceability | QA-01 | Repository tasks and release evidence bindings | verifier | Approval for each product task |
| [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) | Product quality model | QA-01 | Testable quality targets in the verification rule | verifier | Approved target workloads and thresholds |
| [ISO/IEC/IEEE 29119-2:2021](https://www.iso.org/standard/79428.html) | Test processes | QA-01 | Test commands, result identity and limitations | verifier | Full product test runs |
| [ISO 10007:2017](https://www.iso.org/standard/70400.html) | Configuration management guidance | REL-01 | Reviewed snapshots, immutable manifests and drift check | release maintainer | Clean signed-off producer receipts |

The Rust registry checks identities and trace links. Product acceptance and
semantic review remain manual where evidence is not machine-verifiable.
[Rules](../README.md) · [SemVer](https://semver.org/) · [OpenAPI](https://spec.openapis.org/oas/v3.1.0.html) · [JSON Schema](https://json-schema.org/) · [Protobuf](https://protobuf.dev/)
