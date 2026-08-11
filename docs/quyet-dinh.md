# Quyết định — trả lời 12 câu ở mục 9 của bản thiết kế hội đồng

> Đầu vào: [`docs/ui-ux-council.md`](ui-ux-council.md) mục 9 · [`docs/rust-port-brief.md`](rust-port-brief.md)
> Ngày: **01/08/2026** · Trạng thái mã nguồn tại thời điểm chốt: `e30e2ee`
>
> **Tình trạng: 12/15 đã quyết.** Hai câu cũ còn treo là quyết định chi tiền và phạm vi, không chặn thiết kế. Câu thứ mười lăm, `Q15`, thì **có chặn** — nó là một mục tiếp cận mức 1.
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
| 13 | Đóng M6 khi cổng tái lập không đạt | ✅ **phương án A** |
| **14** | **`BP-01` và `BP-05` đâm nhau: có mở đường bàn phím tới lệnh xóa không** | ✅ **phương án B**, đã kiểm trên màn hình thật |
| **15** | **Phép dò của `ĐM-08` bỏ sót Narrator — làm gì** | ❓ **chờ chủ dự án · CHẶN mức 1** |

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

---

## ✅ Q13 — Đóng M6 theo phương án A · **chủ dự án đã chọn 02/08/2026**

Cổng M6 đòi "build tái lập được từ CI công khai, mã băm khớp bản tải về". Đo ra **không đạt**, vì hai chỗ chặn nằm ngoài mã của dự án — xem [`ke-hoach-port.md`](ke-hoach-port.md) §M6.

Ba phương án đã đặt ra, chủ dự án chọn **A**.

### A nghĩa là gì, nói cho hết

| Vẫn đúng | Không còn đúng |
|:---|:---|
| Bản phát hành là **đúng tệp máy chủ CI đã dựng**, không ai đụng vào giữa chừng | Người dùng **không** tự dựng lại rồi so byte được |
| `SHA256SUMS.txt` bắt được mọi sửa đổi **trên đường truyền** | Mã băm **không** chứng minh tệp ấy sinh ra từ đúng mã nguồn này |
| `ZaloCleanup.ps1` vẫn là văn bản thuần, đọc được từng dòng, và **cùng bộ test lái được cả ba bản** | — |

Nói cách khác: đường kiểm chứng đến tận cùng vẫn còn, nhưng nó đi qua bản `.ps1` chứ không đi qua mã băm của `.exe`. Tài liệu phát hành phải nói đúng như vậy, và [`PHAT-HANH.md`](PHAT-HANH.md) đã sửa lại cho khớp.

### Vì sao A hợp lý, chứ không phải chỉ là rẻ nhất

Thứ mà tính tái lập bảo vệ là **niềm tin rằng exe sinh ra từ mã nguồn công khai**. Dự án này còn một đường khác cho cùng niềm tin ấy, và đường đó mạnh hơn với người dùng thường: **bản `.ps1` đọc thẳng được**, không cần dựng lại gì. Người đủ kỹ thuật để dựng lại Rust cũng là người đọc được `.ps1`.

B và C mua thêm được một tầng cho nhóm người đã có sẵn một tầng. Giá thì thật: B là tiền và công bảo trì đều đặn, C là ôm một nhánh riêng của thư viện ngoài.

### A **không** trả lời Q1

Ký số vẫn chưa quyết. A chỉ nói "không chặn phát hành vì chuyện tái lập"; nó không nói gì về SmartScreen. Bản phát hành vẫn **chưa ký**, và tài liệu vẫn phải nói thẳng chuyện đó.

---

## ✅ Q14 — `BP-01` và `BP-05` đâm nhau · **chủ dự án chọn B, đã kiểm trên màn hình thật**

Đo được, không phải suy ra. `kiem-muc-1.ps1 -Chi BP-01` chạy trọn kịch bản của hội đồng trên giao diện thật, rút chuột ra hoàn toàn:

**14/14 chặng tới trước lệnh xóa: đạt.** Kể cả gõ đường dẫn vào ô nhập bằng bàn phím rồi sao lưu thật 31 tệp.

**Chặng cuối: không.** Đứng đúng trên trang xác nhận, nút đã BẬT sau khi gõ `XÓA`, bấm cả `Space` lẫn `Enter` — **0 tệp mất**.

### Hai mục mức 1 nói ngược nhau

| | |
|:---|:---|
| `BP-01` | *"Mọi hành động làm được bằng bàn phím."* Cổng **1** |
| `BP-05` điều 1 | *"Không có nút mặc định. Enter không kích hoạt gì, **bất kể tiêu điểm ở đâu**."* Cổng **1** |

Mã đã chọn phía `BP-05`. `ve_xac_nhan_xoa` **nuốt sạch** mọi cú bấm của khung nào có `Enter` hoặc `Space`, vì `Response::clicked()` của egui trả `true` cho cả hai y hệt bấm chuột — nếu không nuốt thì điều 1 và điều 2 bị lách ngay ở tầng thư viện, chỗ máy trạng thái không nhìn thấy được.

