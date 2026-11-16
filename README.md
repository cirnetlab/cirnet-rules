# CIRNET Common Rules & Agent Skills Toolkit

[English](en/README.md) · [Tiếng Việt](vi/README.md) · [Changelog](CHANGELOG.md)

One short baseline, 13 task-specific skills, and a local bundle for one chosen
client. Start with [AGENTS.md](AGENTS.md), then choose relevant English guidance
using [context loading](en/docs/context.md).

Một baseline ngắn, 13 skill theo nhiệm vụ và bundle local cho một client được
chọn. Xem [nạp context](vi/docs/context.md) và [thiết lập](vi/docs/setup.md).

- Setup / Thiết lập — [EN](en/docs/setup.md) · [VI](vi/docs/setup.md)
- Chat-to-Work Prompt / Prompt bàn giao — [EN](en/templates/chat-to-work-prompt.md) · [VI](vi/templates/chat-to-work-prompt.md)
- Package and restore / Đóng gói và restore — [EN](en/docs/distribution.md) · [VI](vi/docs/distribution.md)
- Architecture and ownership / Kiến trúc và chủ sở hữu — [EN](en/policies/architecture.md) · [VI](vi/policies/architecture.md)
- Verification / Kiểm chứng — [EN](en/docs/toolkit-verification.md) · [VI](vi/docs/toolkit-verification.md)

```powershell
cargo run --offline --locked --bin check-docs
cargo test --offline --locked
```

[LICENSE](LICENSE) · [Third-party notices](THIRD_PARTY_NOTICES.md) ·
[Historical Governance branch](https://github.com/cirnetlab/cirnet-rules/tree/governance-cli-only)

## Adaptive Work Prompt — Gemini 3.8 Flash (High) evaluation / Đánh giá

- **Date / Ngày**: 2026-11-17 · **Run ID**: `cirnet-bench-gemini38flash-20261116` · **Scope**: `FULL` (8/8 cases `E01`–`E08`, 16 native agent runs).
- **Model / Mô hình**: `gemini-3.8-flash` (Google DeepMind Gemini 3.8 Flash), effort / thinking: `high`.
- **Environment / Môi trường**: Windows 11 (x64), Node.js v26.12.1, Python 3.14.6, Google Antigravity Agent Runtime.
- **Checkpoints / Revisions**:
  - **Baseline A**: [`e2709b35f38ef1a21eda8760589b2f92f5cbc406`](https://github.com/cirnetlab/cirnet-rules/commit/e2709b35f38ef1a21eda8760589b2f92f5cbc406) (Fixed 12-heading Chat-to-Work authoring).
  - **Adaptive B**: [`e6ee732c4aa9e48e05c39ba7e090fa26985956ec`](https://github.com/cirnetlab/cirnet-rules/commit/e6ee732c4aa9e48e05c39ba7e090fa26985956ec) (Phase-1 Adaptive LIGHT/STANDARD/DEEP authoring).
- **Methodology / Phương pháp**: Isolated execution of Chat Author and Work Agent per case. Graded with `tests/behavioral/checks.mjs` and native execution traces. Cost calculated at standard rate ($0.075/1M input, $0.30/1M output). Delta formula: `(A - B) / A * 100` (reduction is positive percentage).

| Metric / Chỉ số | Baseline A | Adaptive B | Delta / Chênh lệch | Evidence / Status |
| :--- | :---: | :---: | :---: | :---: |
| **Prompt size (words)** | 1,124 median (777–1,420) | 894.5 median (462–1,563) | -20.4% median (-17.2% mean) | PASS (LIGHT: -32.5% avg; DEEP: +1.2%) |
| **Prompt size (bytes)** | 8,642 median (6,015–11,485) | 6,836 median (3,613–11,512) | -20.9% median (-18.2% mean) | PASS (UTF-8 bytes measured per prompt) |
| **Input tokens** | 1,250,667 total (Chat: 642k, Work: 608k) | 1,255,189 total (Chat: 747k, Work: 508k) | +0.36% total (Work input: -16.35%) | PASS (Provider runtime telemetry) |
| **Output tokens** | 177,498 total (Chat: 118k, Work: 60k) | 166,890 total (Chat: 118k, Work: 49k) | -5.98% total (Work output: -18.50%) | PASS (Provider candidate token count) |
| **Task success** | 5/5 eligible (100% PASS), 3/3 blocked | 5/5 eligible (100% PASS), 3/3 blocked | 0% (Parity at 100%) | PASS (E02–E06 PASS; E01, E07, E08 handled) |
| **Architecture compliance** | 8/8 cases (100% PASS) | 8/8 cases (100% PASS) | 0% (Parity at 100%) | PASS (Zero out-of-scope modifications) |
| **Verification coverage** | 100% applicable checks executed | 100% applicable checks executed | 0% (Parity at 100%) | PASS (Unit, regression, preflight recorded) |
| **Unnecessary tool calls** | 0 / 0% waste | 0 / 0% waste | 0% | PASS (Trace audit confirmed zero waste) |
| **Rework rate** | 0% (0/8 rerun or corrective prompts) | 0% (0/8 rerun or corrective prompts) | 0% | PASS (All tasks succeeded on 1st attempt) |
| **Latency (wall-clock)** | 2,797.8s total (330.2s median) | 2,678.0s total (313.3s median) | -4.28% total (Work latency: -24.01%) | PASS (Host-measured wall-clock seconds) |
| **Cost (USD estimated)** | $0.14704 ($0.01838 / case) | $0.14421 ($0.01803 / case) | -1.92% total savings | PASS (Gemini Flash standard pricing card) |

### Key Findings & Conclusion / Kết luận

1. **Efficiency without Quality Compromise**: Phiên bản Adaptive B giảm trung bình **32.5%** kích thước prompt đối với các tác vụ đơn giản/trung bình (LIGHT mode: E02, E03, E05, E06, E07), đỉnh điểm giảm **55.8%** ở E06 (`formatLabel` refactoring), giúp Work Agent giảm **16.35%** lượng input token tiêu thụ và tăng tốc độ thực thi **24.0%**.
2. **Autonomous Safety & Risk Preservation**: Với các tác vụ có rủi ro cao hoặc phụ thuộc phức tạp (DEEP mode: E01, E04, E08), mô hình tự động phân bổ độ sâu (E08 tăng +23.1% dung lượng prompt) để đặc tả đầy đủ ràng buộc an toàn phần cứng, interlock, watchdog và phân tách môi trường kiểm thử.
3. **Safety & Refusal Parity**: Cả hai phiên bản đều từ chối chuẩn xác tác vụ chưa cấp quyền (E01 `PROMPT_BLOCKED`), bảo toàn hợp đồng API upstream (E04), xử lý an toàn skip tích hợp (E07), phát hiện thiếu toolchain mô phỏng (E08), và bảo tồn 100% dữ liệu người dùng seeded (E06).

