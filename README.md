# CIRNET Common Agent Rules & Skills

[English documentation](en/README.md) · [Tài liệu tiếng Việt](vi/README.md)

**Agents use English. Vietnamese is for human readers.**
**Agent đọc tiếng Anh. Bản tiếng Việt dành cho người đọc và review.**

The canonical baseline is [en/AGENTS.md](en/AGENTS.md); root
[AGENTS.md](AGENTS.md) is its identical standalone distribution copy.
[CLAUDE.md](CLAUDE.md) imports that copy.
Use only the relevant English hooks, policies, plans, specs, references, rules
and skills. After implementation/build, update both affected documentation versions.

Nguồn chuẩn là tiếng Anh trong `en/`; mỗi tài liệu có bản Việt cùng đường dẫn
dưới `vi/`. Sau triển khai/build, Agent cập nhật cả cặp bị ảnh hưởng, soát nghĩa
và chạy hook kiểm tra. Không nạp bản Việt thành nguồn mệnh lệnh thứ hai.

- Setup / Thiết lập: [EN](en/docs/setup.md) · [VI](vi/docs/setup.md)
- Documentation policy / Chính sách tài liệu: [EN](en/policies/documentation.md) · [VI](vi/policies/documentation.md)
- Hooks and checks / Hooks và kiểm tra: [EN](en/hooks/README.md) · [VI](vi/hooks/README.md)
- Verification / Nghiệm thu: [EN](en/docs/bilingual-verification.md) · [VI](vi/docs/bilingual-verification.md)

Run from this checkout / Chạy tại checkout này:

```powershell
python -B hooks/check_docs.py
python -B -m unittest discover -s tests -p 'test_*.py' -v
git diff --check
```

The optional local checker needs installed Python; reading the rules does not.
No native/global hooks are installed. Product repositories retain their own
architecture, build/test, contracts, release and physical-safety decisions.

Checker local là tùy chọn và cần Python đã có; đọc rules không cần runtime.
Không cài hook native/global. Mỗi repo sản phẩm vẫn giữ quyền quyết định riêng.

[LICENSE](LICENSE) · [Third-party notices](THIRD_PARTY_NOTICES.md) ·
[Legacy Governance backup](https://github.com/cirnetlab/cirnet-rules/tree/governance-cli-only)
