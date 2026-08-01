# Việc còn lại — và prompt khởi động phiên triển khai

> Rà soát ngày **01/08/2026** · **189/189 phép thử đạt**
>
> Nguồn: [`rust-port-brief.md`](rust-port-brief.md) · [`ui-ux-council.md`](ui-ux-council.md) · [`quyet-dinh.md`](quyet-dinh.md)
> Mọi việc dưới đây đều đã truy về tận mã nguồn, không lấy từ trí nhớ.

---

## Tình hình một dòng

Ba tài liệu thiết kế đã xong. **10/12 quyết định đã chốt.** Hai câu còn treo là chi tiền và phạm vi, không chặn thiết kế.

Rà soát lần này phát hiện **một lỗ hổng an toàn đang sống trong bản PowerShell** — đã sửa xong, xem P0 ngay dưới.

---

# P0 — Lỗ hổng đang sống · ✅ ĐÃ SỬA

## P0-1 · Khử trùng lặp không kiểm lại bản giữ lại trước khi xóa

**Trạng thái: ✅ đã sửa, có phép thử canh cả hàm lẫn chỗ nối dây.**

`Invoke-DedupScan` ghi lại đường dẫn bản giữ lại vào trường `Keeper` lúc quét. `Invoke-Delete` **không bao giờ đọc trường đó** — đã truy: `Keeper` chỉ xuất hiện ở chỗ hiển thị (`Show-ScanDetail`) và chỗ xuất CSV.

