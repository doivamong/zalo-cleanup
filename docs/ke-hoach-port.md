# Kế hoạch port sang Rust

> Trả lời chín câu ở [`rust-port-brief.md`](rust-port-brief.md) §9 và [`viec-con-lai.md`](viec-con-lai.md) P3-A.
> Lập ngày **01/08/2026** · **204/204 phép thử đạt**
> Mọi con số trong tài liệu này đều đo trên máy thật, không ước lượng.

---

## Khảo sát làm nền cho kế hoạch

**Phân bố 3.098 dòng PowerShell theo mối quan tâm** — đây là cơ sở của cách chia mô-đun:

| Nhóm | Hàm | Dòng | Tỷ lệ |
|:---|---:|---:|---:|
| Quét và băm | 14 | 785 | 25,3% |
| Động tới dữ liệu (xóa · sao lưu · khôi phục) | 5 | 733 | 23,7% |
| Vỏ giao diện | 21 | 671 | 21,7% |
| Hạ tầng | 38 | 528 | 17,0% |
| **Lõi an toàn** | **9** | **186** | **6,0%** |

**Phân bố 190 chỗ gọi `Assert`** — đây là cơ sở của chiến lược kiểm thử:

| Cách lái | Số phép thử | Tỷ lệ |
|:---|---:|---:|
| Gọi hàm trực tiếp / bóc AST | **121** | 64% |
| Lái công cụ bằng chuỗi phím qua stdin | **69** | 36% |

**Số crate, đo bằng `cargo tree`:**

| Cấu hình | Crate |
|:---|---:|
| Lõi, dự phóng lúc lập kế hoạch: `serde_json` `sha2` ~~`walkdir`~~ `csv` `rand` `windows-sys` | **36** |
| Lõi, đo thật sau M2 (`unicode-normalization` + `sha2`) | **13** |
| Cộng `eframe` (giao diện) | **112** |

Con số **36** là dự phóng, đo bằng cách nạp trước cả danh sách. Thực tế tới hết M2 mới dùng **13**, và `walkdir` đã bị loại nên trần cuối cùng thấp hơn 36. Số chính xác sẽ đo lại ở mốc phát hành — dự phóng không phải phép đo.

→ **76 trong 112 crate là do giao diện.** Lõi an toàn có thể soát ở mức 36 crate, và phần kéo theo nhiều nhất lại là phần không đụng tới một byte dữ liệu nào của người dùng. Đây là lý lẽ kiến trúc mạnh nhất của cả kế hoạch.

---

# Q1 · Chia mô-đun

## Nguyên tắc một câu

> **`gui` phụ thuộc `core`. `core` KHÔNG BAO GIỜ phụ thuộc `gui`.** Lõi phải biên dịch và kiểm thử được mà không có một dòng giao diện nào trong cây phụ thuộc.

Đây không phải khẩu hiệu mà là **điều kiện kiểm được bằng máy** — xem cổng M0 ở Q4.

## Cây mô-đun

```
zalo-core/                       36 crate · không có egui
  protect/     vùng bảo vệ, hai mức, cả chiều ngược
  confirm/     cụm từ xác nhận, bỏ dấu thanh
  gate/        Test-KeeperAlive · Test-BackupClean — chốt trước khi xóa
  walk/        duyệt cây, KHÔNG đi xuyên reparse point
  hash/        SHA-256 toàn tệp và chữ ký nhanh, băm song song
  scan/        bốn chế độ quét, trạng thái kết quả quét
  act/         xóa · sao lưu · khôi phục · dọn thư mục rỗng
  sysinfo/     dung lượng trống, loại ổ, tiến trình, nâng quyền, VSS
  store/       catalog.json · settings · profiles · nhật ký · CSV
  lock/        khóa một tiến trình, dùng chung với bản PowerShell

zalo-cli/                        bin headless — xem Q5
zalo-gui/                        bin egui — vỏ, không chứa quyết định xóa
```

## Ranh giới không được vượt

| Quy tắc | Vì sao |
|:---|:---|
| Mọi hàm trong `act/` nhận **kết quả quét đã được `gate/` duyệt**, không tự quyết | Chốt an toàn nằm một chỗ, kiểm được một chỗ |
| `gui/` **không được** gọi thẳng `act/`. Phải đi qua một lớp lệnh của `core` | Ngăn giao diện lách chốt, kể cả do sơ ý |
| `core` không in ra màn hình. Trả về dữ liệu, người gọi tự hiển thị | Cho phép cùng một lõi phục vụ cả CLI lẫn GUI |
| Không dùng `unsafe` ngoài `sysinfo/` | Thu hẹp vùng phải soát tay |

---

# Q2 · Crate

## Lõi — 36 crate, danh sách đóng

| Crate | Dùng làm gì | Không thay được bằng std vì |
|:---|:---|:---|
| `serde` + `serde_json` | Đọc ghi `catalog.json`, bản kê sao lưu | Phải giữ **đúng** định dạng cũ (Q6) |
| `sha2` | SHA-256 | Std không có |
| `csv` | Xuất CSV | Định dạng có ngoặc kép và escape, tự viết là mời lỗi |
| `rand` | Lấy mẫu 50 tệp khi xác minh | Std không có |
| `windows-sys` | Win32 | Std không phơi ra |
| ~~`walkdir`~~ | **ĐÃ LOẠI ở M2** — xem ghi chú | |

> **`walkdir` đã bị loại ở mốc M2.** Đo trên junction thật: nó **không** đi xuyên, nhưng bản tự duyệt bằng `std::fs` cũng vậy và không tốn thêm crate nào. Chốt reparse point cài tường minh theo cờ `FILE_ATTRIBUTE_REPARSE_POINT`, có phép thử riêng cho cả chốt lẫn hành vi duyệt.

## Giao diện — thêm 76 crate

`eframe` / `egui` (đã chốt `QĐ-01`) · `image` cho JPEG và PNG · `jxl-oxide` cho JPEG XL (`R-03`).

## Về lời hứa "0 phụ thuộc"

Không cứu được, và đừng cố. Nhưng có một câu **thật** và mạnh hơn để thay:

> Lõi quyết định xóa gì dùng **36 crate**. Bảy mươi sáu crate còn lại chỉ để vẽ cửa sổ và không đụng tới một byte dữ liệu nào của bạn.

Sáu chỗ phải sửa trong README, đúng vào ngày phát hành: dòng 7 · 58 · 114 · 501 · 678 · 720.

---

# Q3 · Khung giao diện — xác nhận egui

Hội đồng đã chốt (`QĐ-01`) dựa trên số đo thật. Kế hoạch giữ nguyên, không mở lại.

| | egui | iced | Tauri |
|:---|---:|---:|---:|
| Exe đo được | **2,86 MiB** | 6,00 MiB | 4,36 MiB |
| Cần runtime ngoài | không | không | **WebView2 677 MB** |
| AccessKit | có | **không** | — |

Hai việc kèm theo, đã chốt ở `quyet-dinh.md`: **nạp phông hệ thống + nhúng phông dự phòng** (`R-04`), và **`TaskDialogIndirect` không có ô nhập chữ** nên mọi cửa gõ cụm từ phải do egui tự vẽ.

---

# Q4 · Thứ tự làm và cổng từng mốc

Mỗi mốc có một cổng **đo được**. Không đạt cổng thì không đi tiếp.

## Trạng thái các mốc

