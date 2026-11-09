<!-- doc-id: CIRNETRULES-ADAPTER-CAPABILITIES -->
<!-- revision: 1 -->
<a id="adapter-capabilities"></a>

# External engine interoperability

CIRNETRULES can inspect a selected local engine without creating a session. A matching probe confirms that the peer reported the pinned version and used the expected response shape. Authentication, sandboxing, approval enforcement, and agent execution need separate evidence. Governance authoring works offline.

## Source audit

| Archive | Entries | Archive comment revision | SHA-256 |
| --- | ---: | --- | --- |
| OpenHands-main.zip | 3,277 | `1a66039ecd20d93e792eec257c7f5abf4d709a10` | `7e48061c8e1b3ff61f5382d427d37573d83c544f4bc517706d56542e297da88d` |
| opencode-dev.zip | 12,377 | `fee1ca5aa9e1d62a7ca42ef7eeaa8c02fe6ed080` | `2d25f85683c60484dfadc362269c3bc777b5bc330e0025ea468ecc14a8ba7f6d` |

The audit read selected files directly from the two archives. Both archive roots have MIT licenses. CIRNETRULES implements the wire subsets and UI patterns in Rust and Qt. No upstream source, runtime, or asset was embedded. Imported files retain their own provenance and licensing terms.

| Source mechanism | Classification | Use and evidence |
| --- | --- | --- |
| OpenHands `docs/architecture.md` and `src/components/features/skills/add-skill-modal.tsx` | REUSE_PATTERN | Separate the desktop and engine. The upstream Add Skill modal guides setup. CIRNETRULES authoring uses its own runtime and database. |
| OpenHands `src/api/skills-service.ts`, `src/utils/skill-enablement.ts`, and `src/utils/skill-scope.ts` | REIMPLEMENT | Catalog scope and enablement are separate from authored content. The compatibility preview maps skill metadata without activation. |
| OpenHands `src/api/agent-profiles-service/agent-profiles-service.api.ts` and `profile-behavior.ts` | REUSE_PATTERN | Reference profile resources instead of embedding credentials or copying selections into a live engine. |
| OpenHands `src/utils/plugin-launch-preflight.ts` | REUSE_PATTERN | Preserve typed values and surface unsupported fields. Contract fixtures cover false and opaque fields. |
| OpenHands `config/defaults.json`, `src/api/agent-server-compatibility.ts`, and `docs/architecture.md` | CONNECT_VIA_ADAPTER | Probe Agent Server 1.60.1 with `GET /server_info`. Fixtures cover the version fallback and authentication response. |
| OpenHands `src/api/conversation-service/agent-server-conversation-service.api.ts` and `src/api/event-service/event-service.api.ts` | CONNECT_VIA_ADAPTER (blocked) | Local start needs an upstream agent profile and a resolved workspace. Session events and approvals need runtime-scoped authorization. Governed session start and streaming are blocked. |
| OpenCode `packages/protocol/src/groups/server.ts` and `packages/server/package.json` | CONNECT_VIA_ADAPTER | Probe OpenCode 2.2.5 V2 with `GET /api/info`. Fixtures cover V2 shape, mismatch and version-only response rejection. |
| OpenCode `packages/core/src/config/plugin/agent.ts`, `packages/core/src/v1/config/agent.ts`, and `packages/schema/src/config/agent.ts` | REIMPLEMENT | Preview an explicitly selected V1 or V2 agent-profile dialect. Tests cover typed field retention and mixed-dialect rejection. |
| OpenCode `packages/core/src/skill.ts` and `packages/core/src/tool/plugin/skill.ts` | REUSE_PATTERN | Separate metadata from content and defer content to an authorized operation. |
| OpenCode `packages/core/src/config/plugin/reference.ts`, `packages/core/src/config/plugin/skill.ts`, and `packages/core/src/permission.ts` | REUSE_PATTERN | Separate file discovery, scoped references, and action/resource permissions. A compatibility preview grants no access. |
| OpenCode `packages/app/src/settings/pages.ts` and `packages/app/src/session/revert-machine.ts` | REUSE_PATTERN | Keep navigation, editor state, and revision transitions separate. Session rewind is distinct from governance rollback. |
| OpenCode `packages/protocol/src/groups/session.ts`, `event.ts`, and `permission.ts` | CONNECT_VIA_ADAPTER (blocked) | V2 creates sessions with location and permission fields. Its event stream can miss events when disconnected. Governed session creation, event reconciliation, and tool approval interception remain blocked. |

The public references describe profiles and configuration. The archive pins determine the implemented wire subset. [OpenHands Agent Profiles](https://docs.openhands.dev/openhands/usage/agent-canvas/agent-profiles) and [OpenCode configuration](https://opencode.ai/docs/config).

## Capability and trust boundaries

| Feature | OpenHands | OpenCode | Codex, Claude, Antigravity |
| --- | --- | --- | --- |
| Local version and shape probe | Available | Available | No probe |
| Offline source preview | SKILL.md frontmatter | Selected V1 or V2 agent-profile object | No format mapping |
| Portable local handoff | No adapter handoff | No adapter handoff | Handoff only |
| Session creation, context injection, model/tool calls, approvals, streaming, interrupt, resume, isolation, diff, usage | Blocked | Blocked | Native execution blocked |
| Verified live engine execution | Not run | Not run | Not run |

The private IPC exposes `cirnetrules.integration.list`, `cirnetrules.integration.probe`, and `cirnetrules.integration.import_preview`. The probe takes a numeric port and engine ID, with no host override. It sends a read-only GET to the fixed version endpoint at `127.0.0.1`. It uses one admitted job on a dedicated worker, a 1.2-second overall socket budget, an 8 KiB header limit, and a 32 KiB body limit. It never follows redirects, uses a proxy, sends credentials or reads any returned server paths. A server demanding authentication is reported separately. HTTP/1.x responses with an explicit content length or connection-close body are accepted for JSON inspection.

## Offline format subset

- OpenHands skill preview requires YAML frontmatter in `SKILL.md` with `name` and `description`. It extracts optional `license` and `compatibility`. Extensions remain source-only.
- OpenCode accepts strict JSON or Markdown with YAML frontmatter for one agent profile. It previews string model selection, description, mode and the disabled flag. The V1 `disable` flag maps to `disabled`. V1 `permission` and V2 `permissions` never become CIRNETRULES grants. Settings that belong to the other dialect produce a diagnostic. Whole-project configuration, JSONC, structured model selection, provider parameters, and upstream instructions remain outside this mapping.
- The report classifies fields as supported, translated, retained as source, or rejected. Policy, execution, and credential fields are rejected. The preview does not fetch references, activate skills, persist configuration, or start a process. It bounds input size and field count, rejects YAML aliases, anchors, tags and directives, and locates parse diagnostics when the parser supplies a location.

## Verification status

Fixture tests exercise both loopback version endpoints, dialect selection, malformed and duplicate fields, sensitive-field exclusion, and a slow probe alongside normal stdio dispatch. The probe fixture serves source-derived responses. It does not contain an upstream engine.

OR-17 has fixture coverage for metadata and compatibility reports. OR-40 and OR-41 have partial contract coverage for the supported probe and profile subset. Event mapping, actual server negotiation, and native execution remain unverified. OR-42 and OR-43 were not run because neither pinned engine was installed in the test environment and no authorized isolated engine or credentials were available. OR-44 has capability inventory coverage. The desktop capability presentation is verified separately.