Nghĩa là công cụ xóa bản trong `resource\` mà **không kiểm bản gốc trong `video\`/`picture\` còn sống hay không**.

**Vì sao nguy hiểm hơn vẻ ngoài:** chế độ khử trùng lặp cố ý dùng **xác nhận nhẹ** — chỉ `c/k`, không bắt gõ `XÓA` — và điều đó chỉ chính đáng nhờ tiền đề *"bạn không mất gì, còn một bản giống hệt"*. Nếu bản giữ lại đã biến mất giữa lúc quét và lúc xóa thì tiền đề sai, mà mức xác nhận vẫn nhẹ.

**Cửa sổ rủi ro:** người dùng xóa hội thoại trong Zalo, hoặc Zalo tự dọn, giữa hai thời điểm. Kết quả quét được giữ tới 2 giờ.

**Đã sửa thế nào:** thêm `Test-KeeperAlive`, gọi trong vòng lặp của `Invoke-Delete` ngay trước lệnh xóa. Bản giữ lại phải còn tồn tại **và** còn đúng cỡ. Không đạt thì bỏ qua tệp đó, đếm riêng, ghi trạng thái `MẤTBẢNGỐC` vào nhật ký, và báo lên màn hình kèm lời khuyên quét lại.

Chỉ so tồn tại và cỡ, **không băm lại** — băm lại là đọc trọn cả hai tệp cho từng cặp, tức nhân đôi lượng đọc đĩa của cả lượt xóa. Lúc quét đã đối chiếu SHA-256 toàn tệp rồi.

**Một chỗ hở của chính bộ test lộ ra khi kiểm bằng đột biến.** Năm phép thử đơn vị cho `Test-KeeperAlive` đều xanh, nhưng **gỡ hẳn lời gọi hàm đó ra khỏi `Invoke-Delete` thì cả bộ test vẫn xanh** — vì phép thử một hàm rời chỉ chứng minh *hàm* đúng, không chứng minh nó *được gọi*.

Đã bổ sung một nhóm phép thử đọc AST để canh **chỗ nối dây**: `Invoke-Delete` phải gọi `Test-KeeperAlive`, `Test-BackupClean`, `Test-Protected`; `Invoke-Scan` phải gọi `Test-Protected`; `Invoke-Backup` phải còn chốt `IsPathRooted`.

> Bài học chung: **tách một lớp an toàn thành hàm cho dễ thử thì phải canh luôn chỗ nối dây**, nếu không là tự tay tạo ra một lỗ hổng câm.

> Đây chính là đường tấn công mà hội đồng xếp mức **CHẾT NGƯỜI**. Hội đồng coi nó là vấn đề của giao diện mới; rà soát này cho thấy nó đã có sẵn trong bản đang chạy.

---

# P1 — Bản PowerShell, làm được ngay, có lợi ngay

| # | Việc | Nguồn | Ghi chú |
|:-:|:---|:---|:---|
| **P1-1** | **Khóa liên tiến trình.** Mutex đặt tên + tệp khóa mang PID, dùng chung với bản Rust sau này | T-2 · Q4③ · QĐ-05 | Chạy song song đã là chế độ chốt, nên cảnh hai bản cùng mở **sẽ** xảy ra. Cũng là điều kiện để ship nút "Mở bản dòng lệnh" (Q7) |
| **P1-2** | **Nhãn xác minh nói đúng độ phủ.** Thay một chữ "sạch" bằng hai con số thật: 100% cỡ tệp + số tệp đã băm | T-3 · Q5 | Nhãn hiện tại cấp bảo chứng rộng hơn thứ nó thật sự kiểm |
| **P1-3** | **Ép xác minh 100% với ổ tháo rời và ổ mạng** | Q5 | Đó là nơi lỗi bit im lặng hay xảy ra |
| **P1-4** | **README giới hạn phạm vi "0 phụ thuộc"** cho bản PowerShell | Q3 | Bản Rust kéo 112 crate. Để câu đó phủ cả hai bản là nói dối |
| **P1-5** | **Kiểm ngoại tuyến** để đóng nốt phần còn treo của Q9① | Q9① | Ngắt mạng, mở hội thoại cũ đã dọn. Còn hiện = đọc bản gốc tại chỗ. Ô trống = tải lại từ máy chủ, và khi đó câu *"không mất một tấm ảnh nào"* phải viết lại kèm điều kiện |

---

# P2 — Chờ chủ dự án quyết

| # | Câu | Chặn cái gì |
|:-:|:---|:---|
| **P2-1** | **Ngân sách chứng chỉ ký mã hằng năm?** | Toàn bộ kế hoạch phát hành. Chứng chỉ tự ký **không** làm SmartScreen im lặng, và chứng chỉ mới vẫn phải tích lũy uy tín |
| **P2-2** | **Windows 10 có trong phạm vi hỗ trợ không?** | Chỉ chặn phạm vi kiểm thử. Không chặn thiết kế — egui chạy tốt trên Win 10 |

---

# P3 — Phiên triển khai Rust

## P3-A · Kế hoạch phải trả lời trước khi viết code

Chín câu ở mục 9 của [`rust-port-brief.md`](rust-port-brief.md). Tóm tắt:

1. Chia mô-đun — tách rõ **lõi an toàn** khỏi **vỏ giao diện**, lõi phải kiểm thử được mà không cần giao diện
2. Chọn crate, và cân với lời hứa "0 phụ thuộc"
3. Khung giao diện — **đã chốt egui** (QĐ-01), kế hoạch chỉ cần xác nhận
4. Thứ tự làm, kèm chốt kiểm chứng từng bước
5. 189 phép thử port kiểu gì — chúng đang lái công cụ bằng chuỗi phím qua stdin, cách đó không dùng lại được với giao diện đồ họa
6. Giữ tương thích bản sao lưu cũ, và kiểm chứng ra sao
7. Ký số · SmartScreen · diệt virus · kiểm chứng nguồn · cập nhật
8. Bố cục bên trong `rust\` và cách hai bản dùng chung tệp — **đã chốt cách chia** (Q11), kế hoạch chỉ cần chi tiết hóa
9. Mốc dừng — tiêu chí đo được để kết luận đi tiếp hay dừng

## P3-B · Ràng buộc đã chốt, mang thẳng vào bản Rust

| Mã | Ràng buộc | Nguồn |
|:---|:---|:---|
| **R-01** | **egui/eframe.** 2,86 MiB, không cần runtime ngoài | QĐ-01 |
| **R-02** | **Ngửi magic byte, CẤM tin phần mở rộng.** 43,7% số tệp không có đuôi, 88,5% trong số đó là JPEG | Q10 |
| **R-03** | **Có bộ giải mã JPEG XL.** Thiếu nó thì cửa xác nhận hiện 5 ảnh thật và 7 ô trống | Q10 · T-6 |
| **R-04** | **Nhúng phông dự phòng phủ tiếng Việt.** Không được dừng hẳn vì thiếu phông | Q8 |
| **R-05** | **Nhận diện tài khoản bằng mã số + dung lượng + ngày sửa.** Không có nguồn nào ngoài vùng bảo vệ đọc được tên hay avatar | Q9② · T-4 |
| **R-06** | `catalog.json` **chung** · `settings.json`/`profiles.json` **riêng** · `logs\` **chung** | Q11 |
| **R-07** | **Không có nút "Tiếp tục phần còn lại".** Quét lại chỉ mất 5,2 giây, lý do tồn tại của nó đã mất | Q12 |
| **R-08** | **Xác minh lại bản giữ lại ngay trước khi xóa** — cùng lỗ hổng P0-1, bản Rust không được lặp lại | Q6 |
| **R-09** | **Không đi xuyên reparse point** khi duyệt cây. `EnumerateDirectories` của .NET thì có, thư viện Rust phải kiểm chứng riêng | Brief §7 |
| **R-10** | **Không xóa đệ quy.** Xóa không đệ quy, ném lỗi khi thư mục hết rỗng | Brief §7 |
| **R-11** | **So chuỗi và đường dẫn dùng ordinal**, không theo vùng miền. Công cụ có phép thử chạy dưới `vi-VN` | Brief §7 |
| **R-12** | **Đo dung lượng trống của ổ đĩa**, không bao giờ cộng byte đã xóa. Volume Shadow Copy nuốt dung lượng | Brief §7 |
| **R-13** | **Chỉ đếm là đã xóa khi tệp thật sự biến mất** | Brief §7 |
| **R-14** | **Sao lưu sạch = không lỗi VÀ trọn vẹn.** Hai vế | `Test-BackupClean` |
| **R-15** | **Trình đọc màn hình là cổng mức 2, không phải mức 1.** egui + AccessKit chưa hoàn chỉnh | T-5 |

## P3-C · Chiến lược kiểm chứng, bắt buộc

- **Đối chiếu song song** bản PowerShell và bản Rust trên cùng sandbox, so từng dòng đầu ra và từng tệp còn lại
- **So bằng SHA-256**, không so bằng số lượng
- **Kiểm bằng đột biến** — cố tình phá một lớp an toàn rồi xem test có đỏ không
- **Không chạy thử trên dữ liệu Zalo thật ở chế độ có xóa.** Chỉ quét
- **189 phép thử hiện có là hợp đồng.** Port hết, không bỏ cái nào

---

# Prompt khởi động phiên triển khai

Dán nguyên khối dưới đây vào một phiên mới.

```text
Kho mã: D:\zalo-tool  ·  https://github.com/doivamong/zalo-cleanup  ·  nhánh main

