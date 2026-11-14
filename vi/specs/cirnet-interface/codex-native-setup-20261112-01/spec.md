<!-- doc-id: CNR-CODEX-NATIVE-SETUP-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 3c7646ff7ded723e4dfa04a6b659363c89b5fd8ded15eeacb0addc1721eb1ca2 -->
<!-- translation-status: synchronized -->
<a id="scope"></a>

# Phạm vi cấu hình Codex cấp dự án

Binding đích `codex-e2e-20261112-01` trong `cirnet-interface` tại `f1ff70b9f22554c0680b92679e9e44a5544247ff`. Phạm vi đọc, ghi và kiểm chứng setup lấy đúng đường dẫn từ preview chỉ dẫn đã review và đích MCP của dự án.

- `.agents/skills/implementation/SKILL.md`
- `AGENTS.md`
- `docs/cirnetrules/en/AGENTS.md`
- `docs/cirnetrules/en/commands/README.md`
- `docs/cirnetrules/en/hooks/README.md`
- `docs/cirnetrules/en/policies/governance.md`
- `docs/cirnetrules/en/profiles/developer.md`
- `docs/cirnetrules/en/profiles/product-architecture.md`
- `docs/cirnetrules/en/references/cirnetrules-local.md`
- `docs/cirnetrules/en/routing/README.md`
- `docs/cirnetrules/en/rules/architecture.md`
- `docs/cirnetrules/en/rules/coding.md`
- `docs/cirnetrules/en/rules/documentation.md`
- `docs/cirnetrules/en/rules/security.md`
- `docs/cirnetrules/en/rules/testing.md`
- `docs/cirnetrules/en/skills/implementation/SKILL.md`
- `.codex/config.toml`

Review khối quản lý trong `AGENTS.md` và bảo toàn byte của nội dung người dùng. Thay đổi TOML chỉ giới hạn ở bảng chỉ đọc `mcp_servers.cirnetrules`. Cấu hình toàn cục, quy tắc phê duyệt và cấu hình sandbox nằm ngoài phạm vi. Tài liệu chỉ dẫn sinh ra có ngôn ngữ ranh giới governance, nên mức rủi ro đề xuất là high và nghiệm thu vẫn cần review độc lập. Dùng bước `interface-diff` đã đăng ký và giữ receipt của cả hai thao tác ghi. Spec và plan ghi nhận bài thử cấu hình này nên không cần đích đồng bộ tài liệu riêng sau nghiệm thu.

[Kế hoạch](../../../plans/cirnet-interface/codex-native-setup-20261112-01/plan.md)