| Mốc | Trạng thái | Bằng chứng |
|:---|:---|:---|
| **M0** Khung sườn + CI | ✅ **đạt** | `7db50ad` · CI xanh cả hai job · cổng kiến trúc đã đột biến hai nhánh, cả hai đỏ đúng |
| **M1** Lõi an toàn | ✅ **đạt** | `361e250` · đối chiếu **57.572 đầu vào, 0 khác biệt** · 3 đột biến, cả ba đỏ |
| **M2** Duyệt cây và băm | ✅ **đạt** | Đối chiếu **57.351 tệp, 0 lỗi, 0 khác biệt** · junction không đi xuyên · **0,507 s** so với ngưỡng 1,5 s · **Mốc kiểm điểm bắt buộc nằm ngay sau mốc này** |
| **M3** Chế độ headless | ✅ **đạt** | **28/28** phép thử đầu-cuối chỉ đọc, cùng một bộ test lái cả hai bản · 5 đột biến, cả năm đều đỏ · lõi 16+61 phép thử đơn vị |
| **M4** Động tới dữ liệu | ✅ **đạt** | **67/67** phép thử đầu-cuối (kể cả `-Full`) · **19/19** phép liên thông hai chiều, so SHA-256 từng tệp · 8 đột biến, cả tám đều đỏ |
| **M5** Giao diện | ◐ **gần xong** | exe **3,61 MiB** · **68** phép thử giao diện · **§8.1-1 đạt 8/8** · **`kiem-muc-1.ps1` đã đo 8 trong 9 mục mức 1 trên giao diện thật, bắt được 3 lỗi và sửa cả ba** · `Q14` chốt phương án B nên `BP-01` **đạt** · còn `ĐM-08` cần NVDA thật và ba mục greyscale cần **ba người thử** |
| **M6** Phát hành | ◐ **sẵn sàng, CỐ Ý CHƯA CÔNG BỐ** | Máy móc phát hành đã xong và đã kiểm: CI dựng, mã băm khớp cả 5 tệp, hai exe chạy được từ đúng bản tải về. **Chưa gắn thẻ** vì mục tiếp cận mức 1 của M5 chưa xong hết, mà luật của hội đồng ghi mức 1 chặn hẳn bản phát hành. Cổng tái lập không đạt → chốt **phương án A** |

## M0 · Khung sườn — ✅ đạt

Dựng workspace, ba crate rỗng, `rust-toolchain.toml` ghim phiên bản.

> **Cổng M0:** `cargo test -p zalo-core` chạy được, **và** `cargo tree -p zalo-core` không chứa `eframe`, `egui`, `winit`. Đây là cổng kiến trúc, chạy trong CI mãi mãi về sau — nó biến "lõi không phụ thuộc giao diện" từ lời hứa thành thứ máy kiểm.

## M1 · Lõi an toàn — ✅ đạt

`protect/` `confirm/` `gate/` — 186 dòng, phần nhỏ nhất và nguy hiểm nhất.

> **Cổng M1:** ① port **toàn bộ phép thử đơn vị thuộc lõi an toàn** ② chạy lại bộ so sánh **≥ 57.144 đầu vào** mà phiên trước đã dùng cho `Test-Protected`, lần này so PowerShell với Rust → **0 khác biệt** ③ mọi đột biến đã dùng trong phiên trước đều làm bộ test Rust đỏ.

**Sửa một chỗ viết sai của chính kế hoạch này.** Bản đầu ghi cổng ① là "port đủ **121 phép thử đơn vị**". Sai phạm vi: 121 là số phép thử đơn vị của **toàn bộ công cụ**, mà phần lớn trong đó kiểm những hàm thuộc M2 và M4 — chưa tồn tại ở M1. Đòi chúng ở đây là đòi một thứ không thể đạt, và tuyên bố "cổng đạt" dựa trên một spec sai thì vô nghĩa.

Con số đúng cho riêng lõi an toàn, đếm trên mã nguồn:

| | |
|:---|---:|
| Chỗ gọi `Assert` phía PowerShell chạm thẳng bảy hàm của lõi an toàn | **45** |
| Hàm `#[test]` phía Rust phủ cùng bề mặt đó | **22** |

Hai con số không bằng nhau và **không cần bằng nhau**: phép thử Rust viết theo bảng, một hàm `#[test]` phủ cả bảng mười ca. Thứ chứng minh tương đương không phải số lượng phép thử mà là **bộ đối chiếu song song** ở cổng ②.

Chọn mốc này làm mốc đầu vì nó nhỏ, rủi ro cao nhất, và **đã có sẵn bộ so sánh chứng minh được cách làm này hiệu quả**.

## M2 · Duyệt cây và băm — ✅ đạt

`walk/` `hash/` `scan/`.

> **Cổng M2:** ① quét cùng một cây thật, hai bản ra **cùng tập tệp và cùng số lỗi** ② dựng junction thật, hai bản **đều không đi xuyên** ③ quét 52.748 tệp **≤ 1,5 giây** (bản PowerShell: 5,2 giây).

Cổng ③ là chỗ kiểm chứng lợi ích hiệu năng. Không đạt thì lợi ích chính không có và phải xem lại.

**Kết quả đo:**

| Cổng | Yêu cầu | Đo được |
|:---|:---|:---|
| ① cùng tập tệp, cùng số lỗi | 0 khác biệt | **57.351 tệp · 0 lỗi · 0 khác biệt** |
| ② junction | không đi xuyên | **không đi xuyên**, có phép thử riêng cho cả chốt lẫn hành vi |
| ③ tốc độ | ≤ 1,5 s | **0,507 s** — nhanh hơn bản PowerShell **10,2 lần** |

Kèm hai phép đối chiếu ngoài cổng: **băm** 70 tệp chạm cả nhánh `FULL:` lẫn `Q:`, 0 khác biệt; **phần mở rộng** 310 tên tệp, 0 khác biệt.

### Hai chỗ số đo lật lại quyết định của kế hoạch

**Loại `walkdir`.** Kế hoạch để nó ở diện *ứng viên chưa chốt*, chờ đúng một phép đo. Đã đo trên junction thật: `walkdir` **không** đi xuyên — nhưng bản tự duyệt cũng vậy, mà **không tốn thêm crate nào**. Nên tự viết, và `walkdir` bị loại khỏi danh sách phụ thuộc ở Q2.

**Phần mở rộng phải theo luật .NET, không theo luật Rust.** Đây là bẫy kế hoạch không lường:

| Tên tệp | .NET `Path.GetExtension` | Rust `Path::extension` |
|:---|:---|:---|
| `.rescache` | `".rescache"` | `None` |
| `a.` | `""` | `Some("")` |

Dữ liệu Zalo thật có **4.226 tệp `.rescache`**, và công cụ dùng đúng phần mở rộng đó để loại chúng khỏi lượt quét. Dùng thẳng `Path::extension()` là phân loại sai 4.226 tệp — không phải khác biệt lý thuyết. Đã cài đúng luật .NET và đối chiếu 310 tên tệp thật.

## M3 · Chế độ headless — ✅ đạt · mốc quyết định của cả kế hoạch

`zalo-cli` nhận **đúng giao thức phím qua stdin** như bản PowerShell.

> **Cổng M3:** chạy được **69 phép thử E2E hiện có, KHÔNG SỬA MỘT KÝ TỰ**, chỉ đổi đường dẫn công cụ, và cho cùng kết quả.

Đạt cổng này thì toàn bộ bộ test thành **hợp đồng sống cho cả hai bản**, và bộ so sánh song song có sẵn công cụ. Không đạt thì chiến lược kiểm chứng sụp và phải thiết kế lại trước khi viết thêm dòng nào.

### Cổng này kế hoạch viết sai phạm vi — lần thứ hai

Đếm lại trên mã nguồn: **67** phép thử đầu-cuối, không phải 69. Nhưng sai lớn hơn nằm ở chỗ khác — **39 trong số đó đòi công cụ XÓA, SAO LƯU hoặc KHÔI PHỤC**, tức đúng những việc mà chính kế hoạch xếp vào **M4**. Đòi đủ 67 phép ở M3 là đòi một thứ không thể đạt.