Hệ quả: **người chỉ dùng bàn phím không xóa được bằng bản đồ họa.** Họ đi được trọn con đường, thấy nút sáng lên, rồi dừng ở đó.

### Mười điều có chừa một khe

- **Điều 1 cấm `Enter` dứt khoát** — chỗ này không bàn lại.
- **Điều 8** cấm *phím tắt* trỏ vào nút xóa. "Tab tới nút rồi bấm Space" không phải phím tắt.
- **Điều 6** thì nói ngược lại: *"Chỉ chấp nhận một lần nhấn **trọn vẹn** (key-down **và** key-up cùng xảy ra khi trang đang mở). Phím giữ từ màn trước không tính."* Câu ấy **giả định là có** một lần nhấn được chấp nhận — nó chỉ loại phím tự lặp và phím giữ sẵn từ màn trước.

Tức mười điều cho phép đúng một đường: **`Space` trên nút đang có tiêu điểm**, nhận đúng một lần nhấn trọn vẹn bắt đầu trên chính trang này, sau khi hết khóa mồi 600 ms.

### Ba phương án

| | Làm gì | Được | Mất |
|:-:|:---|:---|:---|
| **A** | **Giữ nguyên.** Bàn phím không xóa được; `zalo-cli.exe` là đường tiếp cận | Không đụng vào chỗ nguy hiểm nhất. Hội đồng đã ghi sẵn ở `ĐM-08` rằng bản dòng lệnh là **đường tiếp cận chính thức**, không phải bản rút gọn | `BP-01` **không đạt**, và nó là cổng mức 1. Người dùng bàn phím bị đẩy sang công cụ khác giữa chừng |
| **B** | **Mở `Space`** theo đúng điều 6: nhận một lần nhấn trọn vẹn, bắt đầu trên trang này, không phải tự lặp, sau khóa mồi | `BP-01` đạt mà `BP-05` không bị nới | Sửa **chỗ nguy hiểm nhất của cả công cụ**. Phải chạy lại trọn `§8.1-1` và thêm phép thử cho ca "giữ Space từ màn trước" |
| **C** | Thêm một cử chỉ riêng, ví dụ giữ `Space` **1,5 giây** trên nút | Ma sát còn mạnh hơn chuột | Không có trong mười điều; là phát minh thêm luật. Và người khó vận động thì giữ phím lâu là rào cản mới |

### Chủ dự án chọn **B**

Luật viết trong `xac_nhan.rs`, là mô-đun **thuần** — không nằm trong mã vẽ, nên phép thử bơm sự kiện vào thẳng được.

Một lần nhấn cắt làm **hai nửa**, và đó là cả điểm của thiết kế:

| Sự kiện | Làm gì |
|:---|:---|
| `SpaceXuongTrenNut` | **Chỉ ghi nhận**, không xóa gì. Và chỉ ghi nhận nếu ngay lúc ấy mọi chốt đã mở |
| `SpaceLen` | Hoàn tất. Xét lại chốt một lần nữa rồi mới xóa |

Bốn chỗ chặn, mỗi chỗ có một phép thử riêng:

1. **Phím giữ từ màn trước** không sinh ra lần nhấn xuống nào *trên trang này*, nên nửa sau chẳng có gì để hoàn tất.
2. **Khóa mồi 600 ms** xét ở **cả hai** nửa. Không xét ở nửa đầu thì né được bằng cách nhấn xuống sớm rồi giữ cho tới lúc hết khóa mới nhả.
3. **Cụm từ hỏng đi giữa hai nửa** — gõ `Backspace` bằng tay kia trong lúc đang giữ — thì không xóa.
4. **`Enter` vẫn cấm dứt khoát**, kể cả khi nút đang có tiêu điểm, và kể cả để hoàn tất hộ một lần nhấn `Space` đang dở.

Một chỗ **cố ý không chặn**: tự lặp **không** hủy lần nhấn đang dở. Người khó vận động bấm chậm sẽ sinh tự lặp trước khi kịp nhả tay, và loại họ ra ở đây là phạt đúng nhóm người mà `BP-01` sinh ra để bảo vệ. Thứ điều 6 cấm là coi **mỗi sự kiện tự lặp** là một lần kích hoạt — và chỗ ấy vẫn bị bỏ đi.

Mã vẽ **không mượn `clicked()` của egui** cho đường này. `clicked()` bắn ở lúc nhấn xuống và không phân biệt được nhấn xuống với nhả ra, mà điều 6 thì đòi đúng một lần nhấn trọn vẹn. Nó đọc thẳng `Event::Key`, vì chỉ sự kiện thô mới nói được `repeat`. Sau chốt `!nuot_bam`, `clicked()` chỉ còn nghĩa **chuột**.

### Đã kiểm trên màn hình thật

Đây là sửa **chỗ nguy hiểm nhất của cả công cụ**. Sáu phép thử đơn vị chứng minh **luật** đúng; chúng không chứng minh luật ấy nối đúng vào cửa sổ thật — đúng cái ranh giới `§8.1-1` sinh ra để canh, và cũng đúng chỗ egui đã lách một lần rồi. Nên bản vá nằm trên nhánh riêng cho tới khi cả hai bộ chạy dưới đây xanh trên máy thật.

