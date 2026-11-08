<!-- doc-id: SKILL-bilingual-sync -->
<!-- revision: 1 -->
<!-- source-sha256: 0e14774d8f483768c9155ec0e793368309e2821aef14d2f2c52a749bc769cf23 -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Đồng bộ song ngữ

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Tài liệu tiếng Anh thay đổi |
| Đầu vào | Nguồn EN, bản VI, thuật ngữ và ID quan trọng |
| Bước | Cập nhật revision tiếng Anh và lời văn tiếng Việt cùng nhau. Giữ định danh và anchor. Ghi digest nguồn đã chuẩn hóa. Kiểm tra link và rà ngữ nghĩa. |
| Đầu ra | Cặp tài liệu đạt kiểm tra cấu trúc và ghi rõ trạng thái rà ngữ nghĩa. |
| Khi lỗi | Hash cũ, tệp mồ côi hoặc sai ngôn ngữ an toàn chặn nghiệm thu. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
