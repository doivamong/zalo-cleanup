# Dọn dẹp Zalo

**Lấy lại dung lượng ổ đĩa bị Zalo chiếm trên Windows — mà không mất một tấm ảnh nào.**

Zalo lưu mọi ảnh, video, file bạn nhận được vào ổ C và không bao giờ tự dọn theo cách bạn kiểm soát được. Sau vài năm, con số thường là vài chục GB. Công cụ này giúp bạn lấy lại phần đó một cách có chủ đích: **xem trước rồi mới xóa, sao lưu có xác minh, và nói thật với bạn về việc dung lượng có thực sự quay về hay không.**

Một tệp PowerShell duy nhất. Không cài đặt, không phụ thuộc, không chạy nền.

---

## Mục lục

- [Vì sao có công cụ này](#vì-sao-có-công-cụ-này)
- [Cài đặt](#cài-đặt)
- [Bắt đầu dùng](#bắt-đầu-dùng)
- [Bốn nguồn dung lượng](#bốn-nguồn-dung-lượng)
- [Vì sao dọn xong mà ổ đĩa không trống thêm](#vì-sao-dọn-xong-mà-ổ-đĩa-không-trống-thêm)
- [Thiết kế an toàn](#thiết-kế-an-toàn)
- [Sao lưu và khôi phục](#sao-lưu-và-khôi-phục)
- [Tham chiếu](#tham-chiếu)
- [Xử lý sự cố](#xử-lý-sự-cố)
- [Phát triển](#phát-triển)
- [Giới hạn đã biết](#giới-hạn-đã-biết)
- [Miễn trừ trách nhiệm](#miễn-trừ-trách-nhiệm)
- [English summary](#english-summary)

---

## Vì sao có công cụ này

Công cụ ra đời sau một sự cố mất dữ liệu. Khoảng 31,8 GB ảnh trong thư mục `picture\` của Zalo bị một tiến trình tự động xóa vĩnh viễn — không qua Thùng rác, không hỏi han, ngay giữa lúc đang phân tích thư mục đó.

Bài học rút ra định hình toàn bộ thiết kế: **thời điểm xóa phải do bạn quyết, và bạn phải nhìn thấy chính xác cái gì sắp mất trước khi nó mất.**

Vì vậy công cụ này:

- **Không đăng ký Scheduled Task**, không hook, không dịch vụ nền, không tự khởi động. Nó chỉ sống trong lúc cửa sổ của nó đang mở. Đóng cửa sổ là hết.
- **Không xóa được nếu chưa quét.** Mọi thao tác xóa đều dựa trên một danh sách bạn đã xem.
- **Không tự mở rộng phạm vi.** Gõ nhầm bộ lọc thì nó giữ nguyên lựa chọn cũ chứ không âm thầm chọn tất cả.

Nếu bạn chỉ muốn một nút "dọn cho nhanh", công cụ này sẽ khiến bạn bực. Nó cố tình bắt bạn nhìn trước khi bấm.

---

## Cài đặt

### Yêu cầu

| | |
|---|---|
| Hệ điều hành | Windows 10 hoặc Windows 11 |
| PowerShell | 5.1 — có sẵn trong Windows, **không cần cài gì thêm** |
| Zalo | Không bắt buộc. Máy chưa cài Zalo thì công cụ vẫn chạy được phần dọn cache hệ thống |
| Quyền quản trị | Không bắt buộc. Chỉ cần khi muốn dọn cache cấp hệ thống hoặc thao tác với Shadow Copy |

Không cần cài .NET, Python, hay bất cứ thứ gì. Không có trình cài đặt.

### Cách 1 — Tải bản nén (khuyến nghị cho người dùng thường)

1. Vào [trang repo](https://github.com/doivamong/zalo-cleanup), bấm nút xanh **Code** → **Download ZIP**.
2. Giải nén ra thư mục bất kỳ, ví dụ `D:\zalo-cleanup` hoặc `C:\Tools\zalo-cleanup`. Đường dẫn có dấu cách cũng được.
3. Bấm đúp **`ZaloCleanup.cmd`**.

Windows có thể hiện cảnh báo SmartScreen vì tệp vừa tải từ mạng. Bấm **More info** → **Run anyway**. Bạn cũng có thể đọc toàn bộ mã nguồn trước khi chạy — đó là điểm chính của việc nó là mã mở.

### Cách 2 — Clone bằng Git

```bash
git clone https://github.com/doivamong/zalo-cleanup.git
```

Rồi bấm đúp `ZaloCleanup.cmd` trong thư mục vừa tải về.

### Nếu bạn muốn chạy trực tiếp tệp `.ps1`

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File ".\ZaloCleanup.ps1"
```

Tệp tải từ mạng mang dấu Mark-of-the-Web nên PowerShell có thể từ chối chạy. Gỡ dấu đó một lần:

```powershell
Get-ChildItem -Recurse | Unblock-File
```

Tệp `ZaloCleanup.cmd` đã bao sẵn `-ExecutionPolicy Bypass` nên nếu dùng nó thì bạn không gặp vấn đề này. Công cụ **không** thay đổi Execution Policy của máy bạn.

### Chép sang máy khác

Chép cả thư mục là dùng được. Công cụ không giả định gì về máy đích:

- **Không giả định ổ C.** Mọi đường dẫn hệ thống lấy từ `%SystemDrive%`, `%WINDIR%`, `%ProgramData%`, `%ProgramFiles%`.
- **Không giả định ngôn ngữ Windows.** Đầu ra của `vssadmin` bị bản địa hóa nên công cụ in nguyên văn thay vì lọc theo từ khóa tiếng Anh.
- **Không giả định định dạng số theo vùng miền.** `20,000` và `20.000` đều đọc đúng.
- **Tự xử lý môi trường.** Đặt bảng mã console về UTF-8 để chữ tiếng Việt hiển thị đúng rồi trả lại lúc thoát; tự phát hiện Windows có bật hỗ trợ đường dẫn dài hay không; phát hiện Controlled Folder Access của Windows Defender và cảnh báo trước thay vì để bạn nhận một loạt lỗi khó hiểu.

---

## Bắt đầu dùng

Mở công cụ, bạn thấy màn hình này:

```
  ╔══════════════════════════════════════════════════════════╗
  ║   DỌN DẸP ZALO — chỉ chạy khi bạn mở công cụ             ║
  ╚══════════════════════════════════════════════════════════╝

   Ổ C còn trống    62.41 GB
   Thư mục Zalo     28.7 GB

   Bạn muốn làm gì?

    1   Lấy lại dung lượng ổ đĩa
    2   Xem máy đang chiếm bao nhiêu
    3   Khôi phục dữ liệu đã sao lưu

    9   Tùy chọn nâng cao
    0   Thoát
```

Bạn không cần biết khái niệm quét, bộ lọc, hay kết quả nằm trong bộ nhớ. Cứ chọn theo mục đích, công cụ lo phần còn lại.

### Nếu bạn chỉ muốn thử một lần cho an tâm

Chọn **`2`** — chỉ đọc và báo cáo, không đụng vào bất cứ thứ gì. Bạn sẽ biết Zalo đang chiếm bao nhiêu và nằm ở những thư mục nào.

### Lần dọn đầu tiên nên bắt đầu từ đâu

Chọn **`1`** rồi **`2` — Bản trùng lặp trong Zalo**. Đây là lựa chọn an toàn nhất: mọi tệp bị xóa đều đã được đối chiếu SHA256 toàn bộ nội dung với một bản giống hệt đang được giữ lại. **Bạn không mất tấm ảnh nào**, chỉ mất bản thừa.

Với nhiều người, riêng bước này đã lấy lại được hơn 10 GB.

---

## Bốn nguồn dung lượng

Chọn `1` ở màn hình chính rồi chọn nguồn:

### `1` — Dữ liệu Zalo cũ theo thời gian

Công cụ đo sẵn từng mốc rồi mới hỏi bạn:

```
   1  Trước 01/01/2026        →   18.42 GB  (74,203 tệp)
   2  Cũ hơn 6 tháng          →   24.10 GB  (98,551 tệp)
   3  Cũ hơn 12 tháng         →   12.88 GB  (51,207 tệp)
   4  Tôi tự nhập ngày
```

Mốc nào không có dữ liệu sẽ tự ẩn. Đây là **dữ liệu thật** — ảnh và video quá hạn lưu trên máy chủ Zalo sẽ mất vĩnh viễn, nên bước xác nhận ở đây nặng nhất.

### `2` — Bản trùng lặp trong Zalo

Zalo lưu mỗi tấm ảnh và mỗi video **hai bản**: một bản độc lập trong `video\ picture\ voice\ file\`, một bản trong `resource\<mã hội thoại>\`. Chế độ này tìm bản thừa bên `resource\` và **luôn giữ bản độc lập**.

Quy trình bốn bước, chỉ bước cuối mới có quyền kết luận:

1. Lập chỉ mục các bản giữ lại theo kích thước
2. Lọc ứng viên có kích thước trùng
3. Lọc nhanh bằng chữ ký 64 KB đầu và 64 KB cuối
4. **Xác minh SHA256 toàn bộ nội dung**

Bước 4 không thừa. Trong một lần chạy thực tế, 11 ứng viên trùng kích thước đã bị loại ở bước băm vì nội dung khác nhau. Nếu đoán theo tên hoặc kích thước, 11 tệp đó đã bị xóa oan.

Chế độ này bỏ qua bộ lọc thời gian.

### `3` — Cache của ứng dụng Zalo

Dọn `Cache`, `Code Cache`, `GPUCache`, `media\update`, `media\temp` ở cấp `ZaloData`. Zalo tự tạo lại. **Không chứa tin nhắn hay ảnh video đã nhận.**

### `4` — Cache hệ thống ngoài Zalo

Chế độ duy nhất hoạt động ngoài phạm vi Zalo. Nó chạy trên một **danh sách trắng** các vị trí đã được kiểm chứng, chia ba nhóm:

- **A — cache công cụ lập trình**: npm, pip, uv, cargo, Playwright, Puppeteer, HuggingFace, pre-commit…
- **B — bộ cài đặt thừa**: tệp cài đặt còn sót của Ollama, LM Studio, driver Intel, GIGABYTE
- **C — tệp tạm và trình duyệt**: Temp người dùng, Windows Temp, cache Chrome/Edge/Firefox, ảnh thu nhỏ, INetCache, crash dump, Windows Update đã cài

Công cụ **không bao giờ dò tìm theo mẫu tên** kiểu `*cache*`. Rất nhiều ứng dụng đặt tên thư mục là `Cache` nhưng bên trong là dữ liệu thật không tái tạo được; quét theo mẫu tên chính là cách làm hỏng máy.

Mỗi mục hiển thị dung lượng đo trực tiếp, số tệp, mô tả mất gì khi xóa, và các nhãn cảnh báo:

```
  ── A Cache công cụ lập trình
   [x]  1. pre-commit                664.2 MB   26,803 tệp
           Môi trường hook, tự dựng lại nhưng phải tải từ mạng
   [x]  2. npm                       219.6 MB    6,681 tệp  · đang chạy: node
           Gói npm tải lại khi cài lại
   [ ]  3. NuGet                       1.8 GB   12,004 tệp  · chưa kiểm chứng tận nơi  · có cảnh báo
           Gói NuGet tải lại khi restore
           ! Đây là kho gói dùng chung cho mọi solution .NET trên máy, không phải cache tạm
```

Cách chọn: gõ số, chữ nhóm, hoặc trộn cả hai — ví dụ `A,12,15`. Gõ `-` bỏ chọn hết, `*` chọn tất cả, `ok` để quét, `admin` để mở lại với quyền quản trị. Nhập sai thì giữ nguyên lựa chọn.

**Chọn hàng loạt không kéo theo mục có cảnh báo.** `*` và chữ nhóm bỏ qua chúng và nói rõ đã bỏ mục nào — muốn chọn thì phải cố ý gõ số.

**Ngưỡng tuổi cho thư mục Temp.** Hai mục `Tệp tạm của bạn` và `Tệp tạm của Windows` chỉ xóa tệp cũ hơn 24 giờ. Lý do: `%TEMP%` chứa tệp làm việc của mọi ứng dụng đang mở. Xóa tệp tạm vừa được tạo có thể làm hỏng tiến trình đang chạy theo cách rất khó chẩn đoán, vì tệp không hề bị khóa nên thao tác xóa vẫn thành công.

**Chặn khi ứng dụng đang chạy.** Ở bước `ok`, công cụ đọc lại danh sách tiến trình ngay lúc đó và dừng lại nếu lựa chọn có mục đang bị ứng dụng khác dùng:

```
  3 mục đang được ứng dụng khác sử dụng:
   · npm                      đang chạy: node
   · pip (Python)             đang chạy: python, pythonw
   · Cache Chrome             đang chạy: chrome

   1  Bỏ các mục đó ra, dọn phần còn lại
   2  Cứ dọn hết
   Enter để quay lại, tự đóng ứng dụng rồi thử lại
```

Lý do có bước này: trong một lần đo thực tế, xóa `Ollama\updates_v2` (1.491 MB) lúc 02:52 thì tiến trình Ollama tải lại đúng tệp đó lúc 03:03. Dọn xong không thu được gì mà còn tốn băng thông.

Công cụ **không bao giờ tự tắt ứng dụng của bạn**. Chỉ Zalo mới bị đóng, và việc đó có xác nhận riêng.

---

## Vì sao dọn xong mà ổ đĩa không trống thêm

Đây là điều quan trọng nhất cần hiểu nếu mục tiêu của bạn là lấy lại dung lượng thật chứ không phải một con số đẹp trong báo cáo.

### Cơ chế

Volume Shadow Copy — thứ đứng sau System Restore và Previous Versions — dùng **copy-on-write**. Khi tồn tại một bản chụp và bạn xóa tệp đã có mặt lúc chụp, Windows phải giữ lại nội dung cũ cho bản chụp đó, nên nó **chép các khối dữ liệu sang vùng shadow storage trước khi cho phép xóa**.

Kết quả: bạn giải phóng X byte khỏi hệ thống tệp và tiêu tốn đúng X byte ở vùng chụp. Thư mục co lại thật, nhưng **dung lượng trống của ổ đĩa đứng yên**.

Con số đo được trên một máy thật:

| Việc làm | Dung lượng thu về |
|---|---|
| Xóa 12,96 GB khi đang có bản chụp System Restore | **0,04 GB** |
| Xóa 15,05 GB sau khi đã tắt System Restore cho ổ C | **14,81 GB** |

### Thứ tự thao tác quyết định kết quả

| Thứ tự | Kết quả |
|---|---|
| Dọn xong **rồi mới** nhả bản chụp | Lấy được dung lượng thật |
| Dọn khi đang có bản chụp, không nhả | Mất trắng vào vùng chụp |

Nên dọn trước rồi mới nhả bản chụp, chứ không phải ngược lại: nếu có sự cố trong lúc dọn thì bạn vẫn còn điểm khôi phục để quay lui.

### Công cụ tự phát hiện

Sau mỗi lần xóa, nó đo dung lượng trống **trước và sau** rồi đối chiếu với số byte đã xóa:

```
  Ổ đĩa trước : 43.01 GB
  Ổ đĩa sau   : 43.05 GB
  Thực tế thu được: +40.0 MB
```

Nếu xóa trên 500 MB mà thực tế thu về chưa tới một nửa, công cụ in cảnh báo và chỉ bạn sang phím `V` trong Tùy chọn nâng cao.

### Phím `V` — Shadow Copy

Màn hình này giải thích cơ chế, hiện dung lượng trống hiện tại, và nếu chạy với quyền quản trị thì cho phép:

- **Hạ trần shadow storage** — giữ vài bản chụp gần nhất, chặn việc nuốt tiếp ở các lần dọn sau. Đây là lựa chọn cân bằng nhất.
- **Xóa bản chụp cũ nhất** — giữ các bản mới hơn.
- **Xóa toàn bộ bản chụp** — lấy lại nhiều nhất, nhưng mất hết điểm khôi phục và Previous Versions. Phải gõ nguyên câu `XÓA HẾT BẢN CHỤP` để xác nhận.

Mỗi thao tác đều báo dung lượng trống trước và sau để bạn thấy hiệu quả thật.

Kiểm tra thủ công nếu muốn:

```powershell
vssadmin list shadowstorage
```

> **Mẹo phân biệt:** so kích thước thư mục trước và sau, đừng chỉ nhìn dung lượng trống ổ đĩa. Thư mục co lại đúng bằng con số báo cáo nghĩa là công cụ đã làm đúng việc của nó; phần còn lại là chuyện của shadow copy.

---

## Thiết kế an toàn

### Năm nguyên tắc bất biến

1. **Không quét thì không thể xóa.** Mọi thao tác xóa đều dựa trên một kết quả quét đã có.
2. **Đổi bộ lọc là kết quả quét cũ bị hủy.** Không bao giờ xóa theo một danh sách lỗi thời.
3. **Nhập sai bộ lọc thì giữ nguyên.** Công cụ không bao giờ tự mở rộng phạm vi khi bạn gõ nhầm.
4. **Vùng bảo vệ bị chặn cứng ở tầng code.** Không bộ lọc nào, kể cả bộ lọc do bạn tự đặt, chạm được vào chúng.
5. **Sao lưu chưa sạch thì không cho xóa.**

### Mức xác nhận tương xứng với rủi ro

| Loại dữ liệu | Xác nhận |
|---|---|
| Dữ liệu Zalo thật | Gõ đúng chữ `XÓA` (chấp nhận cả `XOA` không dấu) |
| Bản trùng lặp, cache | Gõ `c` |
| Sao lưu chưa sạch mà vẫn muốn xóa | Gõ nguyên câu `TÔI CHẤP NHẬN MẤT` |
| Xóa hết bản chụp System Restore | Gõ nguyên câu `XÓA HẾT BẢN CHỤP` |

### Vùng bảo vệ

Những nơi công cụ **không bao giờ** xóa, chặn cứng trong mã nguồn:

| Thư mục | Nội dung | Hậu quả nếu xóa |
|---|---|---|
| `ZaloData\Database\` | Cơ sở dữ liệu tin nhắn | Mất lịch sử chat vĩnh viễn |
| `ZaloData\Partitions\` | Dữ liệu phiên đăng nhập | Phải đăng nhập lại |
| `Windows\WinSxS` | Kho thành phần Windows | Hỏng Windows — chỉ được dọn bằng `DISM` |
| `Windows\Installer` | Gói cài đặt phần mềm | Không gỡ hay sửa được phần mềm nữa |
| `Windows\System32`, `SysWOW64`, `servicing`, `assembly` | Nhân hệ điều hành | Hỏng Windows |
| `hiberfil.sys`, `pagefile.sys`, `swapfile.sys` | Tệp hệ thống | Hỏng ngủ đông và bộ nhớ ảo |
| `.cargo\bin`, `.rustup` | Rust đã cài | Phải cài lại Rust — đây **không** phải cache |
| `AppData\Local\Programs`, `Packages` | Ứng dụng đã cài | Hỏng ứng dụng |
| Thư mục chứa chính công cụ | | Công cụ không tự xóa được mình |

Việc chặn được kiểm tra ở ba nơi độc lập: lúc quét, lúc dọn thư mục rỗng, và một lần nữa ngay trước từng thao tác xóa.

**Hai mức bảo vệ.** Phím `B` trong Tùy chọn nâng cao liệt kê đầy đủ:

| Mức | Ý nghĩa |
|---|---|
| `tất cả` | Chặn chính nó và mọi thứ bên dưới — mức của toàn bộ bảng trên |
| `gốc` | Chỉ chặn khi nhắm **thẳng** vào chính thư mục đó; con vẫn dọn được |

Mức `gốc` phủ lên `%WINDIR%`, `%USERPROFILE%`, `%APPDATA%`, `%LOCALAPPDATA%`, `%ProgramData%`, `%ProgramFiles%` và gốc ổ hệ thống. Nó là lưới chắn cho `catalog.json`: một mục ghi nhầm `"%LOCALAPPDATA%"` sẽ bị loại, còn `"%LOCALAPPDATA%\npm-cache"` vẫn dọn được như thường.

**Chặn cả chiều ngược.** Nhận một thư mục *chứa* vùng bảo vệ cũng nguy hiểm y như nhận chính vùng bảo vệ. Với các thư mục gốc, công cụ hỏi thêm chiều này: `%WINDIR%` bị chặn vì nó chứa `WinSxS`, dù bản thân `%WINDIR%` không nằm trong bảng.

**Junction và symbolic link.** Công cụ không bao giờ xóa hay dọn xuyên qua một reparse point. Junction trỏ tới thư mục rỗng trông y hệt thư mục rỗng, và một junction bị chặn quyền đọc cũng vậy — xóa đệ quy lên chúng có thể xóa xuyên sang đầu bên kia. Việc dọn thư mục rỗng cũng **không dùng lệnh xóa đệ quy**: giữa lúc kết luận "thư mục này rỗng" và lúc ra lệnh xóa có một khe hở, và lệnh đệ quy sẽ cuốn theo tệp vừa được ghi vào mà không qua vùng bảo vệ.

### Nhật ký

Nằm trong `logs\` cạnh script. `daxoa_<thời gian>.log` ghi từng tệp với một trong các trạng thái:

| Trạng thái | Ý nghĩa |
|---|---|
| `ĐÃXÓA` | Xóa thành công |
| `CẮTCỤT` | Tệp đang bị khóa, đã cắt về 0 byte — xem bên dưới |
| `THẤTBẠI` | Không xóa được |
| `BIẾNMẤT` | Tệp đã biến mất trước khi công cụ chạm tới |
| `VÙNGBẢOVỆ` | Bị chặn bởi vùng bảo vệ |

Công cụ chỉ tính là **đã xóa** khi tệp thật sự biến mất sau lệnh xóa. Tệp bị tiến trình khác xóa trước đó được đếm riêng ở `BIẾNMẤT` chứ không cộng vào thành tích.

Bấm `Ctrl+C` giữa chừng là an toàn: nhật ký được ghi liên tục nên mất tối đa 99 dòng cuối, và dòng tổng kết luôn ghi rõ đã hủy giữa chừng.

Ngoài ra có `khoiphuc_*.log`, `saoluu_loi_*.txt`, và `quet_*.csv` khi bạn xuất danh sách. Phím `L` tổng hợp toàn bộ lịch sử và cho xóa nhật ký cũ.

> Nhật ký chứa **đường dẫn đầy đủ tới từng tệp trên máy bạn**. Thư mục `logs/` đã nằm trong `.gitignore` — đừng đưa nó lên đâu cả.

### Cắt cụt tệp bị khóa

Tệp đang bị tiến trình khác giữ thì xóa không được. Nhưng nếu tiến trình ấy mở ở chế độ chia sẻ, công cụ vẫn ghi đè được: nó cắt tệp về 0 byte, thu lại đủ dung lượng, chỉ còn sót cái tên rỗng mà chủ của nó sẽ tự dọn hoặc tự ghi lại.

**Chỉ áp dụng cho cache Zalo và cache hệ thống.** Cắt cụt một tệp đang được dùng có thể làm hỏng ứng dụng đang giữ nó theo kiểu rất khó chẩn đoán, trong khi xóa thất bại thì vô hại. Với cache thì đánh đổi đó đáng; với dữ liệu Zalo thật thì không.

Không có bước nào đánh dấu xóa khi khởi động lại. Mọi thao tác của công cụ đều kết thúc trước khi bạn đóng cửa sổ.

---

## Sao lưu và khôi phục

Sao lưu **không bắt buộc**. Bạn chọn cách công cụ cư xử qua phím `C`, và lựa chọn được ghi nhớ cho các lần chạy sau:

| Chính sách | Hành vi |
|---|---|
| Hỏi mỗi lần (mặc định) | Trước mỗi lần xóa dữ liệu thật, công cụ hỏi bạn muốn sao lưu trước hay xóa luôn |
| Không hỏi | Chỉ còn xác nhận bằng chữ `XÓA` |
| Bắt buộc | Không có bản sao lưu sạch cho lần quét đó thì không xóa được |

### Sao lưu

Phím `9` kiểm tra dung lượng ổ đích **trước** khi chép. Thiếu chỗ thì dừng hẳn, không tạo thư mục nào.

Sau khi chép, công cụ xác minh ở hai mức:

- Đối chiếu kích thước cho **toàn bộ** tệp
- Đối chiếu SHA256 cho mẫu 50 tệp (nhanh) hoặc toàn bộ (chắc chắn tuyệt đối)

Nếu chép lỗi hoặc xác minh lỗi dù chỉ một tệp, **bước xóa bị khóa**. Muốn vượt qua phải gõ đúng câu `TÔI CHẤP NHẬN MẤT`.

Mỗi bản sao lưu chứa tệp chỉ mục `_zalocleanup_backup.json` ghi lại vị trí gốc, thời điểm, số tệp và kết quả xác minh. Đây là thứ cho phép khôi phục.

### Khôi phục

Mục **`3`** ở màn hình chính. Công cụ **tự đi tìm** các bản sao lưu thay vì bắt bạn nhớ đường dẫn: nó tra các thư mục từng dùng rồi quét nông các ổ đĩa để tìm tệp chỉ mục.

Mỗi bản được mô tả đủ để bạn biết mình đang chọn cái gì:

```
   1  01/08/2026 12:00:00   7 tệp · 7.2 MB
      Nội dung : DỮ LIỆU ZALO
      Gồm      : video 6.0 MB · picture 1.2 MB
      Loại tệp : không đuôi (3) · .jxl (4)
      Tệp từ   : 01/04/2025 đến 20/11/2025
      Nằm ở    : D:\SaoLuuZalo\20260801_120000
      Trả về   : ...\ZaloDownloads
```

Gõ `x 1` để xem ba tệp lớn nhất bên trong trước khi quyết định.

Công cụ so phần cần ghi với dung lượng trống **trước khi ghi byte nào**. Thiếu chỗ thì dừng hẳn thay vì làm liều — khôi phục được nửa chừng rồi hết chỗ sẽ để lại trạng thái dở dang, khó biết tệp nào đã về tệp nào chưa. **Bản sao lưu không bao giờ bị đụng đến** trong quá trình khôi phục, nên chạy lại sau khi có chỗ là an toàn.

---

## Tham chiếu

### Menu Tùy chọn nâng cao (phím `9`)

**Bộ lọc**

| Phím | Chức năng |
|---|---|
| `1` | Khoảng thời gian. Chấp nhận `31/12/2025`, `2025-12-31`, `31122025` |
| `2` | Thư mục con cần **bao gồm**. Gõ `*` để chọn tất cả (phải cố ý) |
| `3` | Đuôi tệp cần bao gồm. Gõ `(khong duoi)` để bắt tệp không có phần mở rộng — video Zalo thường ở dạng này |
| `4` | Kích thước tối thiểu tính bằng KB. Dùng khi muốn nhắm video nặng trước |
| `5` | **Loại trừ** — thư mục, đuôi tệp, và bật/tắt việc giữ tệp `.rescache` |
| `6` | Hồ sơ bộ lọc — lưu và nạp lại bộ lọc đã đặt tên |

> Mặc định bộ lọc thời gian là **mọi thời điểm** — công cụ không tự thu hẹp phạm vi quét thay bạn. Muốn nhắm tệp cũ thì đặt mốc bằng phím `1`, hoặc dùng luồng hướng dẫn ở màn hình chính vốn đã hiện sẵn dung lượng từng mốc.

**Quét và thao tác**

| Phím | Chức năng |
|---|---|
| `7` | Quét theo bộ lọc |
| `8` | Xem chi tiết kết quả quét, xuất toàn bộ danh sách ra CSV |
| `9` | Sao lưu kết quả quét sang ổ khác, kèm bước xác minh |
| `X` | Xóa kết quả quét đang giữ |
| `K` | Khôi phục từ một bản sao lưu |

**Thông tin và cài đặt**

| Phím | Chức năng |
|---|---|
| `V` | Shadow Copy — giải thích cơ chế và lấy lại dung lượng thật |
| `B` | Báo cáo vùng bảo vệ |
| `L` | Lịch sử dọn dẹp, kèm xoay vòng nhật ký |
| `C` | Chính sách sao lưu |
| `T` | Đổi tài khoản Zalo |
| `0` | Quay lại |

### Zalo cất dữ liệu ở đâu

```
%APPDATA%\ZaloData\
├── media\<mã tài khoản>\ZaloDownloads\   ← nơi công cụ làm việc
│   ├── video\  picture\  voice\  file\   ← bản độc lập
│   └── resource\<mã hội thoại>\          ← bản thứ hai của cùng nội dung
├── Database\                             ← VÙNG BẢO VỆ, tin nhắn của bạn
└── Partitions\                           ← VÙNG BẢO VỆ, phiên đăng nhập
```

Công cụ tự dò mọi tài khoản Zalo trên máy. Máy có nhiều tài khoản thì nó hỏi bạn chọn, và bạn đổi được bất cứ lúc nào bằng phím `T`.

Hai bản của cùng một tấm ảnh khớp nhau bằng đoạn hash cuối tên tệp — đó là cơ sở của chế độ khử trùng lặp.

### Mở rộng danh mục bằng `catalog.json`

Danh mục cache hệ thống nằm ở [`catalog.json`](catalog.json), sửa được mà không đụng mã nguồn. Xóa hoặc làm hỏng tệp này thì công cụ tự quay về danh mục dựng sẵn trong mã nguồn.

```json
{
  "group": "A",
  "name": "npm",
  "verified": true,
  "risk": "VÀNG",
  "paths": ["%LOCALAPPDATA%\\npm-cache"],
  "procs": ["node"],
  "note": "Gói npm tải lại khi cài lại"
}
```

| Trường | Bắt buộc | Ý nghĩa |
|---|---|---|
| `name` | ✔ | Tên hiển thị |
| `paths` | ✔ | Mảng đường dẫn. Dùng biến môi trường kiểu `%LOCALAPPDATA%`, chấp nhận dấu `*` |
| `group` | | `A`, `B` hoặc `C` |
| `risk` | | `XANH` hoặc `VÀNG`. Vàng nghĩa là phải tải lại từ mạng |
| `verified` | | `true` nghĩa là vị trí đã được đo tận nơi trên máy thật của người phát triển; `false` nghĩa là chỉ dựa vào tài liệu của ứng dụng, và công cụ sẽ hiện nhãn **chưa kiểm chứng tận nơi** |
| `note` | | Mô tả mất gì khi xóa |
| `procs` | | Mảng tên tiến trình. Dùng cho việc chặn khi ứng dụng đang chạy |
| `ageHours` | | Chỉ xóa tệp cũ hơn ngần ấy giờ |
| `warning` | | Cảnh báo. Mục có trường này không được `*` hay chữ nhóm chọn vào |

Mục sai định dạng **được nêu tên kèm lý do** chứ không bị bỏ qua im lặng:

```
  2 mục trong catalog.json bị bỏ qua vì sai định dạng:
   "INetCache" — thiếu "paths" (có phải bạn gõ "path"?)
   "Báo cáo lỗi ứng dụng" — "group" phải là A, B hoặc C — đang là "Z"
```

Các mục còn lại vẫn nạp bình thường. Mục nào không tồn tại trên máy sẽ tự ẩn, nên danh sách luôn gọn theo đúng phần mềm bạn đang cài.

### Tham số dòng lệnh

```powershell
# Chạy trên một thư mục khác — hữu ích khi thử nghiệm
.\ZaloCleanup.ps1 -Root "D:\thu-muc-khac"

# Chỉ định thẳng thư mục ZaloData thay vì để công cụ tự dò
.\ZaloCleanup.ps1 -DataRoot "D:\ZaloData"
```

Không có tham số nào chạy chế độ không tương tác. Đó là chủ ý: mọi thao tác xóa đều phải có người ngồi trước màn hình.

---

## Xử lý sự cố

**Chữ tiếng Việt hiển thị thành ô vuông hoặc ký tự lạ**
Đổi font của cửa sổ Console sang một font có dấu tiếng Việt — bấm chuột phải lên thanh tiêu đề → Properties → Font → chọn *Consolas* hoặc *Cascadia Mono*.

**"cannot be loaded because running scripts is disabled on this system"**
Bạn đang chạy thẳng tệp `.ps1`. Dùng `ZaloCleanup.cmd` thay thế, hoặc chạy lệnh ở mục [Cài đặt](#nếu-bạn-muốn-chạy-trực-tiếp-tệp-ps1).

**Nhiều tệp báo `THẤTBẠI`**
Zalo hoặc một ứng dụng khác vẫn đang giữ chúng. Đóng ứng dụng liên quan rồi quét lại. Nếu là cache thì công cụ sẽ tự cắt cụt để vẫn thu được dung lượng.

**Dọn xong mà ổ đĩa không trống thêm**
Đọc mục [Vì sao dọn xong mà ổ đĩa không trống thêm](#vì-sao-dọn-xong-mà-ổ-đĩa-không-trống-thêm). Gần như chắc chắn là Shadow Copy.

**Báo `[cần quyền quản trị]` ở một số mục**
Gõ `admin` ngay trong màn hình cache hệ thống để mở lại công cụ ở chế độ nâng quyền.

**Windows Defender chặn giữa chừng**
Controlled Folder Access đang bật. Công cụ phát hiện và cảnh báo trước khi xóa. Bạn có thể tắt tạm hoặc thêm PowerShell vào danh sách cho phép.

**Máy chưa cài Zalo**
Công cụ vẫn chạy, tự ẩn ba mục liên quan Zalo và nói rõ lý do. Phần cache hệ thống, Shadow Copy, khôi phục vẫn dùng bình thường.

---

## Phát triển

### Chạy bộ test

```bash
powershell -NoProfile -ExecutionPolicy Bypass -File ".\ZaloCleanup.Tests.ps1" -Full
```

149 phép thử. Bộ test tự dựng sandbox trong `%TEMP%`, **không bao giờ đụng vào dữ liệu Zalo thật**, và tự dọn sau khi chạy.

Nó kiểm chứng những thứ mà một công cụ xóa tệp buộc phải đúng: bộ lọc không tự mở rộng khi nhập sai, sao lưu lỗi chặn được xóa, thiếu chỗ chặn được sao lưu, đếm đúng khi tệp biến mất giữa chừng, khử trùng lặp chỉ xóa bản đã xác minh hash, vùng bảo vệ không bị chạm tới ở cả hai chiều, quét không đi xuyên junction, dọn thư mục rỗng không xóa junction cũng không đụng đích của nó, thư mục hết rỗng vào phút chót thì không bị xóa, và mục sai trong `catalog.json` được nêu tên.

**Chạy bộ test này sau mỗi lần sửa mã nguồn.**

### Quy ước khi sửa mã

- **Mọi tệp `.ps1` phải lưu dạng UTF-8 CÓ BOM.** Thiếu BOM thì PowerShell 5.1 đọc theo bảng mã ANSI và mọi chữ có dấu sẽ vỡ. Bộ test kiểm tra điều này.
- Giữ tương thích PowerShell 5.1. Không dùng cú pháp của PowerShell 7 (`??`, `?:`, `&&`, `||`).
- Không thêm phụ thuộc ngoài. Không `Add-Type`, không P/Invoke, không mô-đun bên thứ ba. Chép thư mục sang máy khác phải chạy được ngay.
- Không đăng ký Scheduled Task, không tạo tiến trình nền, không thêm hành vi xảy ra sau khi người dùng đóng cửa sổ. Đây là ràng buộc nền tảng của dự án, không phải sở thích.
- Thêm tính năng có hành vi xóa thì phải kèm test hồi quy tương ứng.

### Đóng góp

Issue và pull request đều được hoan nghênh. Hữu ích nhất là:

- **Vị trí cache mới cho `catalog.json`** — kèm theo đường dẫn đầy đủ và mô tả mất gì khi xóa. Nếu bạn đã tự kiểm chứng trên máy mình, nói rõ để đặt `verified: true`.
- **Báo cáo trên bản Windows khác** — công cụ được phát triển trên Windows 11. Kết quả trên Windows 10 rất đáng biết.
- **Cấu trúc thư mục Zalo ở phiên bản khác** — nếu Zalo trên máy bạn cất dữ liệu khác đi, đó là thông tin quan trọng.

---

## Giới hạn đã biết

- Khử trùng lặp chỉ đối chiếu `resource\` với các thư mục độc lập. Không tìm bản trùng nằm hoàn toàn bên trong `resource\`.
- Công cụ đọc ngày sửa đổi cuối (`LastWriteTime`), không phải ngày nhận tin nhắn. Hai giá trị này thường trùng nhau nhưng không phải luôn luôn.
- Việc đo dung lượng thư mục được nhớ đệm; bấm `2` ở màn hình chính rồi chọn đo lại nếu muốn số liệu mới.
- Cắt cụt tệp bị khóa chỉ chạy ở hai chế độ cache, và chỉ ăn thua với tệp được mở ở chế độ chia sẻ. Tệp bị khóa độc quyền vẫn không đụng được.
- Mức bảo vệ `gốc` là lưới chắn cho lỗi gõ nhầm trong `catalog.json`, không phải rào cản cho một mục cố tình trỏ sâu vào chỗ không nên đụng.
- Chỉ có giao diện dòng lệnh tiếng Việt. Chưa có bản tiếng Anh và chưa có giao diện đồ họa.

---

## Miễn trừ trách nhiệm

**Đây là dự án độc lập, không liên quan tới VNG Corporation hay đội ngũ phát triển Zalo.** "Zalo" là thương hiệu của VNG. Công cụ này không sửa, không can thiệp, không đọc nội dung tin nhắn của bạn — nó chỉ xóa tệp media trong thư mục tải về, và không bao giờ chạm vào cơ sở dữ liệu tin nhắn.

**Xóa tệp là thao tác không hoàn tác được.** Công cụ không đưa tệp vào Thùng rác. Ảnh và video đã quá hạn lưu trên máy chủ Zalo sẽ mất vĩnh viễn. Hãy dùng chức năng sao lưu nếu bạn không chắc, và luôn xem kỹ danh sách trước khi xác nhận.

Phần mềm được cung cấp "nguyên trạng", không kèm bảo đảm nào. Xem [LICENSE](LICENSE).

---

## Giấy phép

[MIT](LICENSE)

---

## English summary

**zalo-cleanup** is a Windows tool that reclaims disk space taken by [Zalo](https://zalo.me), Vietnam's dominant messaging app, which stores every received photo and video on your system drive indefinitely. It routinely accumulates tens of gigabytes.

The interface is in Vietnamese, matching its intended users.

**What makes it different from a generic cleaner:**

- **Scan-then-delete is enforced.** Nothing can be deleted that you have not seen listed first. Changing a filter invalidates the previous scan.
- **Duplicate detection verifies content.** Zalo stores each media file twice — once standalone, once under `resource\<conversationId>\`. Candidates are matched by size, then by a 64 KB head/tail signature, then confirmed by a **full-file SHA-256**. Only the last step is allowed to conclude. In one real run, 11 same-size candidates were rejected at the hash step.
- **Backups are verified before deletion is permitted.** Size is checked for every file plus SHA-256 on a sample or the full set. A single copy or verification failure blocks the delete step.
- **It tells the truth about Volume Shadow Copy.** Deleting files while a System Restore snapshot exists moves the freed blocks into shadow storage instead of returning them to the volume — measured: deleting 12.96 GB returned 0.04 GB. The tool measures free space before and after every run and warns when the numbers disagree.
- **Hard-coded protected zones** cover the Zalo message database, Windows system directories, and installed toolchains, checked in both directions (a path *containing* a protected zone is refused too) and never traversing junctions or symlinks.
- **Manual only, by design.** No scheduled task, no background service, no delete-on-reboot. Every action completes before you close the window.

Requires Windows 10/11 and the built-in PowerShell 5.1. No installation, no dependencies. Ships with a 149-case regression suite that builds its own sandbox in `%TEMP%`.

Not affiliated with VNG Corporation. MIT licensed.