Đây là lỗi cùng loại với lỗi đã bắt ở cổng M1, và xử lý cũng vậy: sửa spec cho đúng phạm vi rồi ghi lại vì sao, chứ không tuyên bố đạt một cổng viết sai.

> **Cổng M3, đã sửa:** chạy các phép thử đầu-cuối **KHÔNG xóa tệp nào** — **28 phép** — bằng đúng tệp `ZaloCleanup.Tests.ps1` ấy, không sửa một ký tự nào trong chính các phép thử, và cho cùng kết quả với bản PowerShell.

Phân loại theo **hành vi thật** chứ không theo phím: lượt khử trùng lặp và lượt dọn cache có xóa tệp dù không hề gõ `XÓA`, còn phím `B` lại chỉ là báo cáo vùng bảo vệ.

**Kết quả:** 28/28 đạt. Bộ chạy là [`rust/tools/cong-m3.ps1`](../rust/tools/cong-m3.ps1), và CI chạy nó sau mỗi commit.

### Điểm hoán đổi: một biến, và một chỗ phải sửa

`$tool` trong bộ test gánh **hai vai** cùng lúc — thứ bị lái, và mã nguồn PowerShell mà hơn một trăm phép thử đem ra soi bằng AST. Trỏ cả hai vai sang một tệp `.exe` là 135 phép thử chết ngay.

Nên tách: `$tool` giữ nguyên nghĩa mã nguồn, `$toolChay` là công cụ được lái và nhận biến môi trường `ZALO_TOOL`. Zero phép thử đầu-cuối phải sửa — **trừ đúng một chỗ**: phép thử vùng miền vi-VN lái công cụ qua một script phụ thay vì qua `Invoke-Tool`, nên phải đổi một định danh ở đó. Để nguyên thì khi lái bản Rust nó vẫn lặng lẽ chạy bản PowerShell rồi báo xanh, và một phép thử xanh vì chạy nhầm công cụ còn tệ hơn một phép thử đỏ.

### Đột biến tìm ra hai lỗ mà cổng không bịt được

Cổng đạt ngay lượt đầu — con số phải nghi ngờ chứ không mừng. Năm đột biến, và **hai trong số đó cổng vẫn xanh**:

| Đột biến | Cổng M3 |
|:---|:---|
| Hiện cả mốc thời gian rỗng | đỏ |
| Khôi phục không mô tả nội dung bản sao lưu | đỏ |
| Nhập sai thì âm thầm chọn tất cả — phá **nguyên tắc bất biến số 3** | **vẫn xanh** |
| Gỡ hẳn chốt vùng bảo vệ khỏi vòng quét | **vẫn xanh** |
| Bản trùng lặp bị đòi gõ `XÓA` như dữ liệu thật | **vẫn xanh** |

Lý do đều giống nhau: phép thử đầu-cuối tương ứng hoặc chỉ kiểm **câu chữ in ra** chứ không kiểm trạng thái, hoặc nằm trong lượt có xóa tệp nên thuộc M4.

Đã bịt bằng cách tách ba quyết định ấy thành hàm thuần rồi kiểm thẳng: `phan_tich_chon_thu_muc`, `xet_tep`, `muc_xac_nhan`. Đo lại: cả năm đột biến đều đỏ.

Đây đúng loại lỗ đã cắn dự án này ở `Test-KeeperAlive` — chốt có mặt trong mã, nhưng không có gì chứng minh nó được **gọi**.

### Một khác biệt cố ý giữa hai bản

`'{0:N2}'` của PowerShell đổi theo vùng miền (`1,234.56` ở en-US, `1.234,56` ở vi-VN); Rust không có khái niệm vùng miền nên luôn in kiểu en-US. Chỉ khác **dấu phân cách**, không khác con số, và không phép quyết định nào đọc lại chuỗi đã định dạng. Ghi ra đây thay vì để đó — một khác biệt không được viết xuống là một khác biệt sẽ bị phát hiện lại vào lúc bất tiện nhất.

## M4 · Động tới dữ liệu — ✅ đạt

`act/` — xóa, sao lưu, khôi phục, dọn thư mục rỗng.

> **Cổng M4:** ① bản sao lưu do **Rust** tạo phải khôi phục được bằng **PowerShell**, và ngược lại ② so SHA-256 từng tệp, không so số lượng ③ nhật ký hai bản khớp nhau về số dòng và trạng thái.

**Kết quả:** [`rust/tools/cong-lien-thong.ps1`](../rust/tools/cong-lien-thong.ps1) — **19/19 đạt**, cả hai chiều, mỗi chiều 6 tệp gồm thư mục lồng nhau, tên có dấu, tệp rỗng và tệp vượt ngưỡng 128 KB của chữ ký nhanh. Hai nhật ký xóa khớp cả số dòng lẫn số lượng từng trạng thái.

### Cổng đối chiếu song song mở rộng lên toàn bộ

Ở M3, cổng chỉ đòi 28 phép thử đầu-cuối không xóa tệp. Từ M4 nó đòi **cả 67**, và chạy kèm `-Full` — bốn phép thử chậm trong đó chính là bốn đường nguy hiểm nhất của mốc này. **67/67 đạt.**

### Một phép thử phải sửa, và vì sao đó không phải chiều bản Rust

Phép thử `G4 tệp biến mất giữa chừng` cho một tiến trình phá hoại **ngủ 5 giây** rồi mới xóa tệp, để dựng khe hở giữa lúc quét và lúc xóa. Tiền đề ngầm là công cụ chạy lâu hơn 5 giây — đúng với bản PowerShell.

Đo tận nơi: bản Rust xóa xong **20.000 tệp trong 3,58 giây**, tức xong trước khi tiến trình phá hoại kịp ra tay. Phép thử đỏ mà chẳng có lỗi nào ở công cụ cả.

Đã đổi mốc đồng bộ từ **đồng hồ** sang **tệp nhật ký**: cả hai bản đều tạo `daxoa_*.log` ngay khi bắt đầu xóa, và đó là hợp đồng chung chứ không phải chi tiết cài đặt của riêng bản nào. Sửa này làm phép thử đáng tin hơn cho **cả hai** bản, và không nới lỏng điều kiện nào.

### Tám cửa, tám đột biến, tám lần đỏ

| Đột biến | Bị bắt bởi |
|:---|:---|
| Gỡ chốt vùng bảo vệ khỏi vòng xóa | `vung_bao_ve_chan_ngay_trong_vong_xoa` |
| Gỡ chốt bản giữ lại còn sống | `mat_ban_giu_lai_thi_khong_xoa` |
| Đếm tệp đã biến mất là đã xóa | `tep_bien_mat_truoc_khi_xoa_khong_duoc_dem_la_da_xoa` |
| Dùng cỡ ghi lúc quét thay vì cỡ thật lúc xóa | `dung_co_that_luc_xoa_chu_khong_dung_co_luc_quet` |
| Dọn thư mục rỗng bằng xóa **đệ quy** | `xoa_thu_muc_tu_choi_thu_muc_con_tep_ben_trong` |
| Bỏ chốt "tệp không nằm dưới gốc quét" khi sao lưu | `tep_ngoai_goc_quet_bi_chan_chu_khong_ghi_ra_ngoai` |
| Sao lưu thiếu tệp vẫn chấm là sạch | `sao_luu_sach_dung_ca_tam_ca` |
| Ổ đích hết chỗ vẫn chấm là sạch | `sao_luu_sach_dung_ca_tam_ca` |

Lượt đột biến đầu để lọt **một** cửa: đổi `remove_dir` thành `remove_dir_all` ngay trong vòng dọn thư mục mà không phép thử nào đỏ, vì mọi thư mục đưa tới đó đều đã rỗng sẵn nên hai hàm cho cùng kết quả. Khe hở thật nằm giữa lúc kết luận rỗng và lúc hạ tay. Đã tách thành `xoa_thu_muc_neu_rong` rồi hỏi thẳng: *đưa cho nó một thư mục CÓ tệp thì sao?*

