# Quyết định — trả lời 12 câu ở mục 9 của bản thiết kế hội đồng

> Đầu vào: [`docs/ui-ux-council.md`](ui-ux-council.md) mục 9 · [`docs/rust-port-brief.md`](rust-port-brief.md)
> Ngày: **01/08/2026** · Trạng thái mã nguồn tại thời điểm chốt: `e30e2ee`
>
> **Tình trạng: 10/12 đã quyết.** Hai câu còn lại là quyết định chi tiền và phạm vi, không phải quyết định kỹ thuật, và **không chặn thiết kế**.
>
> **Cách đọc:** mục ✅ là đã quyết, có bằng chứng đo được ngay trong tài liệu này.
> Mục ❓ là còn chờ chủ dự án, và ghi rõ nó đang chặn cái gì.
> Ba chỗ tài liệu này **bác lại** kết luận của hội đồng đều được đánh dấu ⚠️ kèm số đo.

---

## Bảng tra nhanh

| Câu | Nội dung | Trạng thái |
|:-:|:---|:---|
| 1 | Ngân sách chứng chỉ ký mã | ❓ chờ chủ dự án |
| 2 | Phạm vi hỗ trợ Windows 10 | ❓ chờ chủ dự án |
| 3 | Số crate phụ thuộc | ✅ chấp nhận 112, sửa README |
| 4 | Ba việc sửa ngay trên bản PowerShell | ✅ ①② đã xong · ③ sẽ làm |
| 5 | Mức xác minh sao lưu mặc định | ✅ giữ mẫu, sửa nhãn ⚠️ |
| 6 | Ngưỡng tuổi kết quả quét | ✅ sửa chỗ khác ⚠️ |
| 7 | Nút "Mở bản dòng lệnh" | ✅ có, sau khi có khóa |
| 8 | Máy thiếu phông chữ | ✅ nhúng phông dự phòng ⚠️ |
| 9① | Zalo có hiện lại ảnh sau khi xóa `resource\` | ✅ **có, hiện bình thường** |
| 9② | Nguồn đọc tên/avatar ngoài vùng bảo vệ | ✅ **không có** |
| 10 | Bộ giải mã JPEG XL | ✅ có ⚠️ tiền đề của hội đồng sai |
| 11 | Hai bản dùng chung tệp cấu hình | ✅ chia ba cách |
| 12 | Nút "Tiếp tục phần còn lại" | ✅ bỏ |

---

## ✅ Q3 — Chấp nhận 112 crate, và phải sửa README

**Đo được:** `cargo tree` trên `eframe 0.29` cho **112 crate riêng biệt**. Hội đồng nói 113 — lệch một, không đáng kể.

**Quyết:** chấp nhận. Nhưng dòng "**0 phụ thuộc**" trong README phải được **giới hạn phạm vi cho bản PowerShell**, vì với bản đó nó vẫn đúng thật. Hai bản sống song song nên README phải nói rõ từng bản, không được để một câu quảng cáo phủ lên cả hai.

**Việc phải làm:** sửa README trước khi phát hành bản Rust đầu tiên.

---

## ✅ Q4 — Hai trong ba việc đã xong

| | Việc | Trạng thái |
|:-:|:---|:---|
| ① | `Test-ConfirmPhrase` nhận mọi kiểu đặt dấu | **Xong** — commit `e30e2ee` |
| ② | Nhãn phím `X` nói rõ là xóa tệp trên đĩa | **Xong** — commit `e30e2ee` |
| ③ | Khóa liên tiến trình dùng chung hai bản | **Sẽ làm** |

**Về ③:** phải làm, vì chạy song song giờ là chế độ đã chốt — nghĩa là cảnh hai bản cùng mở sẽ xảy ra thật chứ không phải giả định. Bản `.ps1` cũng phải được sửa để lấy cùng một khóa.

**Một điểm hội đồng nói mà kiểm lại thì sai:** họ bảo `Test-ConfirmPhrase` cần chuẩn hóa NFC. Đã thử tận nơi — trên máy này dạng tổ hợp rời vẫn so khớp bình thường, nên không đưa vào bản vá.

---

## ⚠️ Q5 — Hội đồng bỏ sót một vế quan trọng

**Hội đồng mô tả** mức xác minh mặc định là "mẫu 50 tệp — 0,4% độ phủ", và xếp nhãn `ĐÃ SAO LƯU · SẠCH` vào diện nói dối.

**Đọc mã thì không phải vậy.** `Invoke-Backup` kiểm **cỡ tệp cho 100% số tệp đã chép**, rồi mới lấy mẫu 50 tệp để băm SHA-256. Nên độ phủ thật là:

> **100% cỡ tệp + 0,4% nội dung**, không phải 0,4% suông.

Khác biệt này quan trọng: chép thiếu, chép cụt, chép ra tệp rỗng — cả ba đều bị bắt ở vòng kiểm cỡ 100%. Cái mà mẫu 50 tệp bỏ lọt chỉ là hỏng nội dung mà vẫn đúng cỡ, tức là lỗi bit im lặng.

**Quyết:**
- Giữ **mẫu làm mặc định**. Ép 100% có thể khiến người dùng bỏ sao lưu hẳn, và đó là kết cục tệ hơn nhiều.
- **Nhãn phải nói đúng cả hai con số**, không được rút gọn thành một chữ "sạch".
- **Ép 100% với ổ tháo rời và ổ mạng**, vì đó là nơi lỗi im lặng hay xảy ra.

**Chi phí của mức 100%:** đọc cả nguồn lẫn đích, tức gấp đôi dung lượng. Ở tốc độ đĩa nguội đo được 41–53 MB/s, một bản sao lưu 9,72 GB mất khoảng 7 phút.

---

## ⚠️ Q6 — Hội đồng sửa sai chỗ

**Vấn đề hội đồng nêu** (đường tấn công về bản trùng lặp): *bản giữ lại chỉ được kiểm lúc quét, không kiểm lại lúc xóa* — người dùng xóa hội thoại trong Zalo giữa hai thời điểm đó.

**Rút ngắn ngưỡng tuổi kết quả quét KHÔNG sửa được điều này.** Nó chỉ thu hẹp cửa sổ chứ không đóng. Người dùng vẫn có thể xóa hội thoại trong 15 phút, hay trong 1 phút.

**Quyết — sửa đúng chỗ:** **xác minh lại bản giữ lại ngay trước khi xóa.** `Invoke-Delete` đã kiểm tồn tại và đọc cỡ thật cho từng tệp sắp bị xóa; thiếu đúng vế đối chiếu bản giữ lại còn sống hay không.

Ngưỡng thời gian giữ nguyên làm lưới chắn phụ, không phải biện pháp chính.

---

## ✅ Q7 — Theo thứ tự, không phải có hay không

Làm **khóa liên tiến trình (Q4③) trước**. Sau khi có khóa, nút "Mở bản dòng lệnh" cho lớp trợ năng hết nguy hiểm và nên ship.

Thứ tự này là bắt buộc: ship nút trước khi có khóa nghĩa là tự tay tạo ra cảnh hai tiến trình cùng thao tác trên một tập tệp.

---

## ⚠️ Q8 — Không được dừng hẳn vì thiếu phông

**Đo trên máy này:** `segoeui.ttf` · `arial.ttf` · `tahoma.ttf` · `segoeuib.ttf` — **có đủ cả bốn**.

**Rủi ro thực tế thấp:** Segoe UI có mặt trong mọi bản Windows kể cả bản N — bản N chỉ gỡ tính năng đa phương tiện, không gỡ phông hệ thống.

**Nhưng không đồng ý với phương án của hội đồng.** Hội đồng chốt: không tìm được phông thì báo lỗi rõ ràng rồi dừng. Một công cụ xóa dữ liệu mà chết vì thiếu phông thì tệ hơn là chạy với phông thay thế.

**Quyết:** chuỗi dự phòng `segoeui → arial → tahoma`, và **nhúng sẵn một phông tối thiểu phủ tiếng Việt làm chốt chặn cuối**. Giá vài trăm KB.

Lý do: chữ trong cửa xác nhận mà không đọc được thì chính nó là nguyên nhân gây mất dữ liệu. Đây là chỗ đáng trả tiền bằng kích thước tệp.

---

## ⚠️ Q10 — Tiền đề của hội đồng sai, đo lại thì kết luận đổi

### Phân bố thật trên 57.035 tệp dữ liệu Zalo

| Loại | Số tệp | Tỷ lệ |
|:---|---:|---:|
| `.jxl` | 26.469 | 46,4% |
| **Không có phần mở rộng** | 24.935 | 43,7% |
| `.rescache` | 4.202 | 7,4% |
| `.jpg` + `.png` | 1.429 | **2,5%** |

Hội đồng nhìn vào con số 2,5% và kết luận: không có bộ giải mã JXL thì phần lớn ảnh không xem trước được, ma sát mạnh nhất của giao diện bị vô hiệu.

### Nhưng nhóm "không có phần mở rộng" là gì?

Ngửi magic byte trên **400 mẫu ngẫu nhiên**:

| Nội dung thật | Tỷ lệ mẫu |
|:---|---:|
| **JPEG** | **88,5%** |
| MP4 / MOV | 7,0% |
| Khác (HTML, AAC, linh tinh) | 4,5% |

### Kết luận đổi

| Phương án | Tỷ lệ tệp xem trước được |
|:---|---:|
| Chỉ giải mã JPEG/PNG, **ngửi magic byte** | **≈ 41%** |
| Thêm bộ giải mã JXL | **≈ 88%** |
| Tin phần mở rộng (cách làm ngây thơ) | 2,5% |

**Hai quyết định:**

1. **BẮT BUỘC ngửi magic byte, CẤM tin phần mở rộng.** 43,7% số tệp không có đuôi, và gần chín phần mười trong số đó là JPEG. Đây là ràng buộc thiết kế, không phải tùy chọn tối ưu.

2. **Thêm bộ giải mã JXL.** Không có nó thì cửa xác nhận hiện 5 ảnh thật và 7 ô trống. Làm yếu đi một nửa ma sát mạnh nhất mà hội đồng thiết kế, để tiết kiệm 1–2 MB, trong một công cụ chuyên xóa ảnh — đó là đánh đổi sai.

---

## ✅ Q9① — Ảnh vẫn hiện bình thường

**Kiểm chứng thực nghiệm, 01/08/2026.** Sau khi xóa **15,05 GB bản trùng lặp** khỏi tài khoản `2068096368017928379`, chủ dự án mở lại hội thoại cũ trong Zalo: **ảnh vẫn hiện bình thường**, không có ô trống.

**Mở khóa ba thứ đang bị chặn:**

- Thẻ **🟢 Bản trùng lặp** giữ nhãn *an toàn nhất*.
- Thứ tự bốn thẻ ở **QĐ-17** đứng nguyên, không phải xếp lại.
- Dòng `→ Bắt đầu từ đây` được đặt trên thẻ Bản trùng lặp.

### Quan sát này chứng minh tới đâu

Cần nói rõ giới hạn, vì lời quảng cáo trong README dựa vào chỗ này.

**Đã chứng minh:** xóa bản trong `resource\<mã hội thoại>\` không làm hội thoại thủng ảnh.

**Chưa phân biệt được:** ảnh hiện lên là do Zalo đọc **bản gốc còn giữ lại** trong `video\`/`picture\`, hay do nó **tải lại từ máy chủ**. Nhìn bằng mắt thì hai trường hợp giống hệt nhau.

Khác biệt này có hệ quả thật:

| Nếu | Thì |
|:---|:---|
| Zalo đọc bản gốc còn giữ lại | Chế độ khử trùng lặp an toàn tuyệt đối, kể cả khi ngoại tuyến và kể cả với ảnh đã quá hạn lưu trên máy chủ |
| Zalo tải lại từ máy chủ | Ảnh **quá hạn lưu trên máy chủ** sẽ thủng, và người dùng ngoại tuyến thấy ô trống — nhãn *an toàn nhất* phải nói thêm điều kiện |

**Cách phân biệt, tốn 30 giây:** ngắt mạng, mở lại một hội thoại cũ đã bị dọn, cuộn tới ảnh cũ. Còn hiện thì là đọc bản gốc tại chỗ. Thành ô trống hoặc quay vòng tải thì là tải lại từ máy chủ.

Việc này **chưa chặn thiết kế** — thứ tự thẻ đã chốt được rồi. Nhưng nếu kết quả là "tải lại từ máy chủ" thì câu quảng cáo *"không mất một tấm ảnh nào"* trong README phải được viết lại kèm điều kiện.

---

## ✅ Q9② — Không có nguồn nào ngoài vùng bảo vệ

Đã kiểm `config.json`, `zsafe-storage.json`, `Preferences` — **chỉ đọc tên khóa, không đọc giá trị**, vì đây là dữ liệu cá nhân.

| Tệp | Số khóa cấp 1 | Khóa liên quan danh tính |
|:---|---:|:---|
| `config.json` | 30 | **không có** |
| `zsafe-storage.json` | 2 | **không có** |
| `Preferences` | 2 | **không có** |

**Kết luận:** ngoài vùng bảo vệ chỉ đọc được **mã số tài khoản**, không có tên hiển thị, không có avatar.

**Hệ quả thiết kế:** màn chọn tài khoản phải phân biệt bằng **mã số + dung lượng thư mục + ngày sửa gần nhất**. Không có đường nào hiện tên mà không đụng vào vùng bảo vệ, và đụng vào là phá nguyên tắc bất biến số 4.

---

## ✅ Q11 — Chia ba cách, không phải chung hết hay riêng hết

| Tệp | Quyết | Lý do |
|:---|:---|:---|
| `catalog.json` | **Dùng chung** | Dữ liệu người dùng tự sửa. Nhân đôi là mời gọi hai bản lệch nhau, rồi người dùng sửa một bên mà tưởng sửa cả hai |
| `settings.json` · `profiles.json` | **Riêng từng bản** | Trạng thái chạy, lược đồ sẽ khác nhau giữa hai bản. Dùng chung là hai bản ghi đè nhau |
| `logs\` | **Dùng chung** | Lịch sử dọn dẹp phải là **một dòng duy nhất**. Người dùng cần biết "máy này đã bị dọn những gì", không cần biết bản nào dọn |

Kèm **khóa liên tiến trình** (Q4③) để hai bản không cùng chạy.

---

## ✅ Q12 — Bỏ, và lý do mạnh hơn hội đồng đưa ra

Hội đồng chốt bỏ nút "Tiếp tục phần còn lại", lập luận: người dùng bấm Dừng để **nhìn**, không phải để tạm nghỉ.

**Có một lý do mạnh hơn.** Nút đó tồn tại để né chi phí quét lại. **Chi phí đó không còn nữa:**

| | Trước tối ưu | Hiện tại |
|:---|---:|---:|
| Quét lại 52.748 tệp | 105,0 s | **5,2 s** |

Chính công việc tối ưu của phiên này đã xóa lý do tồn tại của tính năng. Quét lại giờ rẻ hơn cả việc giải thích cho người dùng hiểu "phần còn lại" nghĩa là gì.

---

## ❓ Q1 — Ngân sách chứng chỉ ký mã · **chờ chủ dự án**

Đây là quyết định chi tiền, không phải quyết định kỹ thuật.

Điều cần biết trước khi quyết:

- **Chứng chỉ tự ký KHÔNG làm SmartScreen im lặng.** Ký bằng chứng chỉ tự tạo không giải quyết được gì cho người tải về.
- **Chứng chỉ mới vẫn phải tích lũy uy tín.** Ngay cả khi đã mua và ký đúng, những người tải sớm vẫn gặp cảnh báo cho tới khi đủ lượt tải.
- **Không ký** thì mọi người tải đều gặp cảnh báo, và một exe xóa hàng loạt tệp kèm gọi `vssadmin` là hồ sơ điển hình của báo động giả từ phần mềm diệt virus.

**Đang chặn:** toàn bộ mục 8 của brief, và kế hoạch phát hành.

---

## ❓ Q2 — Phạm vi hỗ trợ Windows 10 · **chờ chủ dự án**

Windows 10 đã hết hỗ trợ chính thức từ tháng 10/2025, nhưng vẫn còn nhiều máy đang chạy.

**Hỗ trợ gần như miễn phí:** egui chạy tốt trên Windows 10, và quyết định chọn khung (Q3) không đổi dù có nhắm Windows 10 hay không.

**Câu thật sự cần bạn quyết:** có bỏ công **kiểm thử** cho nó không — màn hình 1366×768, ca thiếu phông, các bản LTSC gọn.

**Đang chặn:** phạm vi kế hoạch kiểm thử, không chặn thiết kế.

---

## Ba chỗ tài liệu này bác lại hội đồng

Ghi riêng ra để phiên sau không đọc nhầm bản chốt của hội đồng như lời cuối.

| Câu | Hội đồng nói | Đo lại thì |
|:-:|:---|:---|
| **5** | Xác minh mặc định phủ 0,4% | **100% cỡ tệp + 0,4% nội dung.** Chép thiếu và chép cụt vẫn bị bắt |
| **8** | Thiếu phông thì báo lỗi rồi dừng | **Không được dừng.** Nhúng phông dự phòng — chữ xác nhận không đọc được mới là thứ gây mất dữ liệu |
| **10** | Không có JXL thì hầu hết ảnh mất xem trước | **41% vẫn xem được** nếu ngửi magic byte. Nhóm không đuôi là **88,5% JPEG**, không phải video |

Và một chỗ hội đồng nói mà kiểm lại thì không tái hiện được: **`Test-ConfirmPhrase` không cần chuẩn hóa NFC** trên máy này.
