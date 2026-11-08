<!-- doc-id: PRODUCT-PROFILES -->
<!-- revision: 1 -->
<!-- source-sha256: 1962cdc4edc9fa3413380843ee79ca6dcbaa55493c30dec7b2d503457c1ed5f5 -->
<!-- translation-status: synchronized -->

<a id="profiles"></a>

# Hồ sơ kiến trúc sản phẩm

| Hồ sơ | Quyền sở hữu và chiều phụ thuộc | Bằng chứng |
| --- | --- | --- |
| Rust service/library | Thư viện domain sau adapter ứng dụng. Contracts cấp dữ liệu vào Remote và Hardware. RPC cục bộ. Tách startup, lưu trữ, test và đầu ra gói. | Build khóa, kiểm chứng API/RPC và Stop. |
| C++ worker | Mô hình giới hạn và IPC đóng khung do Hardware giám sát. Tách nguồn, cây build CMake, fixture và binary. | CNP1/CNP2, đầu vào sai, tính tất định và thoát. |
| Frontend | Giữ Vue, TypeScript và Vite với SDK sinh từ schema. Trình duyệt sở hữu UX. An toàn trạm và chọn release thuộc repo khác. | Định danh SDK khóa, typecheck, test ứng dụng và trình duyệt thật trên cặp native đúng. |
| Contracts/codegen | Nguồn contract, schema và sinh mã đi trước binding downstream. Ghi revision generator. Giữ đầu ra sinh tái lập. | Digest handoff chuẩn, operation ID và parity đa ngôn ngữ. |
| FPGA RTL | RTL và testbench sở hữu scaffold logic. Hardware dùng ABI đã duyệt. Tách bằng chứng board, hiệu chuẩn và timing. | Parser, mô phỏng RTL, tổng hợp/timing và board khi công bố. |

Giữ test và schema sản phẩm tại repo sở hữu, lưu artifact ngoài repo.
[Audit sản phẩm](../references/product-audit.md)