### Một chỗ cố ý khác bản PowerShell

Bản PowerShell hỏi rồi **tự đóng Zalo** khi lượt dọn chạm vào dữ liệu của nó. Bản Rust **chỉ báo rồi dừng**, để người dùng tự đóng. Hai lý do: giết một ứng dụng nhắn tin đang chạy có thể làm mất tin nhắn chưa gửi, và nhánh ấy không có phép thử nào canh — sandbox của bộ test nằm trong `%TEMP%` nên không bao giờ chạm tới. Viết mã hủy tiến trình mà không có phép thử là đúng thứ dự án này đã thề không làm.

## M5 · Giao diện — ◐ phần máy đạt, phần cần người thật còn nguyên

Theo `ui-ux-council.md`.

> **Cổng M5:** ① danh mục tiếp cận **mức 1** của hội đồng đạt hết ② 36 đường tấn công đã bịt được kiểm lại tay ③ hai chốt xem trước và gõ cụm từ có phép thử tự động.

### Kết quả

| | |
|:---|---:|
| Phép thử đơn vị của giao diện | **60** |
| Tổng phép thử đơn vị (lõi + vỏ) | **149** |
| Kích thước `zalo-gui.exe` | **3,61 MiB** (kể cả phông nhúng 756 KB và bộ giải mã JPEG XL) |
| Crate riêng cho giao diện | **128** |

Cổng kiến trúc M0 vẫn xanh: lõi 17 crate, không dính một dòng giao diện nào. Trước khi thêm bộ giải mã JPEG XL, exe là **2,64 MiB** — nhỏ hơn dự phóng 2,86 MiB của hội đồng.

### Ma sát được dựng lại thành ba mô-đun THUẦN, không nằm trong mã vẽ

An toàn của bản dòng lệnh đến phần lớn từ ma sát. Giao diện đồ họa xóa sạch ma sát đó — mọi thứ cách nhau một cú nhấp. Nên ma sát được dựng lại có chủ đích, và **mỗi mảnh là một mô-đun kiểm được không cần vẽ**: không thể "giữ phím Enter năm giây" trong một hàm `#[test]`, nhưng bơm năm nghìn sự kiện `Enter` vào một máy trạng thái thì được.

| Mô-đun | Canh điều gì |
|:---|:---|
| `xac_nhan` | Mười điều của `BP-05`: Enter không kích hoạt, khóa mồi 600 ms tính lại **mỗi lần** nút bật, chặn dán, bỏ phím tự lặp, bấm rồi không nhận thêm |
| `xem_truoc` | Không mở danh sách tệp sắp mất thì nút xóa **không bật**. Quét lại là chốt đóng lại |
| `muc_rui_ro` | Ba lớp mã hóa của `MAU-01`: chữ, ký hiệu, rồi mới tới màu. Kèm phép đo tương phản WCAG |

### Ba lỗ thật, tìm ra bằng cách hỏi máy chứ không bằng cách nhìn màn hình

**Phông thiếu glyph `⛨`.** Thiết kế của hội đồng dùng ký hiệu ấy làm huy hiệu vùng bảo vệ. Phông nhúng không có nó, nên nó sẽ hiện thành **ô vuông rỗng** — mà một huy hiệu an toàn hiện thành ô vuông rỗng còn tệ hơn không có huy hiệu nào. Đã thay bằng `⊘`, và gom mọi ký hiệu vào một bảng có phép thử quét toàn bộ, để lỗi loại này không quay lại.

**egui cho phép kích hoạt nút bằng Enter và Space.** `Response::clicked()` trả `true` y như bấm chuột, tức điều 1 và điều 2 của `BP-05` bị lách **ngay ở tầng thư viện** — máy trạng thái không nhìn thấy được. Đã bịt bằng cách nuốt sạch mọi cú bấm của khung nào có Enter hoặc Space.

**Lõi chưa có đường hủy.** `BP-08` đòi Esc dừng được lượt xóa đang chạy và nhật ký ghi "đã hủy giữa chừng". Đã thêm cờ hủy vào `act::xoa`, kèm phép thử: dừng ngay, `hoan_tat = false`, và nhật ký ghi đúng — thiếu vế cuối là người dùng bấm Esc rồi mở nhật ký thấy `hoàn tất=True`.

### Chín mục MỨC 1 — tám trong số đó hóa ra ĐO ĐƯỢC BẰNG MÁY

Chín mục này từng mang nhãn *"không cỗ máy nào kết luận được"*. Lý do ấy **sai một nửa**, và chỉ cần thử mới biết.

egui bật `accesskit`, mà AccessKit phơi **toàn bộ cây widget** ra UI Automation của Windows — kèm tên, vai trò, trạng thái bật/tắt và khung bao theo tọa độ màn hình. Tức là hỏi được bằng máy đúng những câu tưởng phải nhìn bằng mắt:

| Câu hỏi | Hỏi ai |
|:---|:---|
| Tiêu điểm đang nằm ở phần tử nào | `AutomationElement::FocusedElement` |
| Nút Xóa đã bật chưa | `IsEnabled` |
| Phần tử nào tràn khỏi vùng vẽ | `BoundingRectangle` so với `GetClientRect` |
| Hai nút cách nhau bao nhiêu dip | cùng nguồn |

[`rust/tools/kiem-muc-1.ps1`](../rust/tools/kiem-muc-1.ps1) lái giao diện **thật** trong hộp cát `%TEMP%` và đo hết. Nó bắt được **hai lỗi mà cả mốc M5 không bắt được**.

#### Lỗi ① — `DPI-04`: một phần ba số ảnh xem trước nằm ngoài màn hình

Đo ở **1092×614 dip**, đúng cỡ vùng vẽ của màn 1366×768 @125%: **4 trong 12 ô ảnh** nằm ngoài mép phải, ô thứ tám bị cắt đôi. Lưới nằm trong một `ScrollArea::horizontal`, nên về kỹ thuật là "cuộn tới được" — nhưng `DPI-04` cấm đúng chuyện đó, và ô ảnh **không nhận tiêu điểm**, nên bằng bàn phím thì không cuộn tới được thật.

Nặng hơn phần chữ nghĩa: lưới ấy tồn tại để người dùng **nhìn thấy** thứ sắp mất. Giấu một phần ba số ảnh sau một cú cuộn ngang là bỏ đi đúng phần ma sát nó sinh ra để tạo — người ta đếm tám ô rồi tưởng đã nhìn hết.

Đã sửa: chia hàng tường minh theo bề rộng còn trống. Không dùng `horizontal_wrapped` — đã thử và đo, egui chỉ ngắt dòng theo **widget rời**, không theo bố cục con, mà mỗi ô là một `ui.vertical` lồng trong. Sau khi sửa: đủ 12 ô trên hai hàng, **0 phần tử tràn trên cả 8 màn hình**.

#### Lỗi ② — `BP-01`: nút đứng sau một nút đang tắt biến mất khỏi bàn phím

Đo trên màn sao lưu:

| "Bắt đầu sao lưu" | Vòng Tab đo được |
|:---|:---|
| đang **TẮT** | ô nhập → radio → radio → *(rỗng)* → ô nhập |
| đã **BẬT** | ô nhập → radio → radio → Bắt đầu → ← Quay lại |

Nút `← Quay lại` **không bao giờ nhận được tiêu điểm** ở dòng trên. Nguyên nhân đọc thẳng trong mã nguồn egui 0.29, `Context::create_widget`:

