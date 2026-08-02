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
| **M3** Chế độ headless | ⬜ chưa | |
| **M4** Động tới dữ liệu | ⬜ chưa | |
| **M5** Giao diện | ⬜ chưa | |
| **M6** Phát hành | ⬜ chưa | Chặn bởi `P2-1`, ngân sách ký số |

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

## M3 · Chế độ headless — mốc quyết định của cả kế hoạch

`zalo-cli` nhận **đúng giao thức phím qua stdin** như bản PowerShell.

> **Cổng M3:** chạy được **69 phép thử E2E hiện có, KHÔNG SỬA MỘT KÝ TỰ**, chỉ đổi đường dẫn công cụ, và cho cùng kết quả.

Đạt cổng này thì toàn bộ bộ test thành **hợp đồng sống cho cả hai bản**, và bộ so sánh song song có sẵn công cụ. Không đạt thì chiến lược kiểm chứng sụp và phải thiết kế lại trước khi viết thêm dòng nào.

## M4 · Động tới dữ liệu

`act/` — xóa, sao lưu, khôi phục, dọn thư mục rỗng.

> **Cổng M4:** ① bản sao lưu do **Rust** tạo phải khôi phục được bằng **PowerShell**, và ngược lại ② so SHA-256 từng tệp, không so số lượng ③ nhật ký hai bản khớp nhau về số dòng và trạng thái.

## M5 · Giao diện

Theo `ui-ux-council.md`.

> **Cổng M5:** ① danh mục tiếp cận **mức 1** của hội đồng đạt hết ② 36 đường tấn công đã bịt được kiểm lại tay ③ hai chốt xem trước và gõ cụm từ có phép thử tự động.

## M6 · Phát hành

> **Cổng M6:** build tái lập được từ CI công khai, mã băm khớp bản tải về.

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
