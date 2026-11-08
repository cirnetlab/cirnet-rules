<!-- doc-id: LANG-01 -->
<!-- revision: 1 -->
<!-- source-sha256: 0485d70835b13079db100e861645a8fd4512c384a9503ddb7bb627455f8aab9c -->
<!-- translation-status: synchronized -->

<a id="LANG-01"></a>

# Ngôn ngữ và tích hợp

Dùng Rust cho dịch vụ, logic domain và công cụ quản trị CIRNET, C++ cho lõi Simulator, Python chỉ cho kiểm thử hoặc benchmark. Giữ stack Interface và RTL hiện có. Wrapper gọi công cụ native. Giữ IPC worker trừ khi ADR so sánh lỗi, thông lượng và ownership. Nếu cần FFI, quy định kiểu ABI, allocator/free, lifetime, panic/exception, luồng, hủy và deadline. Đối chiếu dependency, đóng gói và hai đầu biên.

[Chính sách](../policies/governance.md)