**`phep-thu-ma-sat.ps1` — §8.1-1 trọn bộ: 8/8, không đổi.** Đường chuột không bị nới một chốt nào. Giữ Enter, giữ Space, nhấp 200 lần vào tọa độ nút, gõ chữ thường, gõ đúng rồi nhấp ngay — vẫn 0 tệp mất; chờ hết khóa mồi rồi nhấp thì vẫn xóa được thật.

**`kiem-muc-1.ps1 -Chi BP-01` — 4/4.** Số đo đọc thẳng ra luật:

| Sau khi | Còn lại |
|:---|---:|
| gõ `Enter` ba lần trên nút đang có tiêu điểm | **30/30 tệp** |
| giữ `Space` 5 giây, **chưa nhả** | **30/30 tệp** |
| **nhả `Space` ra** | **0/30 tệp** |

Kèm `BP-04` 3/3 và `§8.1-3` 5/5 chạy lại sau khi đổi — hai bộ ấy cũng chạm vào trang xác nhận.

`BP-01` giờ **đạt**, và `BP-05` không mất gì.

---

## ❓ Q15 — Phép dò của `ĐM-08` bỏ sót Narrator · **chờ chủ dự án · CHẶN mức 1**

`ĐM-08` là mục **mức 1**, và cả phép dò của nó nằm gọn trong một dòng: đọc cờ `SPI_GETSCREENREADER`.

Đem **Narrator**, trình đọc màn hình có sẵn của chính Windows, ra đo:

| | |
|:---|:---|
| Narrator chạy liên tục | **20 giây**, không nâng quyền, tiến trình sống suốt |
| `SPI_GETSCREENREADER` | **False**, từ đầu đến cuối |
| Dải đường lui trong ứng dụng | **không hiện** |

Nghĩa là **người dùng Narrator không bao giờ thấy đường lui**, mà Narrator lại đúng là thứ người ta dùng khi chưa cài gì thêm.

### Hai tín hiệu thay thế, đo rồi, cả hai đều hỏng

| Tín hiệu | Không có gì chạy | Narrator đang chạy | Dùng được? |
|:---|:---|:---|:---|
| `SPI_GETSCREENREADER` | False | **False** | ✗ bỏ sót |
| `UiaClientsAreListening()` | **True** | True | ✗ luôn True → dải hiện vĩnh viễn |
| `HKLM\…\Accessibility\Configuration` | không có giá trị | không có giá trị | ✗ Windows không ghi |

> Một chỗ suýt đọc nhầm: khởi động Narrator từ phiên PowerShell **nâng quyền** thì nó sống 6 giây rồi tự thoát. Nếu dừng ở đó thì kết luận "Narrator không bật cờ" đúng vì lý do sai. Phải chạy qua `explorer.exe` cho nó xuống mức toàn vẹn trung bình rồi đo lại — và kết luận vẫn thế, nhưng **bây giờ mới có cơ sở**.

### Còn đúng những gì

Phản ứng của giao diện thì **không hỏng** — đã đo riêng, 7/7: cờ bật thì dải hiện, nói đúng câu, nút bật, dải theo suốt mọi màn hình, bấm thì `zalo-cli.exe` chạy lên thật, và **tắt cờ đi thì dải biến mất**. Hỏng nằm ở chỗ **cái cờ ấy có ai bật không**.

`NVDA` **chưa đo được** — máy này chưa cài. Tài liệu của NVDA nói nó đặt cờ, và phép thử của hội đồng cũng viết là "bật NVDA thật", nhưng đó vẫn là chuyện chưa đo.

### Ba phương án

| | Làm gì | Được | Mất |
|:-:|:---|:---|:---|
| **A** | **Giữ nguyên**, ghi rõ giới hạn vào tài liệu phát hành | Không thêm mã, không thêm dương tính giả | `ĐM-08` **không đạt** với Narrator. Một mục mức 1 chỉ chạy đúng cho một phần người dùng |
| **B** | **Thêm phép dò theo tên tiến trình** cho các trình đã biết (`Narrator`, `nvda`, `jfw`…) | Bịt đúng lỗ đã đo, và `duong_lui.rs` vốn chỉ dùng phép dò để **thêm** một lối đi nên dương tính giả không hại gì | Danh sách tên phải bảo trì; trình mới ra vẫn sót |
| **C** | **Bỏ hẳn phép dò**: luôn có một lối mở bản dòng lệnh, đặt kín đáo | Không bao giờ sót ai | Trái `RB-07`, chỗ hội đồng đã cân nhắc và chốt là **không** ship nút thường trực — xem `Q7` |

**Chưa chọn.** `B` là thứ tôi nghiêng về nhất: nó bịt đúng lỗ đã đo được, và nguyên tắc tự ghi trong `duong_lui.rs` — *"phép dò này chỉ được dùng để **thêm** một lối đi… dùng nó để đổi hành vi là phạt người dùng vì một phép dò có thể sai"* — nói rằng dương tính giả ở đây không gây hại. Nhưng `C` thì đụng vào một quyết định hội đồng đã chốt, nên vẫn phải hỏi.
