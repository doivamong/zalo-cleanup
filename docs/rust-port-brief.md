# Brief cho phiên lập kế hoạch: viết lại `zalo-cleanup` bằng Rust, có giao diện, phát hành exe dựng sẵn

> Dán toàn bộ tệp này vào một phiên mới, kèm câu: **"Đọc brief này rồi lập kế hoạch. Chưa viết code."**
>
> Đây là ảnh chụp thực trạng đo được ngày **01/08/2026**, không phải trí nhớ. Mọi con số trong đây đều đo trên máy thật.

---

## 0. Mục tiêu cuối

Ba thứ, theo thứ tự phụ thuộc:

1. **Viết lại lõi bằng Rust**, giữ nguyên mọi bảo đảm an toàn của bản PowerShell hiện tại.
2. **Hội đồng UI-UX nghiên cứu và dựng giao diện hoàn chỉnh** (mục 6).
3. **Phát hành `.exe` dựng sẵn** cho người dùng cuối tải về chạy ngay, không cần cài PowerShell, không cần bật ExecutionPolicy, không cần build.

### Đã chốt: bố cục kho mã và chế độ chạy song song

Chủ dự án đã quyết hai điều, không cần hỏi lại:

- **Bản Rust nằm ở `D:\zalo-tool\rust\`**, cùng kho mã với bản PowerShell. Một repo, hai bản.
- **Hai bản sống song song.** Bản PowerShell KHÔNG bị bỏ khi bản Rust chạy được. Nó ở lại làm hai
  việc: làm chuẩn đối chiếu cho mọi bước kiểm chứng (mục 10), và làm bản đọc-được-mã cho ai không
  muốn chạy một tệp exe.

Hai hệ quả kỹ thuật đi kèm:

1. **`D:\zalo-tool` đã nằm sẵn trong vùng bảo vệ mức `tất cả`** (`$script:ToolDir`, xem
   `Initialize-ProtectedAbs`). Nên `rust\` được bảo vệ tự động — công cụ không bao giờ tự xóa mã
   nguồn của chính nó. Bản Rust phải giữ đúng luật này cho chính nó.
2. **`.gitignore` đã thêm `rust/target/`.** `Cargo.lock` thì GIỮ LẠI, vì đây là ứng dụng phát hành
   dạng exe dựng sẵn nên phải build lại được đúng bộ phụ thuộc của bản đã phát hành.

Kế hoạch phải nói rõ: cấu trúc thư mục bên trong `rust\`, hai bản dùng chung `catalog.json` /
`settings.json` / `profiles.json` / `logs\` ở thư mục gốc hay mỗi bản một bộ riêng, và nếu dùng chung
thì xử lý ra sao khi hai bản cùng mở.

Phiên sau **lập kế hoạch, chưa viết code**. Kế hoạch phải trả lời hết mục 9.

Chủ dự án đã cân nhắc và quyết định làm. **Không lập luận lại chuyện có nên port hay không** — việc của phiên sau là làm cho tốt, và nêu rủi ro cụ thể ở từng bước chứ không nêu chung chung.

---

## 1. Vì sao việc này đáng làm — và vì sao lý do đã đổi

Đọc kỹ đoạn này, vì nó sửa lại một kết luận sai trước đó.

Ở phiên khảo sát, tôi kết luận "không nên viết lại bằng Rust" dựa trên **tốc độ**. Kết luận đó **vẫn đúng nếu chỉ xét tốc độ**: sau khi tối ưu, bản PowerShell chỉ còn chậm hơn Rust khoảng **10–15 giây mỗi phiên làm việc** (mục 4). Mua 15 giây bằng một bản viết lại là không đáng.

**Nhưng yêu cầu đã đổi, và với yêu cầu mới thì lập luận đó không còn liên quan.**

| Yêu cầu mới | PowerShell 5.1 làm được không |
|---|---|
| Giao diện đồ họa hoàn chỉnh | Không, một cách thực tế. WinForms/WPF qua PowerShell là đường cụt về bảo trì |
| Một tệp `.exe` chạy ngay, không cần cài gì | Không. `.ps1` cần PowerShell và cần người dùng vượt ExecutionPolicy |
| Phát hành cho người không rành kỹ thuật | Không. Hướng dẫn hiện tại đòi người dùng mở terminal |

→ **Rust được chọn vì năng lực, không phải vì tốc độ.** Tốc độ chỉ là phần thưởng kèm theo. Kế hoạch phải nói đúng như vậy ở phần mở đầu, đừng bán bằng con số hiệu năng — con số đó nhỏ và người đọc kiểm được.

Đổi lại, có ba thứ bị mất, phải chấp nhận có ý thức:

- **Mất khả năng đọc mã trước khi chạy.** Hôm nay người dùng mở `.ps1` bằng Notepad là đọc được toàn bộ logic xóa. Một `.exe` thì không. Với công cụ chuyên xóa dữ liệu vĩnh viễn, đây là mất mát thật.
- **Mất "0 phụ thuộc"** — điểm README đang lấy làm điểm bán hàng.
- **Mất tính sửa-là-chạy.** Đổi một bộ lọc giờ cần cài toolchain và build lại.

---

## 2. Đối tượng

| | |
|---|---|
| Kho mã | `D:\zalo-tool` · https://github.com/doivamong/zalo-cleanup · nhánh `main` |
| Công dụng | Công cụ Windows dọn dữ liệu Zalo và cache hệ thống để lấy lại dung lượng ổ đĩa |
| `ZaloCleanup.ps1` | **2.859 dòng · 80 hàm** |
| `ZaloCleanup.Tests.ps1` | **892 dòng · 163 phép thử** (161 chỗ gọi `Assert`, 24 nhóm) |
| `README.md` | 722 dòng — tài liệu chính, tả cả thiết kế an toàn |

**Bản chất công cụ: nó xóa vĩnh viễn hàng chục GB dữ liệu cá nhân, không qua Thùng rác.** Mọi quyết định thiết kế, nhất là quyết định giao diện, phải đọc dưới ánh sáng đó.

Năm hàm lớn nhất chiếm phần lớn độ phức tạp:

| Hàm | Dòng | Việc |
|---|---:|---|
| `Invoke-Delete` | 225 | Xóa, ghi nhật ký, đếm, cắt cụt tệp bị khóa |
| `Invoke-Restore` | 194 | Khôi phục từ bản sao lưu |
| `Invoke-CatalogScan` | 183 | Quét cache hệ thống theo `catalog.json` |
| `Invoke-Backup` | 160 | Sao lưu + xác minh hai mức |
| `Invoke-DedupScan` | 155 | Khử trùng lặp bằng SHA-256 |

Còn lại là **68 chỗ hỏi người dùng** và các màn hình menu — đây chính là phần hội đồng UI-UX phải thiết kế lại.

---

## 3. Trạng thái máy phát triển

| | |
|---|---|
| CPU / RAM | i5-12400 (6C/12T) · 32 GB |
| Ổ chứa dữ liệu | Kingston A400 **SATA** SSD, C: còn ~56 GB trống |
| OS / Shell | Windows 11 Pro 26200 · PowerShell **5.1** |
| Rust | **rustc 1.94.1**, target `x86_64-pc-windows-msvc` |
| Linker | VS 2022 **BuildTools** có sẵn — đã thử `cargo build --release`, **chạy được**, hello-world ra exe 127 KB |
| Dữ liệu thật | 56.950 tệp / 32,29 GB trong `%APPDATA%\ZaloData\media\<id>\ZaloDownloads` |

---

## 4. Hiệu năng — bar đã dịch chuyển

Codebase **vừa được tối ưu mạnh**. Bar mà bản Rust phải vượt là bar mới.

| Thao tác | Trước tối ưu | **Hiện tại** | Rust ước tính |
|---|---:|---:|---:|
| Quét theo bộ lọc, 52.748 tệp | 105,0 s | **5,2 s** | ~0,5 s |
| Khử trùng lặp, hồ sơ thật | 7,1 s | **1,0 s** | ~0,9 s |
| Chu kỳ sao lưu + xóa, 4.000 tệp | 11,1 s | **8,2 s** | ~5 s |
| Bộ test đầy đủ | 108 s | **62 s** | — |

**Trần vật lý đã đo, Rust không vượt được:**

- Đọc đĩa nguội: **41 MB/s** một luồng, **53 MB/s** tám luồng.
- SHA-256 của .NET khi dữ liệu đã ở trong RAM: **840 MB/s** — nhanh gấp 20 lần tốc độ đĩa nạp dữ liệu.

→ **Khử trùng lặp bị chặn bởi ổ đĩa, không phải bởi ngôn ngữ.** Đừng đặt kỳ vọng vào phần băm.
→ Chỗ Rust thắng thật là **duyệt cây thư mục và xử lý từng tệp**.

---

## 5. Những gì phải dựng lại bằng Rust

### 5.1 Lệnh ngoài — chỉ đúng một

`vssadmin` (4 chỗ): `list shadowstorage`, `resize shadowstorage`, `delete shadows /oldest`, `delete shadows /all`.
Rust vẫn `std::process::Command` như cũ. **Không parse theo từ khóa tiếng Anh** — `vssadmin` bị bản địa hóa theo ngôn ngữ Windows; bản PowerShell in nguyên văn đầu ra và chỉ bỏ dòng tiêu đề. Giữ đúng cách đó.

### 5.2 Cmdlet cần hàm thay thế

| PowerShell | Số chỗ | Hướng thay bằng Rust |
|---|---:|---|
| `Get-PSDrive` / dung lượng trống | 6 | `GetDiskFreeSpaceExW` |
| `Get-Process` | 3 | `CreateToolhelp32Snapshot` hoặc crate `sysinfo` |
| `Start-Process -Verb RunAs` | 1 | `ShellExecuteExW` với verb `runas` |
| `Stop-Process` | 1 | `OpenProcess` + `TerminateProcess` |
| `Export-Csv` | 1 | crate `csv` |
| `Get-Random` | 2 | crate `rand` |

Phần còn lại chủ yếu là `[IO.Path]` và `[IO.File]` — có `std::fs` / `std::path` thay thẳng.

### 5.3 Đặc thù Windows — chỗ dễ làm sai nhất

- **Đường dẫn dài**: đọc registry `LongPathsEnabled`; nếu tắt thì thêm tiền tố `\\?\`.
- **Controlled Folder Access**: đọc registry `EnableControlledFolderAccess` để cảnh báo trước.
- **Reparse point / junction**: xem mục 7, đây là bẫy nghiêm trọng nhất.
- **Nâng quyền**: cần cho một số mục cache hệ thống. Với bản GUI phải nghĩ lại cách xin quyền — hiện tại là khởi động lại chính mình.

### 5.4 Định dạng tệp — BẮT BUỘC tương thích ngược

| Tệp | Vai trò |
|---|---|
| `catalog.json` | Danh mục cache hệ thống, **người dùng sửa được**, 33 mục. Có kiểm tra hợp lệ và nêu tên mục sai. Hỏng thì quay về danh mục dựng sẵn |
| `settings.json` · `profiles.json` | Cấu hình và hồ sơ bộ lọc |
| `_zalocleanup_backup.json` | **Bản kê của mỗi lần sao lưu — khôi phục sống nhờ tệp này** |
| `logs\daxoa_*.log` | Nhật ký xóa, dạng TSV: `TRẠNGTHÁI<TAB>BYTES<TAB>ĐƯỜNGDẪN` |
| `logs\khoiphuc_*` · `saoluu_loi_*` · `quet_*.csv` | Nhật ký khôi phục, lỗi sao lưu, xuất CSV |

> **Yêu cầu cứng: bản Rust phải khôi phục được bản sao lưu do bản PowerShell tạo ra.** Người dùng đang có bản sao lưu thật trên máy. Đổi định dạng là làm hỏng đường lui của họ.

---

## 6. Hội đồng UI-UX

### 6.1 Vì sao cần một hội đồng chứ không phải một người vẽ giao diện

Giao diện của công cụ này không phải bài toán thẩm mỹ. **An toàn của bản hiện tại đến một phần lớn từ ma sát**: phải quét mới xóa được, phải gõ đủ chữ `XÓA`, phải đi qua nhiều màn hình. Giao diện đồ họa xóa sạch ma sát đó — mọi thứ cách nhau một cú nhấp.

> **Rủi ro số một của cả dự án này không phải là bug trong Rust. Là một giao diện đẹp khiến người ta xóa nhầm 30 GB ảnh trong ba giây.**

Hội đồng tồn tại để chuyện đó không xảy ra.

### 6.2 Thành phần và sản phẩm bắt buộc

Mỗi ghế phải ra được sản phẩm cụ thể, không phải ý kiến.

| Ghế | Trả lời câu gì | Sản phẩm |
|---|---|---|
| **An toàn & rủi ro** | Ma sát nào bắt buộc phải giữ? Xác nhận thế nào cho tương xứng mức rủi ro? | Bảng phân mức rủi ro cho từng hành động, kèm dạng xác nhận tương ứng |
| **Kiến trúc thông tin** | Bốn nguồn dung lượng và menu nâng cao ánh xạ sang giao diện ra sao? | Sơ đồ điều hướng, quy tắc bộc lộ dần |
| **Thiết kế tương tác** | Trạng thái quét / đang xóa / đã xong hiện thế nào? Hủy giữa chừng ra sao? | Mô tả từng màn hình và từng trạng thái, kể cả trạng thái lỗi |
| **Tiếp cận & bản địa** | Tiếng Việt, chỉ dùng bàn phím, trình đọc màn hình, DPI cao, chế độ tối | Danh mục kiểm tra, có tiêu chí đạt/không đạt |
| **Khả thi kỹ thuật** | Khung giao diện nào chịu được ràng buộc một-tệp-exe? | So sánh khung, kèm kích thước exe đo thật |

**Bắt buộc có một vòng phản biện đối kháng**: sau khi có bản thiết kế, một lượt riêng đi tìm cách làm người dùng xóa nhầm dữ liệu bằng chính giao diện đó. Tìm được đường nào thì đường đó phải bị bịt trước khi chốt.

### 6.3 Ràng buộc bất di dịch mà thiết kế phải tôn trọng

- Năm nguyên tắc bất biến ở mục 7 — giao diện **không được** phá cái nào.
- **Không quét thì không thể xóa.** Giao diện không được có nút "dọn ngay" bỏ qua bước quét.
- **Xem trước cái sắp mất.** Người dùng phải thấy được danh sách trước khi xác nhận, không chỉ thấy con số.
- **Sao lưu là đường lui duy nhất**, nên trạng thái sao lưu phải nhìn là thấy, không phải mở ra mới biết.
- **Vấn đề Volume Shadow Copy phải hiện lên giao diện** (mục 7.1). Người dùng xóa 15 GB mà ổ đĩa không trống thêm sẽ nghĩ công cụ hỏng. Bản dòng lệnh giải thích bằng chữ; bản đồ họa phải làm tốt hơn thế.
- **Hủy giữa chừng phải an toàn và phải có.** Bản hiện tại nói "Ctrl+C bất cứ lúc nào".
- **Công cụ không tự chạy** — không Scheduled Task, không hook, không tiến trình nền. Giao diện đồ họa không được lén biến thành ứng dụng chạy nền hay khởi động cùng Windows.

### 6.4 Ràng buộc kỹ thuật do yêu cầu một-tệp-exe đặt ra

Đây là chỗ UI và phát hành dính vào nhau, phải quyết cùng lúc:

- Khung dựa trên WebView (**Tauri**) cần **WebView2 runtime**. Windows 11 có sẵn, Windows 10 cũ thì không chắc → phá vỡ lời hứa "tải về chạy ngay".
- Khung Rust thuần (**egui**, **iced**) biên dịch thẳng vào một exe, không cần runtime → hợp với yêu cầu, nhưng khác cảm giác ứng dụng Windows gốc.
- Win32 / WinUI gốc: đúng chất Windows nhất, nhiều việc nhất.

Hội đồng phải chọn có căn cứ và **đo kích thước exe thật** cho từng phương án, không đoán.

---

## 7. Bài học đã trả giá — port sai là mất dữ liệu

Phần quan trọng nhất của brief. Một bản viết lại "sạch sẽ" từ đầu sẽ làm sai gần hết những điều dưới đây, vì chúng đều phản trực giác.

1. **Volume Shadow Copy nuốt dung lượng.** Xóa tệp khi còn bản chụp System Restore thì dung lượng **không** được trả về ổ đĩa — VSS chép khối cũ sang shadow storage theo copy-on-write. Bằng chứng đo được: xóa 12,96 GB chỉ thu về 0,04 GB. Sau khi tắt System Restore, xóa 15,05 GB thu về 14,81 GB. → **Luôn kiểm chứng bằng dung lượng trống của ổ đĩa, không bao giờ bằng tổng byte đã xóa.**

2. **Ứng dụng tự tải lại thứ vừa xóa.** Xóa cache cập nhật của Ollama lúc 02:52 thì nó tải lại đúng tệp đó lúc 03:03. → Chặn xóa khi tiến trình liên quan đang chạy (`procs` trong `catalog.json`).

3. **Junction.** `Get-ChildItem -Recurse` của PowerShell **không** đi xuyên junction, nhưng `EnumerateDirectories` của .NET **có**. Trong Rust phải kiểm chứng riêng cho junction trên NTFS chứ đừng tin mặc định của thư viện. → Đi xuyên là mở đường cho lệnh xóa lan sang thư mục ở đầu bên kia.

4. **Không bao giờ xóa đệ quy.** Giữa lúc kết luận "thư mục này rỗng" và lúc ra lệnh xóa có một khe hở; tiến trình khác kịp ghi tệp vào đó thì xóa đệ quy cuốn luôn tệp ấy mà không qua lớp kiểm vùng bảo vệ. Bản PowerShell dùng xóa **không đệ quy**, vốn ném lỗi khi thư mục hết rỗng — đúng thứ ta muốn.

5. **Vùng miền.** Công cụ có phép thử chạy dưới `vi-VN`, nơi `20.000` nghĩa là hai mươi nghìn. Mọi phép so khớp chuỗi và so sánh đường dẫn phải dùng **ordinal**, không bao giờ theo vùng miền hiện hành.

6. **Chỉ đếm là đã xóa khi tệp thật sự biến mất.** Kiểm lại sau khi gọi lệnh xóa, không tin giá trị trả về.

7. **Sao lưu lỗi dù chỉ một tệp thì chặn luôn bước xóa.**

8. **Vùng bảo vệ chặn cứng ở tầng code**, hai mức: `tất cả` (chặn cả cây bên dưới) và `gốc` (chỉ chặn khi nhắm thẳng vào chính nó). Thư mục gốc còn phải kiểm **chiều ngược**: nhận một thư mục *chứa* vùng bảo vệ cũng nguy hiểm y như nhận chính vùng bảo vệ.

9. **Một nhánh không có test là một nhánh chưa từng chạy.** Mức xác minh `SHA-256 toàn bộ` từng hỏng hoàn toàn suốt nhiều phiên bản mà không ai biết, chỉ vì chưa phép thử nào từng chọn mức đó — nghĩa là người dùng chọn mức chắc chắn nhất trước khi xóa lại là người duy nhất gặp lỗi.

### Năm nguyên tắc bất biến

Chép nguyên văn từ đầu `ZaloCleanup.ps1`. Bản Rust và giao diện mới phải giữ đủ cả năm:

1. Không quét thì không thể xóa.
2. Đổi bộ lọc là kết quả quét cũ bị hủy.
3. Nhập sai thì giữ nguyên, không bao giờ tự mở rộng phạm vi.
4. Vùng bảo vệ bị chặn cứng ở tầng code.
5. Sao lưu chưa sạch thì không cho xóa.

---

## 8. Phát hành exe dựng sẵn — vấn đề phải giải, không phải phụ lục

Một exe **không ký**, có giao diện, xóa hàng loạt tệp và gọi `vssadmin`, là hồ sơ điển hình của cảnh báo SmartScreen và báo động giả của phần mềm diệt virus. Bản `.ps1` hiện tại không dính vấn đề này. Yêu cầu phát hành exe đã chốt, nên kế hoạch **bắt buộc** phải xử lý:

- **Ký số.** Chứng chỉ ký mã tốn tiền hằng năm. Chứng chỉ tự ký **không** làm SmartScreen im lặng.
- **Uy tín SmartScreen.** Ngay cả khi đã ký, chứng chỉ mới vẫn phải tích lũy uy tín qua lượt tải. Người tải sớm vẫn gặp cảnh báo. Kế hoạch phải nói rõ sẽ sống chung thế nào.
- **Báo động giả của diệt virus.** Cần quy trình gửi mẫu cho các hãng, và một trang giải thích cho người dùng.
- **Kiểm chứng nguồn.** Vì mất khả năng đọc mã (mục 1), nên bù lại bằng: build tái lập được, công bố mã băm của bản phát hành, và dựng exe bằng CI công khai để ai cũng đối chiếu được.
- **Cập nhật.** Có tự kiểm tra bản mới không? Nếu có thì phải đối chiếu với nguyên tắc "công cụ không tự chạy" và "không kết nối mạng lén".

---

## 9. Kế hoạch phải trả lời được

1. **Chia mô-đun thế nào?** Cây mô-đun cho ~2.900 dòng PowerShell, tách rõ **lõi an toàn** (vùng bảo vệ, xóa, sao lưu) khỏi **vỏ giao diện**. Lõi phải kiểm thử được mà không cần giao diện.
2. **Chọn crate nào**, và vì sao. Mỗi crate là một phụ thuộc mới trong một công cụ từng quảng cáo "0 phụ thuộc".
3. **Khung giao diện nào**, dựa trên kết luận của hội đồng và ràng buộc một-tệp-exe (mục 6.4).
4. **Thứ tự làm**, kèm chốt kiểm chứng ở từng bước. Gợi ý: lõi trước, giao diện sau; trong lõi thì bắt đầu từ phần thuần tính toán và dễ đối chiếu nhất (vùng bảo vệ, duyệt cây, băm), để `Invoke-Delete` và `Invoke-Restore` sau cùng.
5. **163 phép thử port kiểu gì?** Chúng đang là test đầu-cuối, lái công cụ bằng chuỗi phím qua stdin. Có giao diện đồ họa thì cách lái đó không dùng lại được. Chứng minh độ phủ không giảm.
6. **Giữ tương thích bản sao lưu cũ bằng cách nào**, kiểm chứng ra sao?
7. **Toàn bộ mục 8** — ký số, SmartScreen, diệt virus, kiểm chứng nguồn, cập nhật.
8. **Bố cục bên trong `rust\`** và cách hai bản dùng chung tệp cấu hình, nhật ký, thư mục sao lưu
   (đã chốt là chạy song song — xem mục 0). Hai bản cùng mở một lúc thì sao?
9. **Mốc dừng.** Sau bước 1, lấy tiêu chí đo được nào để kết luận đi tiếp hay dừng?

---

## 10. Chiến lược kiểm chứng — bắt buộc, không phải tùy chọn

Phiên trước đã chứng minh kỹ thuật này hiệu quả: khi tăng tốc `Test-Protected` 46 lần, hai bản cũ và mới được chạy song song trên **57.144 đầu vào** (231 ca biên dựng máy móc + toàn bộ 56.913 đường dẫn thật) và cho **0 khác biệt**. Nhờ vậy mới dám thay một lớp an toàn.

Dùng đúng cách đó ở mọi bước:

- **Đối chiếu song song.** Chạy bản PowerShell và bản Rust trên cùng một sandbox, so từng dòng đầu ra và từng tệp còn lại. Khác một ly là dừng.
- **So bằng SHA-256, không so bằng số lượng.** Sao lưu phải khớp nội dung, không chỉ khớp số tệp.
- **Kiểm bằng đột biến.** Cố tình phá một lớp an toàn rồi xem test có đỏ không. Phiên trước làm ba lần, lần nào cũng lộ ra chỗ test còn hổng.
- **Không bao giờ chạy thử trên dữ liệu Zalo thật ở chế độ có xóa.** Chỉ quét. Sandbox trong `%TEMP%` cho mọi thứ có xóa.
- **163 phép thử hiện có là hợp đồng.** Mỗi phép thử là một bài học đã trả giá. Port hết, không bỏ cái nào.

---

## 11. Câu cần hỏi chủ dự án trước khi chốt kế hoạch

- **Ngân sách ký số**: có sẵn sàng trả phí chứng chỉ ký mã hằng năm không? Câu này quyết định phần lớn mục 8.
- **Đối tượng người dùng**: chỉ mình chủ dự án và người quen, hay phát hành rộng? Quyết định mức đầu tư cho SmartScreen và tài liệu.
- **Windows 10 có nằm trong phạm vi hỗ trợ không?** Ảnh hưởng thẳng tới việc có được dùng khung dựa trên WebView2 hay không.
- Chấp nhận thêm bao nhiêu **crate phụ thuộc**, khi README đang quảng cáo "0 phụ thuộc"?

---

## 12. Đọc gì trước khi lập kế hoạch

| Nguồn | Vì sao |
|---|---|
| `README.md` | Nguồn chính xác nhất về hành vi và phím bấm. **Đừng tin trí nhớ về phím.** |
| `ZaloCleanup.ps1` dòng 1–110 | Năm nguyên tắc bất biến và toàn bộ biến trạng thái |
| `Test-Protected` · `Build-ProtectedIndex` · `Get-FilesSafe` | Lõi an toàn, và là ba hàm chạy nhiều nhất |
| `Invoke-Delete` · `Invoke-Backup` · `Invoke-Restore` | Ba hàm động tới dữ liệu người dùng |
| `Show-Home` · `Invoke-WizardReclaim` · `Show-AdvancedMenu` | Kiến trúc thông tin hiện tại — đầu vào cho hội đồng UI-UX |
| `ZaloCleanup.Tests.ps1` | 163 phép thử = 163 điều đã từng sai hoặc có thể sai |
| `git log` | Mỗi lời nhắn commit ghi rõ **vì sao**, không chỉ **cái gì** |