```rust
if allow_focus && w.sense.focusable {
    ctx.memory.interested_in_focus(w.id);      // chạy cho CẢ widget đang tắt
}
if allow_focus && (!w.enabled || …) {
    mem.surrender_focus(w.id);                 // rồi lấy lại ngay
}
```

Widget đang tắt **giành** tiêu điểm ở bước một — vì `give_to_next` đang bật sau khi người dùng gõ Tab — rồi bị tước ở bước hai. Chặng Tab tiêu vào khoảng giữa.

Dính **ba màn**, và chỗ nặng nhất không phải màn sao lưu mà là **trang chủ**: `Lấy lại dung lượng ổ đĩa` tắt khi máy chưa cài Zalo, và cả hai nút sau nó chết theo — toàn bộ trang chủ không dùng được bằng bàn phím.

Đã vá bằng `nut_co_the_tat`: nút đang tắt khai thẳng là **không nhận tiêu điểm** (`Sense::hover()`). Nó vốn không giữ nổi tiêu điểm nên không mất gì, mà chuỗi Tab thì liền lại. Kèm **hai** phép thử chạy vòng lặp egui thật và bơm phím Tab vào: một phép canh bản vá, một phép **ghim lỗ của egui** để ngày egui tự sửa thì nó đỏ lên và người nâng phiên bản buộc phải bỏ bản vá đi thay vì để nó nằm lại như một câu bùa.

#### Kết quả đo được, từng mục

| Mục | Kết quả |
|:---|:---|
| **`DPI-01`** *(mức 1, **lọt sổ** — không có trong danh sách chín mục, cũng không có trong cổng M5)* | ✅ tiến trình chạy `PER_MONITOR_AWARE_V2`, đo bằng `AreDpiAwarenessContextsEqual`. Kho mã không có tệp manifest nào — winit đặt lúc chạy |
| **`DPI-04`** | ❌→✅ 8/8 màn hình, 0 phần tử tràn ở 1092×614 dip |
| **`DPI-08`** | ✅ không cửa sổ nào ngoài cửa sổ chính phơi ra widget · ba widget của trang xác nhận nằm gọn trong vùng vẽ · cùng màn hình với cửa sổ cha |
| **`BP-04`** | ✅ 30 chặng Tab rơi đúng vào 2 phần tử của trang xác nhận, không chặng nào thoát ra · không widget nào của màn trước còn sống |
| **`§8.1-3`** | ✅ 5/5 |
| **`MAU-01`** | ✅ 4/4 phần máy. Ký hiệu ↔ câu chữ khớp một-một; trên **ảnh đã bỏ màu**, mỗi mức vẽ ra một hình riêng, và hai dòng cùng mức vẽ ra cùng một hình |
| **`MAU-09`** | ✅ 4/4 phần máy. Khác chữ, khác biểu tượng, cách nhau **48 dip**, `Hủy` đứng trước theo thứ tự đọc |
| **`ĐM-08`** | ◐ **phản ứng đúng 7/7, nhưng phép dò bỏ sót Narrator.** Dải hiện ra, nói đúng câu, nút bật, dải theo suốt mọi màn hình, bấm thì `zalo-cli.exe` chạy lên thật, tắt cờ thì dải biến mất. Nhưng **Narrator chạy 20 giây mà cờ không lên** → `Q15` |
| **`BP-01`** | ✅ **4/4 sau khi chốt `Q14` phương án B.** Kịch bản đi trọn bằng bàn phím, kể cả lệnh xóa · `Enter` vẫn không kích hoạt · giữ `Space` chưa nhả thì chưa xóa · nhả ra thì xóa thật |

##### Bốn kiểu gõ của §8.1-3, vì nó là mục dễ tưởng đã kiểm nhất

Unikey không "gõ tiếng Việt"; ở bước cuối nó đẩy vào ứng dụng một chuỗi `Backspace` cộng ký tự đã ghép. Bộ chạy gửi **đúng chuỗi ấy** bằng `SendInput` với `KEYEVENTF_UNICODE`:

| Cách gõ | Chuỗi thật đẩy vào | Nút xóa |
|:---|:---|:---|
| kiểu **mới**, dấu trên A | `X` `O` `A` `⌫` `Á` | **bật** |
| kiểu **cũ**, dấu trên O | `X` `O` `A` `⌫⌫` `Ó` `A` | **bật** |
| không dấu | `X` `O` `A` | **bật** |
| tổ hợp NFD | `X` `O` `A` `U+0301` | **bật** |
| chữ thường, kiểu mới | `x` `o` `a` `⌫` `á` | **tắt** ✓ đúng `TV-01` |

#### Lỗi ③ — sao lưu xong là mất luôn lượt quét

Tìm ra ở lượt chạy `BP-01` đầu tiên, khi kịch bản bàn phím đi tới đây rồi **tắc**.

Sao lưu xong thì màn `Kết quả` hiện ra, và nó nói thẳng: *"Sao lưu sạch. **Đã mở khóa bước xóa.**"* Nhưng nút duy nhất trên màn ấy là `Về trang chủ` — mà trang chủ **không có đường nào quay lại kết quả quét**. `ManHinh::KetQuaQuet` chỉ đặt được từ lúc quét xong, từ `XemDanhSach`, từ `SaoLuu`, hoặc từ nút Hủy của trang xác nhận. Không có đường nào từ `KetQua`.

Nghĩa là: người dùng làm **đúng thứ giao diện khuyên họ làm** — sao lưu trước khi xóa — rồi mất luôn lượt quét vừa làm.

Nặng hơn phần phiền toái: `sao_luu_gan_nhat` khóa theo `dau_quet`, nên quét lại là bản sao lưu vừa tạo **cũng hết hiệu lực** và phải sao lưu lần nữa. Một câu *"đã mở khóa bước xóa"* dẫn tới chỗ không có bước xóa thì tệ hơn là không nói gì.

Đã sửa: màn `Kết quả` mọc thêm nút `← Quay lại kết quả quét`, và chỉ hiện khi lượt quét còn trong bộ nhớ.

#### `BP-01` — đã chạy trọn, và kết quả là hai mục mức 1 đâm nhau

Chạy trên máy đã đóng hết ứng dụng khác, rút chuột ra hoàn toàn — bộ chạy **không có một hàm nào** gọi tới chuột, và sự thiếu vắng ấy là một phần của phép thử.

**14/14 chặng tới trước lệnh xóa: ĐẠT.**

| Chặng | |
|:---|:---|
| chọn nguồn → `Lấy lại dung lượng ổ đĩa` | ✓ |
| quét → `Kết quả quét · DỮ LIỆU ZALO` | ✓ |
| xem danh sách → `Những tệp sắp mất` | ✓ |
| Esc quay về kết quả quét (`BP-06`) | ✓ |
| vào màn sao lưu, Tab tới ô nhập | ✓ |
| **gõ đường dẫn bằng bàn phím**, đọc lại đúng từng ký tự | ✓ |
| **sao lưu thật 31 tệp** | ✓ |
| quay lại kết quả quét | ✓ *(chặng này chỉ đi được sau khi vá lỗi ③)* |
| mở trang xác nhận, gõ `XÓA`, nút xóa **bật lên** | ✓ |

**Chặng cuối, ở lượt đo đầu: KHÔNG.** Đứng đúng trên trang xác nhận, nút đã BẬT, gõ cả `Space` lẫn `Enter` — **0 tệp mất**.

Đây không phải lỗi. Đây là **`BP-05` đang làm đúng việc của nó**, và hai mục mức 1 đâm thẳng vào nhau:

| | |
|:---|:---|
| `BP-01` | *"Mọi hành động làm được bằng bàn phím"* |
| `BP-05` điều 1 | *"Không có nút mặc định. Enter không kích hoạt gì, **bất kể tiêu điểm ở đâu**."* |

