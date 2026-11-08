<!-- doc-id: HANDOFF -->
<!-- revision: 2 -->
<!-- source-sha256: 9fe2815061eb2ae47c6c8dcf0044eef3a8f7f44a1045debac0e7d25884425b21 -->
<!-- translation-status: synchronized -->

<a id="decision"></a>

# Bàn giao governance và sản phẩm

**GOVERNANCE_IMPLEMENTATION** bao gồm registry, tài liệu cặp, kiểm tra chạy
được và adapter di động trong checkout. Xem [bằng chứng kiểm chứng](verification.md).

**PRODUCT_0.2.4_CUTOVER_READINESS** bị chặn. [Audit chỉ đọc](product-audit.md)
ghi sáu snapshot chính xác, đầu vào đã phát hành và các gate còn mở.
Tổ hợp đã phát hành tách khỏi train native đề xuất.

Xếp thứ tự sau khi được phép sửa sản phẩm.

1. Thống nhất family contract, biểu diễn đã rà và ca từ chối. Đóng băng bản xuất API Remote đã duyệt và dựng lại SDK Contracts trước khi chọn đầu vào Interface.
2. Dựng worker C++ giới hạn và scaffold FPGA mã nguồn với receipt bất biến. Ghép Hardware với Contracts và worker chính xác, kiểm chứng vòng đời fail-closed.
3. Dựng Interface với SDK khớp. Kiểm chứng runtime Remote, cài sạch, trình duyệt, Stop và rollback. Tách HIL vật lý.
4. Ghi tổ hợp chính xác ở repo sở hữu, xác minh bằng chứng rồi xin phê duyệt phát hành và kích hoạt tường minh.

Dùng [hướng dẫn adapter](../adapters/README.md) để bootstrap theo phạm vi và
[hướng dẫn công cụ](../tools/README.md) cho kiểm tra release và drift.

<a id="record-templates"></a>

## Mẫu hồ sơ dùng chung

| Hồ sơ | Các trường cần điền trước khi bàn giao |
| --- | --- |
| Quyết định thiết kế | ID, vấn đề, ràng buộc, phương án, đề xuất chọn, lý do, đường dẫn và contract bị ảnh hưởng, chủ sở hữu, tham chiếu phê duyệt, rollback. |
| Công việc sản phẩm | Yêu cầu hoặc ADR, ID công việc, chủ sở hữu, repo và đường dẫn, ID công việc phụ thuộc, revision và artifact dự kiến, kiểm tra nghiệm thu, định danh bằng chứng, gate release, bước tiếp theo. |
| Đề xuất ngoại lệ có phạm vi | ID quy tắc, phạm vi và hạn chính xác, lý do, bằng chứng bù trừ, chủ sở hữu, người duyệt và tham chiếu phê duyệt thực. Xem chính sách đóng ngoại lệ ban đầu. |
| Bằng chứng kiểm chứng | Commit nguồn và governance, lệnh, phiên bản công cụ, nền tảng, thời điểm UTC, mã thoát, loại bằng chứng, đường dẫn kết quả và digest, giới hạn. |

Tách trạng thái đề xuất, triển khai và kiểm chứng. Dùng [chính sách](../policies/governance.md)
cho phê duyệt và [định dạng bằng chứng](../schemas/release-evidence.md) cho đầu vào máy.
