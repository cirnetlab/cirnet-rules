<!-- doc-id: CNR-SHARED-COORDINATION -->
<!-- revision: 1 -->
<!-- source-sha256: 7991cdccab9b998aca76c260064ca2fab41bc6a13bcabd491632333a6b7ad469 -->
<!-- translation-status: synchronized -->
<a id="coordination"></a>

# Phối hợp task dùng chung

Xác nhận binding repo mặc định và đăng ký một Git worktree riêng trong cùng
common directory. Đặt map `bindings` tùy chọn trong descriptor trước khi
duyệt task. Task cố định binding đã chọn. Đổi binding cần revision task
tường minh và phê duyệt lại.

```sh
cnr workspace binding add --repo REPO --binding NAME --path WORKTREE_RELATIVE --actor OPERATOR --json
cnr workspace binding validate --repo REPO --binding NAME --json
cnr task status --task ID --json
cnr task session-prepare --task ID --revision REVISION --actor APPROVER --request-id REQUEST --json
cnr task claim --task ID --revision REVISION --session-file SESSION_FILE --request-id REQUEST --json
cnr task renew --task ID --generation GENERATION --session-file SESSION_FILE --request-id REQUEST --json
cnr task report --task ID --revision REVISION --generation GENERATION --session-file SESSION_FILE --request-id REQUEST --report REPORT_JSON --json
cnr task dependency check --task ID --json
cnr audit task --task ID --json
```

Dùng `--workspace` nhất quán nếu workspace khác mặc định. Đọc revision task
và generation lease mới sau mỗi thao tác ghi. Mỗi yêu cầu ghi dùng một
request ID nghiệp vụ riêng. Chỉ dùng lại ID với đối số ban đầu để lấy kết quả
đã ghi. Lease mặc định 900 giây, giới hạn theo hạn quyền. Writer hết hạn có
thể còn tiến trình ngoài đang chạy. Khôi phục cần operator review và xác nhận
tường minh writer ngoài đã dừng. Dùng `cnr task recover --stopped true`
với revision task, generation, actor cấp quyền và request ID.

Tiếp tục qua các cổng `cnr task verify`, review và chấp nhận hiện hành. Báo cáo
worker cung cấp khai báo có giới hạn. CIRNETRULES niêm phong byte đầu vào,
thay đổi, binding và bằng chứng dependency thực tế trước kiểm chứng. Nhãn
actor và phiên local là lời xác nhận. Digest biên nhận hợp lệ không xác thực
được danh tính độc lập.

Để nâng cấp hồ sơ task hiện có, dừng binary cũ và sao lưu thư mục task bị
Git ignore. Chạy `cnr task coordination migrate --confirm preview` với ID
task, revision và actor cấp quyền. Chạy lại với digest đã review. Tiếp tục
qua quy trình sửa revision và review quyền khi cần. Giữ backup gốc để khôi
phục. Binary cũ không thể bị ép tuân theo các trường task mới.

## Gate CI có thể di chuyển

```sh
cnr task evidence export --task ID --output ABSOLUTE_BUNDLE_FILE --json
cnr task evidence inspect --bundle ABSOLUTE_BUNDLE_FILE --json
cnr ci verify --governance-root TRUSTED_ROOT --authority-commit TRUSTED_COMMIT --bundle ABSOLUTE_BUNDLE_FILE --expected-digest APPROVED_DIGEST --target CANDIDATE_CHECKOUT --json
```

Export cần task lá có submission hiện hành đã niêm phong. Operator commit
candidate trên base được duyệt và CI quan sát chính xác đường dẫn thay đổi
cùng byte tệp trong phạm vi. Cấp digest bundle được duyệt và commit authority
tin cậy qua kênh review riêng. Inspect chỉ kiểm tra toàn vẹn. Gate dùng kiểm
tra Git và định dạng có kiểm soát theo policy tin cậy, cùng digest artifact
đã tiêu thụ khi được yêu cầu. Build sản phẩm, review có xác thực và cổng
phần cứng cần bằng chứng riêng.

Workflow tái sử dụng `.github/workflows/ci-task-gate.yml` nhận commit authority,
candidate và evidence bất biến. Gọi từ nguồn tin cậy và cấp digest đã duyệt
độc lập với đầu vào PR. Workflow không thực thi script từ candidate. Gate đọc lại blob Git thô
và yêu cầu byte checkout khớp. Nếu filter đổi xuống dòng hoặc nội dung,
chuẩn bị candidate với byte đã review rồi kiểm tra lại.

Biên nhận gate là kết quả JSON của `cnr ci verify`. Giữ kết quả trong log
hoặc kho artifact của runner tin cậy. Bundle có thể di chuyển và biên nhận
chứa digest cùng đường dẫn logic. Giữ kho task nguồn riêng và tệp phiên
đang hoạt động ở máy local.