Mã đã chọn phía `BP-05`: `ve_xac_nhan_xoa` **nuốt sạch** mọi cú bấm của khung nào có `Enter` hoặc `Space`, vì `Response::clicked()` của egui trả `true` cho cả hai y như bấm chuột. Hệ quả: **không có đường nào từ bàn phím tới lệnh xóa**, kể cả một lần nhấn hoàn toàn có chủ ý.

Đọc kỹ mười điều thì có chỗ hở đáng cân nhắc:

- Điều 1 cấm `Enter` **dứt khoát** — không bàn lại.
- Điều 8 cấm **phím tắt** trỏ vào nút xóa — nhưng "Tab tới nút rồi bấm Space" không phải phím tắt.
- Điều 6 lại nói ngược: *"Chỉ chấp nhận một lần nhấn **trọn vẹn** (key-down **và** key-up cùng xảy ra khi trang đang mở). Phím giữ từ màn trước không tính."* Câu ấy **giả định là có** một lần nhấn được chấp nhận, và chỉ loại phím tự lặp với phím giữ từ màn trước.

Tức mười điều **cho phép** một đường: `Space` trên nút đang có tiêu điểm, nhận đúng một lần nhấn trọn vẹn bắt đầu trên chính trang này, sau khi hết khóa mồi. Làm được thì `BP-01` đạt mà `BP-05` không bị nới.

**Chủ dự án chọn phương án B, và đã làm.** Chi tiết luật ở `Q14` trong [`quyet-dinh.md`](quyet-dinh.md). Đo lại trên máy thật sau khi sửa:

| Bộ chạy | |
|:---|:---|
| `phep-thu-ma-sat.ps1` — §8.1-1 trọn bộ | **8/8, không đổi.** Đường chuột không mất một chốt nào |
| `kiem-muc-1.ps1 -Chi BP-01` | **4/4.** `Enter` ba lần → 30/30 tệp · giữ `Space` 5 giây chưa nhả → 30/30 · nhả ra → **0/30** |
| `BP-04` · `§8.1-3` chạy lại | 3/3 và 5/5 — hai bộ này cũng chạm trang xác nhận |

`ĐM-08` vẫn đứng nguyên như hội đồng ghi: *"Bản dòng lệnh là đường tiếp cận chính thức cho người khiếm thị, không phải tác dụng phụ tình cờ."* Khác là bây giờ nó là **một lựa chọn**, không còn là **lối duy nhất**.

#### Một chuyện chưa truy tới cùng, ghi lại chứ không lấp

Trong lúc dựng bộ chạy, ứng dụng **tự thoát** vài lần giữa chừng. Đo lặp 5 lượt mỗi nhánh, cùng kịch bản:

| Bộ chạy gọi `SetFocus` lên cửa sổ | Ứng dụng tự thoát |
|:---|---:|
| có | **1/5** |
| không | **0/5** |

Thoát **sạch** — không bản ghi sự cố nào trong nhật ký Ứng dụng của Windows — nên là vòng lặp sự kiện kết thúc chứ không phải sập. Bộ chạy đã bỏ `SetFocus` (nó vốn thừa: `SetForegroundWindow` cộng `SetActiveWindow` đã đủ đưa cửa sổ lên). Nhưng **cơ chế thì chưa truy ra**, và người dùng thật vẫn có thể gặp: trình đọc màn hình, tiện ích "luôn nằm trên", điều khiển từ xa đều gọi những hàm ấy. Mẫu 5 lượt quá nhỏ để kết luận. Đây là việc còn nợ, không phải việc đã xong.

### Ba việc từng nợ, giờ đã làm xong

| Việc | Kết quả |
|:---|:---|
| **Bộ giải mã JPEG XL** | Đã có. `.jxl` chiếm **46,4%** dữ liệu Zalo thật, và phép thử giải mã **tệp `.jxl` thật của Zalo** chứ không phải tệp dựng máy móc. Giá: exe từ 2,64 lên **3,61 MiB**, thêm 16 crate |
| **Màn sao lưu và khôi phục** | Đã có, chạy ngoài luồng vẽ. Chốt sao lưu sạch gọi thẳng `gate::sao_luu_sach` của lõi chứ không viết lại |
| **`ĐM-08`** | Đã có. Dải thông báo hiện trên **mọi** màn hình, dò lại theo nhịp vì người dùng có thể bật trình đọc màn hình giữa chừng |

Kèm theo, ảnh xem trước thật: mười hai ảnh lấy ngẫu nhiên, giải mã ngoài luồng vẽ, nhận dạng bằng magic byte. Tệp không xem trước được hiện ô `?` và **vẫn nằm trong danh sách** — giấu đi là người dùng xóa một thứ họ chưa từng nhìn thấy mà lại tưởng đã xem hết.

### Một lỗ nữa, chỉ lộ ra khi chạy thật

Màn hình hiện `? Xong.` thay vì `✓ Xong.`. Đo tận nơi: **Segoe UI phủ đủ 134 chữ cái tiếng Việt nhưng thiếu bốn trên tám ký hiệu** của bảng — `⊘ ⚠ ✓ ✖`.

Phép thử phủ glyph của tôi chỉ hỏi **phông nhúng**, không hỏi phông hệ thống đang thật sự dùng, nên nó xanh trong khi màn hình hỏng. Sửa bằng **chuỗi phông**: hệ thống cho chữ, phông nhúng lấp glyph thiếu. Phép thử mới hỏi **cả chuỗi gộp lại** chứ không hỏi từng phông một.

Đây là lần thứ ba trong mốc M5 một chốt an toàn hóa ra chưa từng được chứng minh — và là lần duy nhất phải mở ứng dụng lên mới thấy.

### §8.1-1 đã chạy trên giao diện thật — 8/8

Phép thử ma sát của hội đồng, chạy bằng [`rust/tools/phep-thu-ma-sat.ps1`](../rust/tools/phep-thu-ma-sat.ps1) trên hộp cát 30 tệp trong `%TEMP%`. Nó lái chuột và bàn phím thật, nên không chạy được trên máy chủ CI.

| Phép thử | Kết quả |
|:---|:---|
| Giữ **Enter** 5 giây ở màn kết quả quét | 0 tệp biến mất |
| Giữ **Enter** 5 giây trên trang xác nhận | 0 tệp biến mất |
| Giữ **Space** 5 giây trên trang xác nhận | 0 tệp biến mất |
| **Nhấp 200 lần** vào đúng tọa độ nút Xóa, không gõ gì | 0 tệp biến mất |
| Gõ `xoa` **chữ thường** rồi nhấp | 0 tệp biến mất (`TV-01`) |
| Gõ đúng cụm từ rồi nhấp **ngay lập tức** | 0 tệp biến mất (khóa mồi 600 ms) |
| **Kiểm ngược:** chờ hết khóa mồi rồi nhấp | **xóa được thật** |

Vế cuối là vế quan trọng nhất. Không có nó thì "0 tệp biến mất" có thể chỉ vì cú nhấp trượt hoặc vì cả đường xóa đã hỏng — và sáu phép thử trên thành vô nghĩa. Một phép thử an toàn luôn xanh là một phép thử chưa chứng minh được gì.

Hai lần bộ chạy này báo hỏng, **cả hai đều là lỗi của chính nó**, không phải của công cụ: lần đầu tọa độ nút sai vì tính theo `GetWindowRect` thay vì `ClientToScreen` (khung winit có viền vô hình), lần sau `keybd_event` gửi phím trần nên ô nhập nhận `xoa` chữ thường — mà chữ thường thì đúng là không được mở khóa. Nhìn vào ô nhập mới biết; không nhìn thì rất dễ kết luận ngược lại.

### Việc M5 còn lại, nói thẳng

