# Third-party notices

The toolkit includes Markdown, the original Rust documentation checker and
tests, and local PowerShell packaging and installation scripts. The checker uses
the Rust standard library. Scripts use PowerShell 7, its .NET runtime and local
Git for packaging provenance. See [LICENSE](LICENSE).

CI pins [actions/checkout v4.2.2](https://github.com/actions/checkout/releases/tag/v4.2.2).
The action runs on the hosted runner. Client documentation is linked in the setup
guide.

The former Governance CLI used SQLite through rusqlite (SQLite public domain),
the Rust MCP SDK (Apache-2.0), Tokio (MIT) and unicode-segmentation
(MIT or Apache-2.0). Their dependency records and original notices remain in the
[preserved Governance revision](https://github.com/cirnetlab/cirnet-rules/tree/646afa38516bbfd276b0aecb7a138ab427df5fa9).
Anyone distributing that runtime must review its own contents and applicable
upstream notices. This note does not replace those notices or license terms.