Đọc bốn tài liệu này trước khi làm bất cứ việc gì, đọc hết chứ đừng lướt:
  docs/viec-con-lai.md      — việc còn lại, đọc TRƯỚC TIÊN
  docs/quyet-dinh.md        — 12 quyết định đã chốt, và ba chỗ bác lại hội đồng
  docs/rust-port-brief.md   — thực trạng, bài học đã trả giá, câu kế hoạch phải trả lời
  docs/ui-ux-council.md     — bản thiết kế giao diện chốt của hội đồng UI-UX

Bối cảnh: công cụ này XÓA VĨNH VIỄN dữ liệu cá nhân, không qua Thùng rác. Chủ dự
án đã dùng nó xóa 149.309 tệp / 37 GB ảnh video thật. Mọi quyết định phải đọc
dưới ánh sáng đó.

Đang viết lại bằng Rust, có giao diện đồ họa, phát hành exe dựng sẵn, đặt tại
D:\zalo-tool\rust\. Hai bản chạy song song, bản PowerShell KHÔNG bị bỏ.

LÀM THEO ĐÚNG THỨ TỰ NÀY:

Bước 1 — làm P1.
  Năm việc nhỏ trên bản PowerShell, đều có lợi ngay cho người dùng hôm nay.
  P1-1 khóa liên tiến trình là điều kiện của nhiều thứ về sau.

Bước 2 — lập kế hoạch port, trả lời chín câu ở P3-A.
  CHƯA viết code Rust ở bước này. Ra kế hoạch trước, có mốc dừng đo được.

Bước 3 — bắt đầu port theo kế hoạch, lõi trước vỏ sau.
  Mỗi bước phải có chốt đối chiếu song song với bản PowerShell.

QUY TẮC BẤT DI DỊCH KHI LÀM:

- Chạy bộ test sau MỖI lần sửa: powershell -NoProfile -ExecutionPolicy Bypass
  -File ".\ZaloCleanup.Tests.ps1" -Full     (hiện 189/189 đạt)
- Sửa lớp an toàn thì phải KIỂM BẰNG ĐỘT BIẾN: cố tình phá rồi xem test có đỏ
  không. Test không đỏ nghĩa là test vô dụng, không phải mã đúng.
- Không chạy thử trên dữ liệu Zalo thật ở chế độ có xóa. Chỉ quét.
- Tệp .ps1 phải lưu UTF-8 CÓ BOM. Thiếu BOM là mọi chữ tiếng Việt vỡ.
- Mỗi thay đổi một nhánh riêng, gộp vào main bằng fast-forward, rồi xóa nhánh.
- Lời nhắn commit viết tiếng Việt, nói VÌ SAO chứ không chỉ nói CÁI GÌ.

ĐIỀU QUAN TRỌNG NHẤT:

Đừng tin tài liệu, kể cả bốn tài liệu trên. Chúng đã sai vài lần và mỗi lần đều
do đoán thay vì đo. Hội đồng UI-UX từng kết luận sai ba chỗ, và bị bác lại bằng
số đo — xem mục cuối của docs/quyet-dinh.md. Một ước tính trước đó lệch thật tế
hai mươi lần, chỉ vì mô phỏng một hàm bằng 2 luật trong khi bản thật có 23 luật.

Nghi ngờ thì ĐO. Đo được thì đừng đoán.
```

---

## Ba lời nhắc cho người đọc prompt trên

**Một.** Bốn tài liệu cộng lại hơn 1.700 dòng. Chúng thay cho việc khảo sát lại từ đầu, không thay cho việc đọc mã.

**Hai.** Bài học đắt nhất của dự án này, lặp lại đúng ba lần trong một phiên: **một nhánh không có test là một nhánh chưa từng chạy.** Mức xác minh SHA-256 toàn bộ từng hỏng hoàn toàn qua nhiều phiên bản mà không ai biết, vì chưa phép thử nào từng chọn mức đó.

**Ba.** Lỗ hổng P0-1 ở đầu tài liệu này được tìm ra bằng cách **đọc lại mã để kiểm chứng một lời của tác nhân**, chứ không phải bằng cách tin nó. Giữ thói quen đó.