Bảng ở đây từng liệt kê ba việc — JPEG XL, màn sao lưu/khôi phục, `ĐM-08`. **Cả ba đã xong**; xem mục *"Ba việc từng nợ"* ở trên. `BP-01` cũng đã xong sau khi chốt `Q14`. Còn lại đúng ba việc, và không việc nào là mã:

| Việc | Cần gì | Vì sao máy không làm thay được |
|:---|:---|:---|
| **`§8.1-2` · `MAU-01` · `MAU-09`** | **ba người thử** nhìn bộ ảnh greyscale đã sinh sẵn | Máy đo được ký hiệu, câu chữ, hình sau khi bỏ màu, vị trí và khoảng cách. Nó **không** đo được người ta có *hiểu* không |
| **`ĐM-08`** — phép dò bỏ sót Narrator | **một quyết định**, `Q15` | Đo xong: Narrator chạy 20 giây mà `SPI_GETSCREENREADER` vẫn `False`, và hai tín hiệu thay thế cũng hỏng. Phản ứng của giao diện thì đúng 7/7. Sửa hay không là chuyện đụng tới `RB-07` của hội đồng |
| Truy nốt vụ **ứng dụng tự thoát** | thêm số đo | Mẫu 5 lượt quá nhỏ. Xem mục ngay trên |

Một chỗ nữa, không phải mục mức 1 nhưng lộ ra trong lúc đo và đáng ghi: **mức rủi ro giữa — `Cần cân nhắc` — không có lối nào tới từ bản đồ họa.** Nó thuộc `CACHE HỆ THỐNG`, mà giao diện chưa mở lối quét ấy. Nghĩa là ba người thử của `MAU-01` chỉ xếp được **hai** mức chứ không phải ba, và câu "33/33" của hội đồng phải đọc lại theo đúng phạm vi ấy.

## M6 · Phát hành — ✗ cổng KHÔNG đạt, và đây là lý do đo được

> **Cổng M6:** build tái lập được từ CI công khai, mã băm khớp bản tải về.

**Cổng này không đạt.** Hai chỗ chặn, cả hai đều đo chứ không suy đoán, và **không chỗ nào nằm ở mã của dự án**.

### Chặn ① — `zalo-gui.exe` không tất định ngay trên một máy

Ba lần dựng sạch liên tiếp, cùng máy, cùng thư mục, cùng bộ cờ:

| | `zalo-cli.exe` | `zalo-gui.exe` |
|:---|:---|:---|
| Lần 1 | `79DA53FB…` | `19940115…` |
| Lần 2 | `79DA53FB…` | `431516A3…` |
| Lần 3 | `79DA53FB…` | `15E47B40…` |

Bản dòng lệnh tất định tuyệt đối; bản đồ họa đổi mỗi lần. Nguyên nhân nằm trong build script của thư viện đồ họa — `glutin_egl_sys` và `glutin_wgl_sys` sinh mã liên kết OpenGL theo thứ tự duyệt bảng băm, mà Rust ngẫu nhiên hóa thứ tự ấy theo từng tiến trình. Bịt được thì phải vá hoặc nhúng bản riêng của thư viện ngoài.

### Chặn ② — bộ công cụ MSVC là đầu vào của bản dựng, mà không ghim được

Ngay cả `zalo-cli.exe`, thứ tất định tuyệt đối trên máy phát triển, vẫn khác bản CI: `79DA53FB…` so với `B7FD9341…`, **cùng kích thước 507.392 byte**, khác 5,32% số byte rải trên 3.865 vùng.

Thư viện CRT của Visual Studio được liên kết vào tệp thực thi, nên phiên bản của nó là một đầu vào ngang hàng với phiên bản `rustc`. Máy chủ GitHub **đổi ảnh máy giữa các lượt chạy** — đo được Visual Studio `18.8.12023.21` ở lượt này và `18.7.11925.98` ở lượt kế. Tức CI còn không tái lập được với chính nó.

### Ba thứ đã sửa được trên đường đi

| Sửa | Bằng chứng |
|:---|:---|
| **Dấu thời gian PE** — trước đó hai lần dựng sạch cùng máy cùng đường dẫn đã ra hai tệp khác nhau | `/Brepro` · sau đó `zalo-cli` tất định |
| **Đường dẫn tuyệt đối nhúng vào tệp** — `glutin` để lọt `D:\<gốc>\rust\target\...\egl_bindings.rs` vào chuỗi báo lỗi | `--remap-path-prefix` · dựng ở hai thư mục khác nhau ra tệp **trùng khít từng byte** |
| **Trình liên kết lấy nhầm toolchain** — script hỏi `rustc --print sysroot` ở gốc repo, nơi `rust-toolchain.toml` không có hiệu lực, nên CI lấy `rust-lld` của 1.97.1 đi liên kết mã do 1.94.1 biên dịch | Hỏi sysroot từ **bên trong** `rust\` |

Lỗi thứ ba là lỗi của chính bộ dựng tôi viết, và nó **không lộ ra trên máy phát triển** vì toolchain mặc định ở đó tình cờ trùng bản đã ghim.

### Ba lựa chọn, kèm giá

| | Làm gì | Giá |
|:---|:---|:---|
| **A** | Nhận hiện trạng: phát hành đúng tệp CI dựng, kèm `SHA256SUMS.txt`. Mã băm chứng minh tệp không bị sửa **trên đường**, và bản `.ps1` vẫn là đường kiểm chứng đến tận cùng | 0 đồng, và tài liệu phát hành đã nói thẳng chuyện này |
| **B** | Máy chủ tự quản với Visual Studio ghim phiên bản | Tiền và công bảo trì đều đặn |
| **C** | Nhúng bản riêng của `glutin` đã vá cho tất định, rồi làm tiếp chặn ② | Ôm một nhánh riêng của thư viện ngoài — món nợ dài hạn |

### Trạng thái: sẵn sàng, và CỐ Ý chưa công bố

Máy móc phát hành đã xong và đã kiểm đầu-cuối:

| | |
|:---|:---|
| CI dựng gói | ✅ `aac3e17` |
| Mã băm khớp tệp thật | ✅ 5/5 |
| `zalo-cli.exe` chạy từ bản tải về | ✅ chữ tiếng Việt đúng |
| `zalo-gui.exe` chạy từ bản tải về | ✅ mở được cửa sổ |
| Gắn thẻ và tạo Release công khai | ⛔ **chưa làm, có chủ đích** |

Lý do dừng ở bước cuối: danh mục tiếp cận của hội đồng ghi **"Mức 1 chặn hẳn bản phát hành, không thương lượng"**, mà M5 còn **9 mục mức 1 chưa ai kiểm**. Công bố lúc này là bỏ qua một luật do chính dự án đặt ra.

Đây là một quyết định, không phải một việc bị quên. Kiểm xong 9 mục ấy thì chỉ cần đẩy một thẻ `v*` — workflow tự dựng, tự băm, tự tạo Release. Không còn việc kỹ thuật nào phải làm.

**Chủ dự án chọn A** (02/08/2026) — xem [`quyet-dinh.md`](quyet-dinh.md) §Q13. Lý do: dự án còn một đường khác cho cùng niềm tin ấy, và với người dùng thường nó mạnh hơn — bản `.ps1` đọc thẳng được, không cần dựng lại gì. B và C mua thêm một tầng cho nhóm người đã có sẵn một tầng.

---

# Q5 · Port 204 phép thử

Đây là chỗ kế hoạch khác hẳn dự đoán ban đầu, nhờ một con số đo được.

Brief lo rằng bộ test lái bằng chuỗi phím nên "không dùng lại được với giao diện đồ họa". Đo lại thì **chỉ 69/190 phép thử lái kiểu đó**; **121 phép thử còn lại gọi hàm trực tiếp** và sang `cargo test` gần như một đổi một.

## Cách xử lý 69 phép thử còn lại

Ship `zalo-cli` — một bin **headless nói đúng giao thức phím cũ**. Ba cái lợi cùng lúc:

1. 69 phép thử E2E chạy được **không sửa**, chỉ đổi đường dẫn công cụ.
2. Bộ so sánh song song có ngay công cụ để lái cả hai bản như nhau.
3. Người muốn đọc mã trước khi chạy vẫn có đường dùng không cần giao diện.

> **Một điều tuyệt đối không được làm:** chế độ headless **đi qua đúng lõi và đúng mọi chốt** như giao diện. Nó là một cái vỏ khác, không phải một đường tắt. Thêm một cờ kiểu `--yes` bỏ qua xác nhận là phá nguyên tắc bất biến số 1, và biến chính công cụ thành thứ mà nó ra đời để chống.

## Bảng phân công

| Nhóm | Số | Sang Rust bằng |
|:---|---:|:---|
| Gọi hàm trực tiếp | 121 | `cargo test` trong `zalo-core` |
| Lái bằng chuỗi phím | 69 | Giữ nguyên tệp `.ps1`, trỏ vào `zalo-cli.exe` |
| Kiểm chỗ nối dây bằng AST | (nằm trong 121) | Rust không có AST runtime → thay bằng **phép thử tích hợp** gọi `act/` với một chốt giả và khẳng định chốt bị gọi |

Dòng cuối là món nợ thật: PowerShell đọc được AST của chính nó, Rust thì không. Bù bằng phép thử tích hợp có tiêm phụ thuộc — nhiều việc hơn, nhưng mạnh hơn vì nó kiểm hành vi chứ không kiểm mã nguồn.

---

# Q6 · Tương thích bản sao lưu

**Yêu cầu cứng:** bản Rust phải khôi phục được bản sao lưu do PowerShell tạo. Người dùng đang có bản sao lưu thật trên máy.

Định dạng `_zalocleanup_backup.json` phải giữ **nguyên tên trường và nguyên kiểu**:

```
Tool = "ZaloCleanup"   Version = 4   Created = "yyyy-MM-dd HH:mm:ss"
SourceRoot   ScanKind   Count   Bytes
FullVerify   Verified   VerifyFail   CopyFail
```

> **Chú ý một bẫy:** trường `Version` ghi **4** trong khi công cụ là **v5**. Đó là hiện trạng, và bản Rust phải **ghi đúng số 4**, không được "sửa cho đúng". `Read-BackupSet` hiện không kiểm trường này, nhưng đổi nó là đổi hợp đồng mà không được gì.

Cấu trúc thư mục cũng là hợp đồng: `<đích>\<yyyyMMdd_HHmmss>\<đường dẫn tương đối giữ nguyên>`.

**Kiểm chứng — cổng M4:** sao lưu chéo hai chiều, so SHA-256 từng tệp.

---

# Q7 · Phát hành

| Việc | Quyết |
|:---|:---|
| **Ký số** | Chờ `P2-1`. Chứng chỉ **EV** có uy tín SmartScreen ngay; **OV** phải tích lũy, nên người tải sớm vẫn gặp cảnh báo. Tự ký thì vô ích với SmartScreen |
| **Không ký thì sao** | Vẫn phát hành được, nhưng README phải nói thẳng người dùng sẽ thấy gì và vì sao, kèm cách vượt qua. Giấu chuyện đó là làm người dùng hoảng đúng lúc họ nên tin |
| **Diệt virus** | Gửi mẫu cho các hãng **trước** ngày phát hành. Exe xóa hàng loạt tệp cộng gọi `vssadmin` là hồ sơ báo động giả điển hình |
| **Kiểm chứng nguồn** | Ghim `Cargo.lock` và `rust-toolchain.toml`, build bằng CI công khai, công bố SHA-256 của exe kèm liên kết tới nhật ký build. Đây là thứ **thay thế** cho khả năng đọc mã đã mất khi chuyển sang exe |
| **Cập nhật** | **Không có bất kỳ kết nối mạng nào.** Không tự kiểm bản mới, không phone home. Công cụ hiện tại không hề chạm mạng và đó là điểm mạnh thật với mô hình đe dọa này. Kiểm bản mới là việc tay, README chỉ chỗ |

---

# Q8 · Bố cục và tệp dùng chung

```
D:\zalo-tool\
  ZaloCleanup.ps1              bản PowerShell — ở lại, không bỏ
  ZaloCleanup.Tests.ps1        204 phép thử — hợp đồng cho CẢ HAI bản
  catalog.json                 DÙNG CHUNG
  settings.json                riêng bản PowerShell
  profiles.json                riêng bản PowerShell
  logs\                        DÙNG CHUNG
  rust\
    Cargo.toml                 workspace
    Cargo.lock                 ghim, có commit
    rust-toolchain.toml        ghim phiên bản, cho build tái lập
    crates\ zalo-core\ zalo-cli\ zalo-gui\
    tests\differential\        bộ so sánh song song
    settings.rust.json         riêng bản Rust
    profiles.rust.json         riêng bản Rust
```

Theo `R-06`. Ba chi tiết phải làm cho đúng:

- **`catalog.json` dùng chung** nên hai bản phải chấp nhận **cùng một tập lỗi định dạng** và nêu tên mục sai **giống nhau**. Đưa vào cổng M1.
- **`logs\` dùng chung** nên thêm một dòng đầu nhật ký `# Bản: powershell` hoặc `# Bản: rust`. Lịch sử dọn dẹp phải là một dòng duy nhất, nhưng phải truy được ai làm.
- **Khóa `Local\ZaloCleanup.singleton`** (`R-16`) đã có ở bản PowerShell. Bản Rust lấy đúng tên đó.

---

# Q9 · Mốc dừng

Bốn tiêu chí. **Chạm một cái là dừng và báo cáo, không tự đi tiếp.**

| # | Điều kiện dừng | Vì sao nó giết dự án |
|:-:|:---|:---|
| **D-1** | Bộ so sánh ở **M1** có **bất kỳ** khác biệt nào chưa giải thích được | Lõi an toàn lệch nghĩa là không port được thứ nguy hiểm nhất. Mọi thứ sau đó vô nghĩa |
| **D-2** | **M2** quét 52.748 tệp **> 1,5 giây** | Lợi ích hiệu năng không có. Chỉ còn giao diện, và lúc đó phải hỏi lại có đáng không |
| **D-3** | **M3** không chạy được 69 phép thử E2E không sửa | Chiến lược kiểm chứng sụp. Viết tiếp là viết mù |
| **D-4** | Lõi vượt **60 crate** | Mất chính lý lẽ kiến trúc của kế hoạch này |

**Mốc kiểm điểm bắt buộc: sau M2.** Lúc đó đã biết lõi có port đúng không và có nhanh hơn thật không, mà chưa tiêu công vào giao diện. Đây là chỗ rẻ nhất để đổi ý.

---

# Rủi ro lớn nhất của chính kế hoạch này

Không phải Rust khó. Là **hai bản cùng sống thì bộ test phải xanh cho cả hai, mãi mãi**.

Mỗi lần sửa bản PowerShell là một lần có thể làm bản Rust lệch, và ngược lại. Nếu không có CI chạy cả hai sau mỗi commit thì hai bản sẽ trôi xa nhau trong im lặng — và ngày phát hiện ra là ngày một trong hai xóa nhầm dữ liệu.

> **Việc phải làm ở M0, không phải để sau:** dựng CI chạy `ZaloCleanup.Tests.ps1` cho **cả** bản PowerShell **và** `zalo-cli` sau mỗi commit.

Đây cũng là bài học lặp lại lần thứ tư trong dự án này, ở dạng khác: **một nhánh không có test là một nhánh chưa từng chạy** — và hai bản không cùng chạy một bộ test là hai bản đang lặng lẽ trở thành hai công cụ khác nhau.
