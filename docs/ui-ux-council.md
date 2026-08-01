# BẢN THIẾT KẾ CHỐT — GIAO DIỆN ĐỒ HỌA `zalo-cleanup` (bản Rust)

> Thư ký hội đồng tổng hợp. Đầu vào: 5 ghế + 36 đường tấn công từ 3 lăng kính phản biện.
> Nguồn đối chiếu đã đọc lại tận nơi: `D:\zalo-tool\docs\rust-port-brief.md`, `README.md`, `ZaloCleanup.ps1`, `catalog.json`.
> Tài liệu này là đầu vào trực tiếp cho phiên lập kế hoạch port. Mọi mục có mã số để phiên sau trích dẫn.
> Ngày chốt: 01/08/2026.

---

## 0. Ba câu tóm tắt cả bản thiết kế

1. **Ma sát của bản dòng lệnh nằm ở ĐƯỜNG ĐI; giao diện xóa sạch đường đi nên phải trả lại bằng BẰNG CHỨNG** — nút Xóa xám cho tới khi người dùng đã thật sự nhìn thấy cái sắp mất, và mọi con số trên nút đều là số sống.
2. **Rủi ro lớn nhất KHÔNG phải người bấm vội mà là người bấm rất bình tĩnh, đọc hết mọi thứ, và tin vào một câu công cụ chưa bao giờ chứng minh.** Bốn câu nguy hiểm nhất đã bị cấm: "SẠCH" đứng một mình, "An toàn nhất — không mất tấm ảnh nào", "Xác minh: 0 lỗi", "Dung lượng xóa được sẽ về ổ đĩa".
3. **Đường lui (sao lưu · khôi phục · bản chụp hệ thống) là nhánh yếu nhất của cả công cụ.** 22 trên 36 đường tấn công nằm ở đó. Toàn bộ nhánh này được siết lại: đo lại thay vì nhớ, chặn cứng vị trí đích, xác minh cả hai chiều, và cấm giao diện dụ người dùng phá đường lui trước khi làm việc nguy hiểm.

---

# 1. QUYẾT ĐỊNH ĐÃ CHỐT

## 1.1 Khung và nền tảng

| # | Quyết định | Căn cứ |
|:-:|:---|:---|
| **QĐ-01** | **egui / eframe.** Không Tauri, không iced, không Win32 thuần cho vỏ. | Đo thật: egui 2,86 MiB không cần runtime; iced 6,00 MiB + **không có accesskit** → loại theo tiêu chí trình đọc màn hình; Tauri 4,36 MiB nhưng cần WebView2 (677 MB) → phá lời hứa tải-về-chạy-ngay |
| **QĐ-02** | **Win32 gốc ở đúng 4 chỗ**: `ShellExecuteExW` verb `runas`; `TaskDialogIndirect` cho cảnh báo chỉ-có-nút; `GetDiskFreeSpaceExW`; `CreateToolhelp32Snapshot` + API liệt kê handle (RmGetList) để biết tệp nào đang bị giữ | egui không thay thế được. **Lưu ý: `TaskDialog` KHÔNG có ô nhập chữ** — mọi cửa gõ cụm từ phải do egui tự vẽ |
| **QĐ-03** | **Nạp font hệ thống lúc khởi động**: `segoeui → arial → tahoma`. Không tìm được thì báo lỗi rõ ràng, không im lặng vẽ ô vuông | Font nhúng sẵn của egui có **1/10** ký tự tiếng Việt (Ubuntu-Light) và 2/10 (Hack). Giá: +14.336 byte |
| **QĐ-04** | **Một tệp exe, một cửa sổ, một tiến trình.** Không tab, không cửa sổ phụ, không khay hệ thống, không tiến trình nền | Công cụ chỉ có MỘT trạng thái quét toàn cục; hai bối cảnh song song là đường nhanh nhất phá nguyên tắc bất biến 2 |
| **QĐ-05** | **Khóa liên tiến trình dùng chung với bản PowerShell** (mutex đặt tên + tệp khóa mang PID). Bản `.ps1` phải được sửa để lấy cùng khóa | Đường tấn công B8: nút "Mở bản dòng lệnh" của lớp trợ năng tạo ra hai tiến trình cùng thao tác trên một tập tệp |
| **QĐ-06** | **Kích thước cửa sổ: tối thiểu 940 × 560 dip, mặc định 1120 × 760.** Phải vừa 1366×768 @125% (≈1093×574 dip khả dụng) không cuộn ngang | Hòa giải ba con số của ba ghế (720 / 960×640 / 1093×614) — xem M-10 |
| **QĐ-07** | **Manifest PerMonitorV2 nhúng trong exe.** Thiếu là dấu tiếng Việt nhòe thành vệt ở 125% trở lên | DPI-01 |

## 1.2 Mô hình xác nhận

| # | Quyết định |
|:-:|:---|
| **QĐ-08** | **Sáu mức rủi ro R0–R4 đo bằng KHẢ NĂNG DỰNG LẠI, không bằng dung lượng.** Lô hỗn hợp lấy mức của mục nguy hiểm nhất |
| **QĐ-09** | **Sáu dạng xác nhận X0–X5.** X0 không hỏi · X1 một nút ma sát tĩnh · X2 hộp hậu quả + ô tick mang số liệu · X3 bằng chứng bắt buộc · X4 X3 + gõ cụm từ · X5 chặn cứng ở lõi |
| **QĐ-10** | **BA CHỐT cho mọi hành động xóa**: (a) kết quả quét còn hiệu lực · (b) màn hình Xem trước của **đúng lượt quét đó** đã được mở và đã đọc tới cuối · (c) đã gõ đúng cụm từ theo mức rủi ro. Thiếu chốt nào thì nút xám và **nói rõ đang thiếu chốt nào** |
| **QĐ-11** | **Đúng bốn cụm từ gõ tay trong toàn dự án**, mỗi cụm nhận ba kiểu viết: `XÓA / XOÁ / XOA` · `TÔI CHẤP NHẬN MẤT / …` · `GHI ĐÈ / GHI DE` · `XÓA HẾT BẢN CHỤP / XOÁ… / XOA HET BAN CHUP`. **Không có cụm thứ năm.** Không dùng cụm gõ tay cho bất cứ thứ gì dưới R4 |
| **QĐ-12** | **Đúng MỘT cửa xác nhận cho mỗi hành động.** Cấm chuỗi hộp thoại nối tiếp. Trạng thái sao lưu là một KHỐI trong trang xác nhận, không phải một bước riêng |
| **QĐ-13** | **Bỏ hẳn đếm ngược.** Chỗ duy nhất từng được đề xuất (xóa toàn bộ bản chụp) thay bằng **liệt kê NGÀY của từng bản chụp sắp mất** — danh sách dạy người ta hiểu, đồng hồ chỉ dạy người ta chờ |
| **QĐ-14** | **Cổng "đã xem" gắn vào NGƯỠNG TUYỆT ĐỐI, không gắn vào màu rủi ro**: mọi lô **> 1.000 tệp hoặc > 1 GB** đều bắt mở danh sách, kể cả lô toàn mục XANH |

## 1.3 Điều hướng

| # | Quyết định |
|:-:|:---|
| **QĐ-15** | **Trang chủ là màn hình gốc duy nhất, độ sâu tối đa 2 tầng.** Bốn nguồn dung lượng lên thẳng Trang chủ dưới dạng thẻ; bỏ tầng menu trung gian `1 Lấy lại dung lượng ổ đĩa` (tầng đó không chứa ma sát an toàn nào) |
| **QĐ-16** | **Trang chủ KHÔNG chứa bất kỳ nút nào xóa dữ liệu.** Không "Dọn ngay", không "Quét toàn bộ", không tổng gộp bốn nguồn |
| **QĐ-17** | **Thứ tự bốn thẻ theo rủi ro tăng dần**: 🟢 Bản trùng lặp → 🟢 Cache Zalo → 🟡 Cache hệ thống → 🔴 Dữ liệu Zalo cũ. Ở giao diện, **vị trí thứ nhất là một lời khuyên** |
| **QĐ-18** | **Băng kết quả quét (BQ) hiện trên MỌI màn hình**, chiều cao cố định. Trên Khôi phục · Xem dung lượng · Vùng bảo vệ thì **thu còn một dòng chữ, không mang nút Xóa, không mang nút Sao lưu** |
| **QĐ-19** | **Bảy hộp thoại chặn, không hơn**: M1 xác nhận xóa · M2 đóng Zalo · M3 nâng quyền · M4 ứng dụng đang chạy · M5 thao tác bản chụp · **M6 cắt cụt tệp bị khóa (mới)** · **M7 sao lưu không trọn vẹn (mới)**. Mọi thứ khác là màn hình hoặc khối trong màn hình |
| **QĐ-20** | **Không màn hình chào, không bài hướng dẫn, không modal lúc khởi động.** Mở exe là vào thẳng Trang chủ |

## 1.4 Đường lui (siết lại sau vòng phản biện)

| # | Quyết định |
|:-:|:---|
| **QĐ-21** | **Trạng thái sao lưu là kết quả của một phép ĐO LẠI, không phải một biến nhớ.** Đo lại khi cửa sổ lấy tiêu điểm, mỗi 30 giây, ngay trước khi mở cửa xác nhận, và lần nữa ngay trước byte đầu tiên bị xóa |
| **QĐ-22** | **Đích sao lưu bị CHẶN CỨNG** nếu nằm trong/chứa ScanRoot, DataRoot, hoặc bất kỳ đường dẫn nào của 33 mục `catalog.json`, `%TEMP%`, `%WINDIR%\Temp`, `%LOCALAPPDATA%`, `%APPDATA%`. Chặn ở nút, không phải cảnh báo bằng chữ |
| **QĐ-23** | **Mọi thư mục chứa `_zalocleanup_backup.json` và toàn bộ cây con trở thành VÙNG BẢO VỆ tự động**, chặn ở tầng lõi, hiện trong bảng Vùng bảo vệ, không có công tắc tắt |
| **QĐ-24** | **Cấm chữ "SẠCH" đứng một mình.** Huy hiệu sao lưu luôn mang độ phủ thật: `khớp kích thước 12.418/12.418 · đối chiếu nội dung 50/12.418 (0,4%)` |
| **QĐ-25** | **Bản chụp System Restore chỉ được phép nhả SAU khi đã dọn xong.** Ba lựa chọn phá hủy bản chụp bị xám khi công cụ đang giữ kết quả quét hoặc máy chưa có lần dọn nào hoàn tất; chúng mở khóa từ trang Kết quả |
| **QĐ-26** | **Ghi trước khi xóa**: đổ toàn bộ tập sắp xóa ra `logs\sapxoa_<mã lượt quét>.csv` và ép xuống đĩa TRƯỚC khi chạm tệp đầu tiên |
| **QĐ-27** | **Cắt cụt tệp bị khóa: mặc định TẮT**, không bao giờ chạy im lặng trong vòng lặp, `%TEMP%` và `%WINDIR%\Temp` bị loại khỏi phạm vi cắt cụt ở tầng lõi |

---

# 2. XỬ LÝ MÂU THUẪN GIỮA CÁC GHẾ

Mười một chỗ hai ghế nói ngược nhau. Mỗi chỗ chốt một bên, không để hai văn bản cùng sống.

| # | Mâu thuẫn | Bên A | Bên B | **CHỐT** | Vì sao |
|:-:|:---|:---|:---|:---|:---|
| **M-01** | Định nghĩa cổng "đã xem" | An toàn: mở **và cuộn tới cuối** | Tương tác §4.4: tab Danh sách được **vẽ một khung** | **Bên A, siết thêm theo C13**: cổng mở khi (i) tab Tổng quan đã vẽ, **và** (ii) tab **Đáng chú ý** đã mở + cuộn hết nếu tab đó khác 0; tab đó rỗng thì lùi về tab Danh sách với điều kiện dòng cuối của tập sắp xóa đã từng nằm trong vùng nhìn thấy, hoặc đã bấm Xuất CSV. Với lô > 5.000 dòng, chấp nhận thay (ii) bằng: tab **Lớn nhất** + tab **Đáng chú ý** đều đã cuộn hết | Bản yếu hơn hạ toàn bộ ma sát xuống 5 cú bấm (A7). Tab Đáng chú ý là tab duy nhất nói được điều người dùng CHƯA biết |
| **M-02** | Số bước của cửa xác nhận xóa | An toàn N3: **một cửa duy nhất** | Tương tác I1: **bước 1 ba nút, bước 2 gõ chữ** | **Bên A.** Một trang. Lựa chọn "xóa luôn, không sao lưu" **không được là một cái nút** — nó là một ô tích mang số liệu | Nút giữa trong hàng ba nút là chỗ tay rơi vào theo vị trí; nó lại nằm TRƯỚC cửa gõ chữ nên trôi qua trước khi người dùng kịp tập trung (A9) |
| **M-03** | Nút "Tiếp tục phần còn lại" | Tương tác §3.11: **có** | Kiến trúc TT: **"đây là chỗ dễ cài một đường tắt nguy hiểm nhất, nên bịt luôn"** | **Bên B — bỏ hẳn.** Sau khi dừng, đường duy nhất đi tiếp là Quét lại | Nút đó hồi sinh một lượt quét đã bị tiêu thụ một phần, mang theo cờ đã-xem cũ, và người dùng bấm Dừng chính vì muốn NHÌN (B6) |
| **M-04** | BQ có mang nút Xóa trên KP / DL / NC không | Kiến trúc TT để ngỏ, nghiêng về không | — | **Không.** Trên KP, DL, NC-3, NC-4, NC-5: BQ thu còn một dòng chữ | Sau khi khôi phục xong, một nút Xóa sáng ngay trước mắt là bước tiếp theo hiển nhiên nhất — và nó xóa lại đúng những tệp vừa cứu về (A6) |
| **M-05** | "Chỉ định thư mục ZaloData" có kiểm không | Kiến trúc TT: mô tả **không kèm phép kiểm nào** | Tương tác: "có chạy `Test-ProtectedRoot`" | **Bên B, cộng một lớp nữa.** Mọi đường đổi gốc đi qua **đúng một hàm** có hai lớp chặn cứng: `Test-ProtectedRoot` + kiểm dấu hiệu cấu trúc ZaloDownloads. Hàm đó có test canh | Chỉ vào `C:\Users\Minh\Documents` thì `Test-Protected` trả false, giao diện đổi nhãn thành "ẢNH VÀ VIDEO THẬT của bạn", cột đường dẫn cắt mất gốc, và không chỗ nào trên màn hình hiện chữ "Documents" (B3) |
| **M-06** | Khung giao diện | Tiếp cận: loại iced (issue #552 còn mở) | Khả thi: chọn egui theo số đo | **Đồng thuận — egui.** Ghi thành QĐ-01 | Hai ghế cùng kết luận, khác căn cứ |
| **M-07** | Mặc định của công tắc cắt cụt | An toàn #13: **mặc định BẬT cho cache**, công bố trong hộp xác nhận | Chính An toàn định nghĩa X1 = **một nút, không có hộp thoại** | **Mặc định TẮT.** Và: ở mức X1, mọi hành vi ghi-đĩa ngoài "xóa tệp trong danh sách" (cắt cụt, dọn thư mục rỗng) **bị tắt cứng**. Không có hộp thì không có công bố; không có công bố thì không được làm | Thiết kế tự mâu thuẫn: hành vi mặc định BẬT mà không có chỗ nào để được đồng ý (B9). Chú thích trong mã còn mạnh hơn giao diện: "có thể làm hỏng ứng dụng đang giữ nó theo kiểu rất khó chẩn đoán" (C10) |
| **M-08** | Đếm ngược 5 giây cho nhóm bản chụp | An toàn N6: dùng khi không xem trước được | Tương tác: **loại — "dạy người ta chờ, không dạy người ta đọc"** | **Bên B.** Thay bằng **liệt kê ngày của từng bản chụp sắp mất** — thứ đó xem trước được, nên điều kiện của N6 không thỏa | N6 tự nó nói: chỉ dùng đồng hồ khi KHÔNG thể xem trước. Danh tính bản chụp thì xem trước được (A4d) |
| **M-09** | Ngưỡng bật thẻ giải thích Shadow Copy | Cả hai chép nguyên: **xóa > 500 MB và thu về < 50%** | — | **Đổi sang tỉ lệ thuần: thu được < 50% đã xóa thì luôn hiện thẻ, bất kể lớn nhỏ.** Chênh lệch quá nhỏ để kết luận thì thẻ **đổi giọng**, không biến mất | Giao diện phóng to TRIỆU CHỨNG (con số to nhất màn hình) mà bê nguyên ngưỡng của bản dòng lệnh, nơi nó chỉ là dòng chữ xám nhỏ (C7) |
| **M-10** | Kích thước cửa sổ tối thiểu | An toàn: 720 dip · Tương tác: 960×640 · Tiếp cận: phải vừa 1093×574 | — | **940 × 560 dip.** 960×640 KHÔNG vừa chiều cao 574 của màn 1366×768 @125% | Ràng buộc cứng nhất thắng; 940 vẫn giữ được khoảng cách nút ≥ 96 dip |
| **M-11** | Nhãn trạng thái `BIẾNMẤT` | Tiếp cận: "Đã không còn từ trước" · Tương tác: "tiến trình khác đã xóa trước" | — | **Tách thành hai kết cục ở tầng lõi**: `KHÔNGCÒN` (đã kiểm được) và `KHÔNGĐỌCĐƯỢCĐƯỜNGDẪN` (chưa kiểm được — **tệp có thể vẫn còn trên máy**) | Mã gộp hai ca: `Exists=false` và hàm dựng `FileInfo` ném lỗi vì đường dẫn dị dạng. Nhãn hiện tại khẳng định NGUYÊN NHÂN mà mã chỉ quan sát được KẾT QUẢ (C11) |

**Luật rút ra, áp cho toàn bộ dự án:** *cấm mọi nhãn khẳng định NGUYÊN NHÂN khi mã chỉ quan sát được KẾT QUẢ.*

---

# 3. RÀNG BUỘC BẤT DI DỊCH

Gộp toàn bộ ràng buộc cứng của 5 ghế + biện pháp bịt của 36 đường tấn công, đã khử trùng lặp. Số hiệu này là số hiệu chính thức, phiên sau trích dẫn theo mã.

## A. Nền tảng và phát hành

| Mã | Ràng buộc |
|:---|:---|
| **RB-01** | Dùng **egui/eframe**. Không Tauri (cần WebView2 runtime bên ngoài). Không iced (không có accesskit → trình đọc màn hình không dùng được). |
| **RB-02** | Nạp font hệ thống theo chuỗi `segoeui → arial → tahoma` lúc khởi động. Không tìm được thì **báo lỗi rõ ràng**, không im lặng vẽ ô vuông. |
| **RB-03** | Exe nhúng manifest **PerMonitorV2**. |
| **RB-04** | Cửa sổ tối thiểu **940 × 560 dip**, mặc định 1120 × 760. Mọi màn hình, kể cả hộp thoại xóa, phải hiện đủ ở 1366×768 @125% mà không cuộn ngang và không mất nút hành động nào. |
| **RB-05** | **Không có đường nào trong giao diện dẫn tới việc công cụ tự chạy**: không Scheduled Task, không hook, không tiến trình nền, không khởi động cùng Windows, không khay hệ thống, không kiểm tra cập nhật ngầm, không thông báo chủ động, không "tiếp tục lần trước" khi mở lại. Đóng cửa sổ là hết tiến trình. Câu "công cụ chỉ chạy khi bạn mở nó" hiện cố định ở thanh dưới cùng. |
| **RB-06** | **Một cửa sổ duy nhất.** Bản thứ hai bị chặn khởi động, hiện thông báo và đưa cửa sổ cũ lên trước. |
| **RB-07** | Khóa một-cửa-sổ là **khóa chung cho cả `.exe` lẫn `ZaloCleanup.ps1`** (mutex đặt tên + tệp khóa mang PID). Nếu bản `.ps1` chưa được sửa để lấy khóa đó thì **không ship nút "Mở bản dòng lệnh"**. |
| **RB-08** | Nút "Mở bản dòng lệnh" (lớp trợ năng) là một cuộc **BÀN GIAO**, không phải sinh sản: hiện hộp thoại nêu kết quả quét sắp mất → nhả khóa → khởi chạy console → **tự thoát**. Cùng luật cho nút nâng quyền: nhả khóa trước khi gọi `ShellExecuteExW runas`; người dùng bấm Hủy ở UAC thì tiến trình cũ lấy lại khóa và ở lại. |
| **RB-09** | Máy dựng bản phát hành và CI phải build ở **đường dẫn ngắn** (build thật đã hỏng `LNK1104` ở đường dẫn 178 ký tự). |

## B. Năm nguyên tắc bất biến — ép bằng kiểu dữ liệu, không bằng kỷ luật

| Mã | Ràng buộc |
|:---|:---|
| **RB-10** | Nút Xóa **chỉ tồn tại trong nhánh trạng thái `ĐãQuét`** của máy trạng thái. Chưa quét thì trên màn hình **không có nút xóa nào để bấm** — không phải mờ đi. Một nút mờ vẫn dạy người dùng chỗ để bấm. *(Nguyên tắc 1)* |
| **RB-11** | **Khóa của lượt quét là hash của TOÀN BỘ đầu vào đã sinh ra danh sách**: bộ lọc + gốc + tài khoản + **tập mục catalog đã tick** + ngưỡng `ageHours` từng mục + phiên bản `catalog.json`. So mỗi khung hình. Lệch → kết quả quét bị hủy ngay. *(Nguyên tắc 2)* |
| **RB-12** | Mọi widget nằm **cùng màn hình** với nút Xóa mà có thể thay đổi phạm vi đều phải thuộc khóa lượt quét. Không thuộc được thì **bị vô hiệu hóa** khi đang giữ kết quả quét, kèm dòng "Đang giữ một kết quả quét — bấm Bỏ kết quả này để chọn lại mục". |
| **RB-13** | Bộ lọc sửa trên **bản nháp**; chỉ nút **Áp dụng** mới ghi vào bộ lọc thật và mới hủy kết quả quét. Không hủy theo từng ký tự gõ vào. |
| **RB-14** | Giá trị không hợp lệ thì **KHÔNG áp dụng gì cả** — không áp phần đúng bỏ phần sai, không tự đoán, không tự mở rộng phạm vi. Ô sai viền đỏ, giữ nguyên giá trị cũ, nêu ví dụ hợp lệ. *(Nguyên tắc 3)* |
| **RB-15** | Kiểm **vùng bảo vệ chạy lại cho TỪNG TỆP** ngay trước lời gọi xóa, trong luồng nền. Cấm tối ưu thành kiểm một lần lúc quét. *(Nguyên tắc 4)* |
| **RB-16** | Giao diện **không có bất kỳ công tắc, chế độ chuyên gia, hay đường vòng nào** bỏ qua vùng bảo vệ. |
| **RB-17** | Sao lưu chưa sạch thì nút Xóa **bị khóa ngay tại chỗ**, không phải chặn sau khi bấm. *(Nguyên tắc 5)* |
| **RB-18** | **Mã định danh lượt quét là số duy nhất tăng đơn điệu**, không phải chuỗi thời gian tới giây như `ScanStamp` hiện tại. Mỗi mã chỉ được **tiêu thụ đúng một lần ở tầng lõi**; giao diện lọt hai lần thì lần hai bị lõi từ chối. |
| **RB-19** | Kèm theo mã lượt quét là **dấu vân của thư mục gốc** (số tệp + tổng byte đo nhanh). Lệch quá ngưỡng lúc bắt đầu xóa → lõi từ chối, không phụ thuộc vỏ. |

## C. Vòng đời kết quả quét

| Mã | Ràng buộc |
|:---|:---|
| **RB-20** | **Băng kết quả quét (BQ) hiện trên MỌI màn hình**, chiều cao cố định, không bao giờ thu thành biểu tượng hay giấu sau menu. Trên KP · DL · NC-3/4/5 thì thu còn một dòng chữ, **không mang nút Xóa, không mang nút Sao lưu**. |
| **RB-21** | **Tám nguyên nhân hủy kết quả quét**, mỗi cái phải **cảnh báo TRƯỚC khi xảy ra** kèm số tệp và dung lượng sắp mất: (1) đổi bộ lọc · (2) nạp hồ sơ · (3) đổi tài khoản Zalo · (4) quét nguồn khác · (5) xóa xong · (6) mở lại với quyền quản trị · (7) người dùng chủ động bỏ · **(8) bất kỳ thao tác nào GHI TỆP vào trong ScanRoot — khôi phục là ca duy nhất hiện có**. |
| **RB-22** | Kết quả quét bị hủy hiện thành **trạng thái ĐÃ HỦY có nêu lý do cụ thể** và số cũ gạch ngang, tồn tại tới khi người dùng quét lại hoặc tự đóng. **Không bao giờ biến mất im lặng.** |
| **RB-23** | **Trạng thái thứ sáu: ĐÃ DÙNG DỞ.** Ngay khi luồng xóa dừng giữa chừng, lõi đánh dấu từng mục theo kết cục thật. BQ hiện `8.180 tệp đã mất vĩnh viễn · 4.237 tệp chưa đụng`, số cũ gạch ngang, kèm nút `[Xem 8.180 tệp đã mất]` và `[Mở nhật ký]`. Nhãn nút chỉ đếm phần chưa đụng. |
| **RB-24** | Tuổi kết quả quét: > 30 phút → nhãn hổ phách. > 2 giờ → nút Xóa xám, bắt quét lại. **Chế độ bản trùng lặp: ngưỡng 15 phút**, và hủy ngay khi phát hiện tiến trình `Zalo*` khởi động lại trong lúc đang giữ kết quả. Lượt quét đã bị tiêu thụ một phần: **ngưỡng 0**. |
| **RB-25** | Cấm hai con số tổng cùng loại xuất hiện trên một màn hình mà không nói rõ quan hệ. Dòng chân màn hình ghi `Đang chọn cho LƯỢT QUÉT SAU`, BQ ghi `SẼ XÓA (lượt quét 14:32)`; lệch nhau thì dòng chân in đỏ kèm "khác với lượt quét đang giữ". |

## D. Cổng xác nhận và ma sát

| Mã | Ràng buộc |
|:---|:---|
| **RB-26** | **BA CHỐT** (QĐ-10). Nút xóa mờ khi thiếu chốt và **phải nói rõ đang thiếu chốt nào**, kèm **một nút hành động sửa được ngay**. Xám mà không chỉ đường là ma sát rỗng. |
| **RB-27** | Cổng "đã xem" gắn vào **ngưỡng tuyệt đối > 1.000 tệp hoặc > 1 GB**, không gắn vào màu rủi ro. Cờ đã-xem gắn với mã lượt quét; quét lại là cờ về 0. |
| **RB-28** | Định nghĩa đo được của "đã xem": xem M-01. Tab **Đáng chú ý** là cổng chính khi tab đó khác 0. |
| **RB-29** | **Nhãn mọi nút hành động = động từ + tân ngữ + số lượng + dung lượng**, lấy từ tập sắp xóa tại thời điểm vẽ. Cấm nhãn mơ hồ. Đặc biệt: nhãn `X Xóa kết quả quét đang giữ` (ps1:2765) **phải bị loại bỏ** — nó gọi `Invoke-Delete` và xóa tệp thật. Thao tác hủy kết quả quét mang tên `Bỏ kết quả quét này`. |
| **RB-30** | Đúng **MỘT cửa xác nhận** cho mỗi hành động. Cấm chuỗi hộp thoại giống nhau nối tiếp. |
| **RB-31** | **Không có nút mang chữ "Xóa luôn, không sao lưu".** Việc bỏ đường lui là một **ô tích mang số liệu**: `Tôi xóa 52.748 tệp · 21,4 GB mà KHÔNG có bản sao lưu nào.` Ô tích không bấm nhầm theo vị trí được và không nằm trong quán tính "bấm tiếp". |
| **RB-32** | Trang xác nhận chỉ có **đúng hai nút**: phá hủy góc dưới-trái, an toàn góc dưới-phải. Không có nút thứ ba ở giữa. |
| **RB-33** | Đúng **bốn cụm từ gõ tay** (QĐ-11). Ô gõ **cấm dán từ clipboard**, cấm tự điền, **cấm bôi đen sao chép cụm từ in trên nhãn**, và ô luôn rỗng khi vào lại. Không nhớ cụm đã gõ. |
| **RB-34** | Câu xác nhận nhận **cả ba kiểu viết** (`XÓA` / `XOÁ` / `XOA`), **chuẩn hóa NFC trước khi so**, phân biệt hoa thường. Bản hiện tại (`Test-ConfirmPhrase`, ps1:189) chỉ nhận hai kiểu → người gõ Unikey "đặt dấu kiểu mới" **không xóa được và sẽ tưởng công cụ hỏng**. |
| **RB-35** | Cụm từ xác nhận là **hằng số của lõi an toàn**, KHÔNG nằm trong bảng chuỗi dịch được. Có test đột biến canh. |
| **RB-36** | **Không có ô "Đừng hỏi tôi nữa"**, không có cách nào tắt vĩnh viễn một cửa xác nhận. Người dùng chỉ đổi được **chính sách sao lưu**. |
| **RB-37** | Đếm ngược bị bỏ hẳn (QĐ-13). Không hành động phá hủy nào được khởi động bởi thời gian trôi qua. |
| **RB-38** | **Không thao tác hàng loạt nào làm TĂNG phạm vi xóa.** Có `Giữ lại` từng tệp và `Giữ lại tất cả đang lọc`; **KHÔNG có** `Chọn tất cả để xóa`. |
| **RB-39** | Nút chọn theo nhóm **không mang bất kỳ lời trấn an nào** — nó chỉ liệt kê. Nhãn là con số sống: `Chọn 12 mục · 12,4 GB — trong đó 4 mục phải tải lại 3,2 GB từ mạng và 2 mục chưa kiểm chứng`. Sau khi bấm, băng kết quả nêu tên **cả hai chiều**: đã thêm gì, đã bỏ qua gì. |
| **RB-40** | Nút chọn nhóm và `Ctrl+A` **tuyệt đối không chạm mục có trường `warning`** (giữ luật lệnh `*`, ps1:1301). Mục đó chỉ được chọn bằng cách bấm đúng hộp kiểm của nó. |
| **RB-41** | Ô lọc/tìm kiếm trong Xem trước **chỉ đổi CÁI ĐANG HIỆN**, không bao giờ đổi tập sắp xóa. Có dòng chữ nói thẳng điều đó ngay dưới ô. Đổi phạm vi phải qua một nút riêng, và nút đó **CHỈ được thu hẹp**. |
| **RB-42** | **Không có nút "Ẩn ảnh xem trước".** Cần che khi chia sẻ màn hình thì làm mờ ảnh, hiện rõ khi rê chuột từng tấm — che được người ngoài mà không tắt được ma sát. |
| **RB-43** | Dưới lưới ảnh xem trước, một dòng không nhỏ hơn chữ thường và không xám: `12 ảnh lấy ngẫu nhiên trong 12.418 tệp (0,1%). Chúng không nói được gì về 12.406 tệp còn lại.` |
| **RB-44** | Ở cửa xác nhận R3–R4: ba tệp lớn nhất + tệp mới nhất kèm **ảnh thu nhỏ thật**, không chỉ tên tệp, không có nút tắt. |

## E. Bàn phím, hình học, chống nhấp nhầm

| Mã | Ràng buộc |
|:---|:---|
| **RB-45** | **Không có đường nào từ phím Enter tới một hành động phá hủy.** Mọi màn hình có hành động phá hủy đều không có nút mặc định (`AcceptButton` rỗng); Enter trong ô gõ cụm từ bị nuốt. |
| **RB-46** | **Esc luôn là Hủy** trước khi xóa, ở mọi màn hình, kể cả khi ô nhập có nội dung. **Esc trong lúc đang xóa = Dừng ngay, không hỏi lại.** Esc không bao giờ mang nghĩa tiếp tục. Alt+F4 và nút ✕ đi vào đúng đường dừng an toàn đó. |
| **RB-47** | Tiêu điểm mặc định của mọi hộp thoại phá hủy nằm ở **ô nhập** (R3–R4) hoặc **nút Hủy** (R1–R2), không bao giờ ở nút phá hủy. Thứ tự Tab: ô nhập → Hủy → Xóa; nút xóa là widget dựng **cuối cùng**. |
| **RB-48** | Nút Hủy ghi rõ hậu quả: `Hủy — không đụng gì`. |
| **RB-49** | **Không phím tắt nào dẫn tới xóa**: không `Ctrl+D`, không `Delete`, không `Backspace`, không `Alt+X`, không accelerator, không phím tắt toàn cục. `Delete` trên bảng danh sách không làm gì. |
| **RB-50** | **Khóa mồi đổi từ "tính từ lúc vẽ trang" sang "tính từ MỖI LẦN nút chuyển sang trạng thái bật"**, áp cho **cả chuột lẫn bàn phím**, 600 ms. Kèm điều kiện cứng: nút chỉ nhận sự kiện nếu đã thấy **một lần nhả** (mouse-up hoặc key-up) SAU thời điểm nó được bật. Phím đang giữ từ trước không tính. Bỏ mọi sự kiện phím tự lặp. |
| **RB-51** | Con trỏ đang nằm trên nút phá hủy lúc nút bật → nút giữ vô hiệu tới khi con trỏ **rời khỏi nút ít nhất một lần**. |
| **RB-52** | Trong vùng danh sách cuộn, **`Space` và `PageDown` bị nuốt**: chúng chỉ được cuộn, không bao giờ chạm tới widget đang có tiêu điểm. Khi vùng danh sách đang nhận cuộn thì tiêu điểm bàn phím nằm **trên vùng danh sách**, không trên nút nào. |
| **RB-53** | Nút phá hủy **không nằm trong vòng Tab** cho tới khi đủ điều kiện, và khi vừa đủ điều kiện thì **không tự nhận tiêu điểm**. |
| **RB-54** | Hình học: nút an toàn **góc dưới-phải**, nút phá hủy **góc dưới-trái**, khoảng cách mép–mép **≥ 96 dip**, tuyệt đối không dưới 48 dip ở bề rộng nhỏ nhất. Nút phá hủy **không trùng tọa độ** với nút chính của màn hình liền trước (kiểm tự động). |
| **RB-55** | Nút phá hủy dùng kiểu **viền đỏ / chữ đỏ / nền trong suốt**. **Nút nổi bật nhất màn hình luôn là nút an toàn.** |
| **RB-56** | **Không có nút xóa trên từng dòng bảng.** Biểu tượng thùng rác cạnh thanh cuộn là nam châm hút nhấp nhầm. Double-click trên dòng bảng nhiều nhất là mở khung xem trước. |
| **RB-57** | Trang xác nhận R4 là **trang riêng**, không phải hộp thoại nổi. Hộp thoại (khi có) canh giữa **cửa sổ cha**, trên **cùng màn hình** với cửa sổ cha, chặn thao tác vào cửa sổ cha, giam tiêu điểm. |
| **RB-58** | **Không kéo-thả** tệp/thư mục vào cửa sổ để đặt gốc quét hoặc để xóa. |

## F. Đường lui: sao lưu, khôi phục, bản chụp

| Mã | Ràng buộc |
|:---|:---|
| **RB-59** | Trạng thái sao lưu là **kết quả đo lại**, không phải biến nhớ (QĐ-21). Phép đo gồm: mở lại `_zalocleanup_backup.json`, đối chiếu `Created + Count + Bytes`, **stat ngẫu nhiên 20 tệp** (tồn tại + đúng kích thước). |
| **RB-60** | `LastBackup` và manifest ghi thêm **số hiệu ổ đĩa (volume serial/GUID)**. Cùng chữ cái ổ mà khác số hiệu = **KHÔNG phải bản sao lưu đó**. |
| **RB-61** | Huy hiệu ⛨ có **bảy bộ mặt**: `CHƯA SAO LƯU` · `ĐANG SAO LƯU n%` · `ĐÃ SAO LƯU + độ phủ` · `SAO LƯU LỖI — XÓA BỊ KHÓA` · `SAO LƯU CỦA LẦN QUÉT KHÁC` · **`KHÔNG CÒN THẤY BẢN SAO LƯU — XÓA BỊ KHÓA`** (mới, bộ mặt duy nhất được phép tự xuất hiện) · **`CÙNG Ổ VỚI NGUỒN`** (mới). |
| **RB-62** | **Cấm chữ "SẠCH" đứng một mình.** Hai mức xác minh mô tả bằng **độ phủ**, không bằng thời gian: `(•) Kích thước toàn bộ + nội dung 50 tệp ngẫu nhiên (0,4%)` / `( ) Nội dung toàn bộ 12.418 tệp (100%)`. Dòng độ phủ lặp nguyên văn ngay trên ô gõ `XÓA`. Đích trên ổ tháo rời hoặc ổ mạng → **mặc định đảo sang 100%**; hạ xuống mức mẫu thì huy hiệu giữ màu hổ phách, không bao giờ xanh. |
| **RB-63** | Đích sao lưu bị **chặn cứng** theo QĐ-22. Lý do ghi thẳng trên nút xám: *"Đích nằm trong chính thư mục sắp xóa. Lần quét sau công cụ sẽ tìm thấy bản sao lưu này và xóa nó."* Danh sách cấm được dựng **lại mỗi lần mở màn hình Sao lưu** (vì `catalog.json` người dùng sửa được). |
| **RB-64** | Đích **cùng ổ với nguồn**: cho phép nhưng bắt buộc hiện *"Sao lưu sang C: thì lần xóa này sẽ KHÔNG làm ổ C rộng thêm — 9,72 GB chỉ đổi chỗ trong cùng một ổ."* Bảng ổ đĩa có **ba trạng thái**: đủ chỗ & khác ổ (khuyến nghị) · đủ chỗ nhưng cùng ổ · không đủ chỗ. |
| **RB-65** | Mọi thư mục chứa `_zalocleanup_backup.json` và cây con → **vùng bảo vệ tự động** (QĐ-23). Chiều ngược lại bịt độc lập: **mọi lượt quét BỎ QUA** cây đó và đếm riêng *"đã bỏ qua 12.418 tệp thuộc một bản sao lưu do chính công cụ tạo"* hiện trên BQ. |
| **RB-66** | `BackupRoots` chỉ ghi nhớ đích **đã qua kiểm**. Đích đã bị từ chối một lần không bao giờ xuất hiện trong danh sách "Đã dùng trước đây". |
| **RB-67** | Manifest ghi **NGAY khi bắt đầu** với `TrangThai = "đang chạy"` + tổng dự kiến; cập nhật thành `"xong"` ở cuối. `Find-Backups` liệt kê cả bản dở dang, dán nhãn vàng. **Không bao giờ in "chưa có bản sao lưu nào"** khi trong `BackupRoots` còn thư mục có dữ liệu. |
| **RB-68** | Thẻ bản sao lưu có **hai cột và phải đếm lại tại thời điểm mở màn hình**: `Bản kê ghi 4.102 tệp` / `Đếm trên đĩa lúc 14:41: 4.102 tệp ✔`. Lệch một tệp → thẻ đỏ, không dùng làm đường lui. Câu `Xác minh lúc tạo` luôn kèm ngày và luôn giữ ba chữ "lúc tạo". |
| **RB-69** | Dòng ở Trang chủ nói rõ bản sao lưu **phủ cái gì**: `3 bản · bản mới nhất phủ 7 tệp · 7,2 MB`. |
| **RB-70** | **Phát hiện hết chỗ NGAY** ở lần chép hỏng đầu tiên (`ERROR_DISK_FULL`) và dừng vòng lặp. Đo lại dung lượng trống **ngay trước byte đầu tiên**, đòi biên tối thiểu `max(2 GB, 10% tổng)`. |
| **RB-71** | Sao lưu không trọn vẹn → **hành động mặc định và nổi bật nhất là "Chỉ xóa n tệp đã sao lưu sạch"** ở mức X4 thường (gõ `XÓA`). Cụm `TÔI CHẤP NHẬN MẤT` chỉ dành cho người cố ý xóa cả phần chưa sao lưu, nút đó nhỏ, xa, góc dưới-trái. |
| **RB-72** | `[Thử lại]` sao lưu **chép tiếp vào đúng thư mục dấu thời gian cũ**, chỉ tệp còn thiếu, cập nhật manifest. Không bao giờ tạo thư mục thứ hai. |
| **RB-73** | **Nhật ký lỗi sao lưu ghi ĐỦ mọi tệp — bỏ trần 200 dòng** (ps1:1593). Đây là bằng chứng duy nhất về thứ sắp mất. Cửa xác nhận khi sao lưu bẩn phải **liệt kê tên tệp lỗi**, không chỉ đếm. |
| **RB-74** | **Khôi phục chép ra tên tạm `.zctmp` rồi mới đổi tên.** Ngắt giữa chừng chỉ để lại `.zctmp`; `.zctmp` không bao giờ được tính là "tệp đã tồn tại", luôn bị xóa và chép lại. |
| **RB-75** | "Bỏ qua, giữ tệp hiện có" phải **so KÍCH THƯỚC** với bản trong sao lưu, không chỉ so sự tồn tại. Lệch kích thước = một dòng đỏ riêng *"n tệp ở đích lệch kích thước — có thể là tệp chép dở của lần trước"* + nút `[Chép đè đúng n tệp này]` (ca duy nhất được đè mà không cần gõ `GHI ĐÈ`). |
| **RB-76** | **Khôi phục có bước xác minh ngang hàng với sao lưu** (kích thước toàn bộ + SHA-256 mẫu 50). Không có dòng kết quả xác minh thì **không được in chữ "Đã khôi phục xong"**. Bỏ câu *"chạy lại là an toàn"* cho tới khi RB-74 + RB-75 có thật. |
| **RB-77** | **Cờ ghi-đè và câu `GHI ĐÈ` đã gõ bị HỦY ngay khi thư mục đích đổi.** Đích mới → về mặc định "Giữ tệp hiện có", ô gõ rỗng. |
| **RB-78** | Màn hình đo chỗ khi khôi phục **tách ba con số, không bao giờ gộp**: `Ghi mới: n` · `Ghi ĐÈ LÊN tệp đang có: m` (in đỏ) · `Bỏ qua: k`. Nút `[Xem m tệp sẽ bị đè]` là **cổng bắt buộc** trước khi ô gõ `GHI ĐÈ` nhận phím. |
| **RB-79** | Đích khôi phục qua `Test-ProtectedRoot`. **Bản sao lưu có `SourceRoot` là gốc ổ đĩa (`C:\`, tức bản sao lưu CACHE HỆ THỐNG) KHÔNG được khôi phục hàng loạt** — chỉ mở thư mục cho người dùng tự chép tay. Khôi phục ra ngoài `SourceRoot` gốc **ép chế độ "Giữ tệp hiện có"**, không cho chọn ghi đè. |
| **RB-80** | **Bản chụp System Restore chỉ nhả SAU khi dọn xong** (QĐ-25). Nút ở Trang chủ và ở cửa xác nhận mang nhãn `[Vì sao ổ đĩa không rộng thêm?]` và **chỉ dẫn tới trang GIẢI THÍCH**. Ba lựa chọn phá hủy mở khóa từ trang Kết quả. |
| **RB-81** | Hộp gõ `XÓA HẾT BẢN CHỤP` có một dòng không bỏ qua được: *"Sau bước này, Previous Versions của mọi thư mục trên ổ C sẽ trống. Ảnh Zalo đã xóa mà chưa sao lưu sẽ không còn cách nào lấy lại."* Kèm **danh sách ngày từng bản chụp sắp mất**. |
| **RB-82** | Hạ trần shadow storage: **X3**; nếu trần mới **nhỏ hơn** dung lượng đang dùng thì tự nâng lên **X4**. Xóa bản chụp cũ nhất: **X3 + hiện NGÀY** của bản sắp mất. |

## G. Nói thật về con số

| Mã | Ràng buộc |
|:---|:---|
| **RB-83** | Trang kết quả luôn hiện **ba con số**: đã xóa khỏi thư mục · ổ đĩa trước · ổ đĩa sau, kèm dòng chênh lệch và thanh tỉ lệ `% về tới ổ đĩa`. **Cấm chỉ khoe tổng byte đã xóa.** |
| **RB-84** | Con số **lớn nhất màn hình là dung lượng ổ đĩa thay đổi**, kể cả khi nó là `+0,04 GB`. |
| **RB-85** | Nhãn là **"Ổ C thay đổi trong lúc dọn"**, không phải "rộng thêm". Khi `thu được > đã xóa` hoặc `thu được < 0` → **thay con số to bằng một câu**: *"Không quy được cho lượt dọn này — ổ C đổi −1,20 GB trong lúc chạy, lệch quá xa phần công cụ đụng tới. Ứng dụng khác đang ghi vào ổ."* + nút `[Đo lại ngay bây giờ]`. Mọi con số dung lượng kèm giờ đo và nút đo lại. |
| **RB-86** | Thẻ giải thích Shadow Copy bật theo **tỉ lệ** (M-09). Chênh lệch quá nhỏ để kết luận → thẻ **đổi giọng**, không biến mất. Quy tắc chung: *con số đủ lớn để được đặt ở chỗ to nhất màn hình thì luôn đủ lớn để bắt buộc có một dòng giải thích đi kèm.* |
| **RB-87** | Cảnh báo Shadow Copy ở **ba điểm chạm**: Trang chủ (sớm) · Cửa xác nhận (đúng lúc quyết định) · Trang kết quả (số đo thật). Chỉ báo có **ba trạng thái CÓ / KHÔNG / CHƯA BIẾT — không bao giờ đoán**. |
| **RB-88** | Chỉ chuyển sang **KHÔNG** khi **đủ hai điều kiện**: đang chạy với quyền quản trị **và** `vssadmin` trả về đầu ra đọc được có cấu trúc. Mọi ca còn lại (không quyền · lỗi · rỗng · không phân tích được) đều là **CHƯA BIẾT**. **Cấm tuyệt đối câu "Dung lượng xóa được sẽ về ổ đĩa" ở mọi trạng thái khác KHÔNG.** |
| **RB-89** | Đầu ra `vssadmin` **in nguyên văn**, không lọc theo từ khóa tiếng Anh (nó bị bản địa hóa theo ngôn ngữ Windows). |
| **RB-90** | Trang kết quả sau khi dọn **bản trùng lặp** kết bằng câu điều kiện, không bằng con số dung lượng: *"Nội dung tệp vẫn còn nguyên ở `picture\` và `video\`. Dọn theo mốc thời gian ở thẻ đỏ sẽ đụng đúng những bản gốc đó."* |
| **RB-91** | Đo ba con số **theo PHIÊN** (ổ đĩa lúc mở công cụ · lúc này · chênh lệch), không chỉ theo lượt xóa. Chênh lệch phiên < 50% số byte đã xóa mà máy **không** có bản chụp → hiện thẻ *"9,72 GB đã đi đâu?"* chỉ thẳng vào thư mục sao lưu. |

## H. Hủy giữa chừng và bằng chứng

| Mã | Ràng buộc |
|:---|:---|
| **RB-92** | **Hủy không bao giờ cần xác nhận.** Một cú bấm hoặc một phím Esc là dừng. Cờ `Arc<AtomicBool>` kiểm ở **đầu mỗi vòng lặp từng tệp, trước lời gọi xóa** — không bao giờ giữa chừng một tệp. |
| **RB-93** | Báo cáo sau khi hủy bắt buộc có **ba con số**: đã làm · **CHƯA ĐỤNG TỚI** · tổng. **Cấm hiện chữ "Đã hủy" khơi khơi** — xóa không phải giao dịch quay lui được. Bốn thao tác có bốn câu khác nhau (quét / sao lưu / xóa / khôi phục). |
| **RB-94** | Sao lưu và khôi phục: tệp đích **đang chép dở bị xóa** và không đếm là đã chép. Xóa tệp dở đó cũng lỗi thì **báo tên nó ra màn hình**. |
| **RB-95** | **Ghi trước khi xóa** (QĐ-26): `logs\sapxoa_<mã lượt>.csv` đổ đầy đủ và ép xuống đĩa trước khi chạm tệp đầu tiên. |
| **RB-96** | Nhật ký xóa mở với `WriteThrough`, **xả mỗi 250 ms hoặc mỗi 100 tệp, lấy cái đến trước**. Dòng cuối luôn là `# Đã hủy giữa chừng…` rồi `# Tổng kết…`. |
| **RB-97** | Lịch sử và thẻ "lần trước dừng dở" **đếm từ các dòng thân nhật ký**, không phụ thuộc dòng tổng kết. Không có dòng tổng kết = trạng thái **"kết thúc bất thường"**, hiện đúng chữ đó + đường dẫn `sapxoa_*.csv`, **không hiện số 0**. |
| **RB-98** | Trang kết quả và thẻ ở Trang chủ luôn có nút `[Mở bản kê những tệp đã xóa]` — bằng chứng cách người dùng **đúng một cú bấm**, không nằm sau menu Nâng cao. |
| **RB-99** | Đóng cửa sổ khi đang chạy: chặn đóng, hỏi lại, bật cờ hủy, `join` có hạn giờ. **Cấm `detach`.** Không luồng nào sống lâu hơn cửa sổ. |
| **RB-100** | `vssadmin` là việc **duy nhất không hủy được**. Màn hình nói trước điều đó, không để nút `Dừng lại` giả vờ. |

## I. Hành vi phá hủy phải được công bố trước

| Mã | Ràng buộc |
|:---|:---|
| **RB-101** | **Cắt cụt tệp về 0 byte: mặc định TẮT** (M-07). Không bao giờ chạy im lặng trong vòng lặp: lần xóa thất bại đầu tiên do khóa thì **thu thập danh sách** rồi hiện **M6** sau khi lô chạy xong, có **tên tệp + tên tiến trình đang giữ**, mặc định **không chọn cái nào**. |
| **RB-102** | Chỉ hai chế độ cache được phép cắt cụt. **Dữ liệu Zalo thật và bản trùng lặp không bao giờ bị cắt cụt** (X5, chặn ở lõi, có test canh). **`%TEMP%` và `%WINDIR%\Temp` cũng bị loại khỏi phạm vi cắt cụt.** |
| **RB-103** | Chữ trong M6 chép **cả hai vế** của chú thích mã nguồn: *"Cắt cụt một tệp mà ứng dụng khác đang mở CÓ THỂ làm ứng dụng đó hỏng theo kiểu rất khó chẩn đoán. Bỏ qua tệp bị khóa thì vô hại — chỉ là không thu được dung lượng của nó."* |
| **RB-104** | `Cứ dọn hết` ở M4 **không bao giờ kéo theo quyền cắt cụt**; công tắc cắt cụt bị vô hiệu cho đúng các mục có ứng dụng đang chạy. |
| **RB-105** | **M4 không được dựa vào trường `procs` của `catalog.json`.** Nó phải hỏi hệ điều hành tệp nào trong phạm vi đang bị handle nào giữ (RmGetList) và nêu tên tiến trình thật. `procs` rỗng không có nghĩa là không ai đang dùng. |
| **RB-106** | Danh sách tiến trình đang chạy **đọc lại đúng lúc bấm QUÉT và lần nữa lúc bấm XÓA**, không dùng ảnh chụp lúc mở màn hình. |
| **RB-107** | **Dọn thư mục rỗng** công bố thành một dòng trong hộp xác nhận + công tắc trong Cài đặt. Ở mức X1 (không có hộp thoại) thì **tắt cứng**. Giữ nguyên: không đệ quy, không xuyên junction. |
| **RB-108** | Nút đóng Zalo mang nhãn bằng đúng việc nó làm: `Yêu cầu Zalo đóng, sau 7 giây thì BUỘC DỪNG tiến trình`, cùng cỡ chữ: *"Buộc dừng có thể làm mất tin nhắn đang soạn và tệp đang gửi dở."* **Lựa chọn an toàn `Tôi tự đóng Zalo, kiểm lại` là mặc định và là nút nổi bật nhất.** |
| **RB-109** | **Bỏ bước buộc dừng khỏi luồng tự động.** Sau 7 giây, quay lại giao diện với ba lựa chọn tường minh: `Chờ thêm 30 giây` · `Tôi đã đóng, kiểm lại` · `Buộc dừng n tiến trình Zalo`. Nếu vẫn buộc dừng, ghi thời điểm vào nhật ký và hiện trên trang kết quả. |
| **RB-110** | Sửa câu ở màn Vùng bảo vệ cho khớp phạm vi thật: *"Công cụ không bao giờ XÓA tệp trong `Database` và `Partitions`"* — **bỏ cụm "ngoài tầm với"**, vì bước đóng Zalo là một đường công cụ vẫn chạm tới chúng. |
| **RB-111** | Trước mỗi lệnh xóa bản trùng lặp: **stat lại KEEPER**. Không tồn tại hoặc kích thước khác lúc quét → **KHÔNG xóa**, ghi trạng thái `KHÔNGCÒNBẢNGIỮ`, đếm riêng, hiện bằng chữ trên trang kết quả. |
| **RB-112** | Sau mỗi lần dọn bản trùng lặp, ghi danh sách keeper vào `logs\bangiulai_<mã lượt>.txt`. Khi quét dữ liệu Zalo thật, đối chiếu và hiện dòng đỏ trong cửa xác nhận: *"n tệp trong lô này là BẢN DUY NHẤT CÒN LẠI — bản thứ hai đã bị xóa trong lần dọn trùng lặp ngày…"* + nút `[Giữ lại tất cả n tệp này]`. |

## J. Tiếng Việt, vùng miền, tiếp cận

| Mã | Ràng buộc |
|:---|:---|
| **RB-113** | **Chuẩn hóa Unicode CHỈ dùng cho chuỗi đem đi hiển thị.** Đường dẫn đưa vào thao tác tệp và vào phép kiểm vùng bảo vệ là **chuỗi gốc nguyên bản, so sánh ordinal**. Normalize trước khi xóa là lỗi mất dữ liệu. |
| **RB-114** | Phông nhúng/nạp có **test tự động duyệt 134 chữ cái tiền tổ hợp tiếng Việt + toàn bộ bảng chuỗi giao diện**, `has_glyphs` phải true hết. Test chạy trong CI và **chặn merge**. |
| **RB-115** | Định dạng số và ngày **cố định kiểu Việt** (`12,96 GB` · `149.309 tệp` · `dd/MM/yyyy`), không phụ thuộc locale máy. Cùng exe chạy trên en-US và vi-VN cho **chuỗi giống hệt**. |
| **RB-116** | Mọi so sánh chuỗi/đường dẫn dùng **ordinal**. Bộ test phải chạy được dưới `tr-TR` (bẫy chữ I không dấu chấm) mà kết quả không đổi. |
| **RB-117** | Ô nhập số **từ chối** `5,5` và `5.5` kèm giải thích, không bao giờ âm thầm hiểu thành 5 hay 55. Tham số gửi `vssadmin` luôn ở dạng bất biến (`/maxsize=5GB`). |
| **RB-118** | CSV xuất ra có **BOM UTF-8, dòng đầu `sep=;`, trường ngăn bằng `;`**, ngày `yyyy-MM-dd HH:mm:ss`. Nhật ký cũng có BOM UTF-8. |
| **RB-119** | Nhãn trạng thái trong nhật ký **giữ nguyên byte** so với bản PowerShell (`ĐÃXÓA` `CẮTCỤT` `THẤTBẠI` `VÙNGBẢOVỆ`), trừ hai nhãn mới thay cho `BIẾNMẤT` (M-11). |
| **RB-120** | **Mức rủi ro mã hóa bằng ba lớp: chữ + ký hiệu hình + màu.** Màu không bao giờ là tín hiệu duy nhất (vi phạm hiện tại: ps1:1267). |
| **RB-121** | **Không dùng thông báo tự tắt** (toast/snackbar) cho bất cứ điều gì liên quan an toàn hoặc kết quả xóa. **Cấm mọi gợi ý rằng có thể hoàn tác.** |
| **RB-122** | Trạng thái "không đủ quyền" **không được trình bày như lỗi**. Mọi nút nâng quyền cảnh báo trước rằng kết quả quét hiện tại sẽ mất. |
| **RB-123** | Trước khi gọi UAC: cảnh báo bằng tiếng Việt rằng hộp thoại Windows có thể bằng tiếng Anh và sẽ ghi *Unknown publisher* vì phần mềm chưa được ký số. |
| **RB-124** | Phát hiện trình đọc màn hình (`SPI_GETSCREENREADER`) → hiện dải thông báo và mở đường sang bản dòng lệnh (theo RB-07/RB-08). **Bản dòng lệnh được giữ sống và ghi trong tài liệu là đường tiếp cận chính thức cho người khiếm thị.** |
| **RB-125** | Hỗ trợ **Chế độ tương phản cao** của Windows (`SPI_GETHIGHCONTRAST`) — egui không làm sẵn, phải tự viết. Theo theme sáng/tối của hệ thống. Có thanh chỉnh cỡ chữ 80–200%, khởi tạo theo cỡ chữ Trợ năng của Windows. |

## K. Thực thi (egui / luồng)

| Mã | Ràng buộc |
|:---|:---|
| **RB-126** | **Luồng giao diện KHÔNG BAO GIỜ chạm hệ thống tệp.** Quét, xóa, sao lưu, khôi phục, đo dung lượng chạy trên **đúng một luồng nền**. Ngoại lệ duy nhất: băm SHA-256 dùng pool cố định 8 luồng bên trong luồng nền đó. |
| **RB-127** | Luồng nền gọi `ctx.request_repaint()` mỗi lần gửi tiến trình. Thiếu là thanh tiến trình đứng im và người dùng tắt cưỡng bức giữa lúc đang xóa. |
| **RB-128** | Tiết lưu tiến trình: **mỗi 200 tệp hoặc mỗi 100 ms, lấy cái thưa hơn**. Kênh `mpsc`, luồng UI `try_recv()` hút cạn mỗi khung, không bao giờ `recv()` chặn. |
| **RB-129** | Danh sách xem trước **ảo hóa bằng `ScrollArea::show_rows`**, chiều cao dòng cố định. **Không có gì tỉ lệ với N chạy trong luồng giao diện**: định dạng chuỗi làm một lần trong luồng quét; sắp xếp/lọc/dựng cột tìm kiếm chạy ngoài luồng khi N > 20.000. |
| **RB-130** | Mọi tác vụ > 400 ms có tiến trình; > 2 giây có **số đếm thật** và nút `Dừng lại`. **Không spinner vô định cho việc dài.** |
| **RB-131** | Khóa mọi widget bộ lọc khi luồng nền đang chạy. |
| **RB-132** | Modal của egui là modal **tự vẽ**, không phải modal thật của Windows → phải tự chặn phím và chuột, tự giam tiêu điểm, tự vô hiệu hóa nền. |

---

# 4. SƠ ĐỒ ĐIỀU HƯỚNG

```
TC  TRANG CHỦ  ── MÀN HÌNH GỐC DUY NHẤT (mở exe là vào thẳng đây)
│   [dải trạng thái máy]  ổ đĩa · thư mục Zalo · bản chụp System Restore · cảnh báo môi trường
│   [BĂNG KẾT QUẢ QUÉT — BQ]  cố định, đi theo mọi màn hình (thu 1 dòng ở KP/DL/NC)
│   [chân trang cố định]  "Công cụ này chỉ chạy khi bạn mở nó…"
│
├── "Lấy lại dung lượng từ đâu?" — bốn thẻ, rủi ro tăng dần, KHÔNG có nút xóa nào
│   ├── BL-1  🟢 Bản thừa Zalo tự nhân đôi
│   ├── BL-2  🟢 Cache của ứng dụng Zalo
│   ├── BL-3  🟡 Cache hệ thống ngoài Zalo   (bảng 33 mục)
│   └── BL-4  🔴 Ảnh video Zalo cũ theo thời gian
│         │
│         └── ba màn hình con dùng chung cho cả bốn bàn làm việc
│             ├── XT  XEM TRƯỚC — CÁI SẮP MẤT    ← cửa bắt buộc
│             ├── SL  SAO LƯU VÀ XÁC MINH
│             └── KQ  KẾT QUẢ SAU KHI XÓA
│
├── DL  XEM DUNG LƯỢNG ĐANG BỊ CHIẾM   (CHỈ ĐỌC — không lối tắt nào tới xóa)
├── KP  KHÔI PHỤC TỪ BẢN SAO LƯU
│
└── NC  NÂNG CAO   (một nút xám, không phải menu bày sẵn)
    ├── NC-1  Bộ lọc chi tiết + Bộ lọc đã lưu
    ├── NC-2  Bản chụp hệ thống (System Restore)
    ├── NC-3  Vùng bảo vệ  (chỉ đọc + ô thử đường dẫn)
    ├── NC-4  Lịch sử dọn dẹp và nhật ký
    └── NC-5  Cài đặt — chính sách sao lưu · tài khoản Zalo · gốc dữ liệu · thông tin bản dựng

BẢY HỘP THOẠI CHẶN, không hơn:
  M1 Xác nhận xóa           M2 Đóng Zalo             M3 Nâng quyền quản trị
  M4 Ứng dụng đang chạy     M5 Thao tác bản chụp     M6 Cắt cụt tệp bị khóa (mới)
  M7 Sao lưu không trọn vẹn (mới)
```

**Ba tầng bộc lộ dần**

| Tầng | Chứa gì | Vì sao |
|:--|:---|:---|
| **0 · Trang chủ** | Số đo · 4 thẻ nguồn · Xem dung lượng · Khôi phục · Nâng cao · BQ | Toàn tầng **chỉ đọc** trừ BQ. Bấm nhầm ở đây không mất gì |
| **1 · Bàn làm việc** | Phạm vi của đúng nguồn đó · Quét · Xem trước · Sao lưu · Xóa | Chu trình đầy đủ của một mục đích |
| **2 · Nâng cao** | Bộ lọc chi tiết · hồ sơ · bản chụp · vùng bảo vệ · lịch sử · chính sách · gốc dữ liệu | Đều đòi hiểu một mô hình dữ liệu riêng |

**Ánh xạ 111 lệnh của bản dòng lệnh: 0 lệnh bị bỏ.** Hai lệnh gộp (`3 Khôi phục` ở Trang chủ ≡ `K` ở nâng cao; hai lối vào mốc thời gian cùng ghi `$FromDate/$ToDate`). Bốn lệnh đổi hình thức nhưng giữ nguyên hiệu lực: `Xem chi tiết? (c/k)` → cửa bắt buộc; hai câu hỏi thời gian của khử trùng lặp → nút Dừng; `*` → nút có nhãn nói ra lớp phanh.

---

# 5. TỪNG MÀN HÌNH, TỪNG TRẠNG THÁI

## 5.1 Chín trạng thái chuẩn — định nghĩa một lần

| Trạng thái | Ký hiệu | Quy tắc trình bày bắt buộc |
|:---|:-:|:---|
| Rỗng | `∅` | Nói **vì sao rỗng** (bộ lọc nào, mốc nào, đường dẫn nào). Hành động đề xuất **không bao giờ tự mở rộng phạm vi** |
| Đang tải | `⏳` | Tại chỗ, không phủ modal. <400 ms: không hiện gì. 400 ms–2 s: skeleton. >2 s: skeleton + **số đếm thật** + `Dừng lại` |
| Có dữ liệu | `✔` | Con số + đơn vị + **mốc thời gian đo**. Mọi số liệu kèm "đo lúc nào" |
| Lỗi | `✖` | Thẻ 5 phần: ① chuyện gì ② **cái gì đã và chưa xảy ra** — với thao tác phá hủy, câu đầu luôn là `Chưa xóa gì.` ③ chi tiết kỹ thuật gấp lại ④ một nút hành động chính ⑤ `Chép chi tiết lỗi` |
| Không đủ quyền | `🔒` | **Không phải lỗi.** Nêu mục nào cần quyền, cái gì vẫn làm được, và **cảnh báo mất kết quả quét khi nâng quyền** |
| Đang chạy | `▶` | Tiến độ theo tệp · số đếm theo từng loại kết cục · tệp đang xử lý · ETA · `Dừng lại` · câu bảo đảm an toàn khi dừng |
| Hủy giữa chừng | `⏹` | **Ba con số**: đã làm · **chưa đụng tới** · tổng |
| Hoàn tất | `◎` | Không tự chuyển màn hình |
| Hoàn tất một phần | `◐` | Khi có bất kỳ tệp nào rơi vào `THẤTBẠI`/`CẮTCỤT`/`KHÔNGCÒN`/`KHÔNGĐỌCĐƯỢCĐƯỜNGDẪN`/`VÙNGBẢOVỆ`/`KHÔNGCÒNBẢNGIỮ`, hoặc bị hủy, hoặc hết chỗ. **Sáu ô đếm luôn hiện đủ**; ô bằng 0 thì xám |

## 5.2 Băng kết quả quét (BQ) — sáu trạng thái

| # | Trạng thái | Nội dung |
|:-:|:---|:---|
| 1 | **TRỐNG** | `Chưa quét lần nào. Chưa có gì để xóa.` Nút Xóa **không tồn tại** |
| 2 | **ĐANG QUÉT** | Nguồn · số tệp/dung lượng chạy dần · `Bước này chỉ đọc, không xóa gì.` · `[Dừng]` |
| 3 | **ĐANG GIỮ, chưa xem** | Nguồn · số tệp · dung lượng · giờ quét + tuổi · **phạm vi của lượt quét** · `Vùng bảo vệ đã chặn n tệp` · huy hiệu ⛨ · `[Xem n tệp sắp mất]` `[Sao lưu]` `(Xóa — cần xem trước)` `[Bỏ kết quả này]` |
| 4 | **ĐANG GIỮ, đã xem** | Như trên, nút Xóa bật, nhãn mang số sống |
| 5 | **ĐÃ HỦY** | Số cũ **gạch ngang** + **lý do cụ thể** (`bạn đã đổi khoảng thời gian từ … sang …`) + `[Quét lại với bộ lọc mới]` `[Đóng]` |
| 6 | **ĐÃ DÙNG DỞ** *(mới, RB-23)* | `8.180 tệp đã mất vĩnh viễn · 4.237 tệp chưa đụng`, số cũ gạch ngang, `[Xem 8.180 tệp đã mất]` `[Mở nhật ký]`. Nút Xóa chỉ đếm phần chưa đụng, và **kết quả quét bị coi là hết hạn ngay** (RB-24) |

## 5.3 Khởi động

| Trạng thái | Trông ra sao |
|:---|:---|
| `⏳` | Bốn dòng tick dần: đọc cấu hình · dựng vùng bảo vệ · dò thư mục Zalo · đo ổ đĩa. Kèm `Công cụ chưa đụng vào tệp nào. Bước này chỉ đọc.` Quá 3 giây ở một bước thì hiện đường dẫn đang xử lý |
| `✖` catalog hỏng | Vào thẳng Trang chủ + băng vàng, `[Xem lỗi]` liệt kê **từng mục sai kèm lý do**. Không chặn khởi động |
| `∅` chưa cài Zalo | Ba thẻ Zalo **vô hiệu kèm lý do ghi ngay trên thẻ**, không ẩn |
| nhiều tài khoản | Màn hình chọn tài khoản trước Trang chủ — **nhận diện bằng 6 ảnh thu nhỏ mới nhất + ngày tệp mới nhất**, không phải dãy 19 chữ số (RB đối phó A12) |
| `✖` bản thứ hai | Không mở cửa sổ; đưa cửa sổ cũ lên trước |

## 5.4 Trang chủ

Dải trạng thái máy: ổ đĩa trống · thư mục Zalo (`chưa đo · bấm để đo` nếu lần trước đo quá 5 giây — **không bịa số**) · bản chụp System Restore (CÓ / KHÔNG / **CHƯA BIẾT**, mặc định khi chưa có quyền quản trị là CHƯA BIẾT) · cảnh báo môi trường.

| Trạng thái | Trông ra sao |
|:---|:---|
| `⏳` đo thư mục | Skeleton + `đang đo… n tệp` + `[Dừng]`. Bốn thẻ vẫn bấm được |
| `∅` chưa có bản sao lưu | `Sao lưu gần nhất: chưa có bản HOÀN CHỈNH nào.` + nếu có thư mục dở dang thì nêu tên, dung lượng, `[Xem]` `[Tiếp tục sao lưu phần còn thiếu]` (RB-67) |
| `🔒` chưa đọc được VSS | `CHƯA BIẾT — cần quyền quản trị để đọc` + `[Đọc tình trạng]` + cảnh báo mất kết quả quét |
| `✖` mất thư mục Zalo | Băng đỏ + `[Dò lại]` `[Chọn tài khoản khác]`; BQ chuyển ĐÃ HỦY |
| `◐` lần trước dừng dở | Thẻ: `Lần chạy trước kết thúc bất thường lúc … — đã xóa n/m tệp` + `[Mở bản kê những tệp đã xóa]`. **Không có nút "Tiếp tục lần trước"** (M-03) |

**Không có trên Trang chủ:** nút "Dọn ngay", "Dọn nhanh", "Quét toàn bộ", tổng gộp bốn nguồn, thanh "sức khỏe máy", điểm số, huy hiệu, con số nhấp nháy.

## 5.5 Bốn bàn làm việc

| Màn hình | Điểm đặc thù | Trạng thái đáng chú ý |
|:---|:---|:---|
| **BL-1 Bản trùng lặp** | Bốn bước có tên và có tiến trình. Cửa chặn nhẹ trước bước băm (`đọc đĩa, có thể mất vài phút`). Thẻ **không** mang chữ `→ Bắt đầu từ đây` cho tới khi có người kiểm chứng hành vi Zalo sau khi xóa `resource\` (xem §9) | `∅` không có `resource\` · `∅` không còn bản độc lập → **thẻ ĐỎ**, không phải xanh · `✔` hiện dòng vàng `n ứng viên trùng kích thước đã bị loại ở bước SHA-256` (bằng chứng bước 4 không thừa) · `⏹` `Đã dừng ở bước 3. Chưa có kết quả quét, chưa đụng vào tệp nào.` |
| **BL-2 Cache Zalo** | 8 thư mục cố định, mỗi dòng tên · số tệp · dung lượng | `∅` `Không có gì để dọn.` |
| **BL-3 Cache hệ thống** | Bảng 33 mục, hộp kiểm từng dòng, nhóm A/B/C. **Nhãn nút nhóm mang số sống và nêu cả VÀNG lẫn chưa-kiểm-chứng** (RB-39). Nút `Quét` đọc lại danh sách tiến trình đúng lúc bấm | `⏳` skeleton từng dòng, đo song song · `🔒` dòng cần quyền vẫn tick được nhưng bị loại lúc quét kèm băng nêu tên · `✖` catalog hỏng → băng vàng + `[Xem n mục bị bỏ qua và lý do]` · `⏹` giữ nguyên ô đã tích |
| **BL-4 Dữ liệu Zalo cũ** | Mốc đo trước rồi mới hỏi. Mốc 0 byte hiện **mờ kèm `không còn dữ liệu`**, không ẩn. Ô ngày nhận 4 dạng; sai thì giữ nguyên | `⏳` `Đang đo dung lượng theo từng mốc…` + `[Dừng]` · `∅` chỉ còn lựa chọn tự nhập ngày · `⏹` `Đã dừng — chưa đo xong, chưa đụng vào gì.` |

**Khi gốc do người dùng chỉ tay** (NC-5 → "Chỉ định thư mục ZaloData"): BQ mang một **dòng đỏ thường trực** `GỐC DO BẠN CHỈ TAY: <đường dẫn> — công cụ KHÔNG xác minh được đây là dữ liệu Zalo`; cột đường dẫn trong XT hiện **đường dẫn tuyệt đối đầy đủ**; thẻ 🔴 đổi nhãn thành `mọi tệp trong thư mục bạn đã chỉ`; cửa xác nhận in nguyên gốc ở dòng đầu, cỡ chữ bằng cỡ số tệp. *(bịt B3)*

## 5.6 XT — Xem trước, cái sắp mất (màn hình quan trọng nhất)

Bốn tab. Mặc định **Tổng quan**.

| Tab | Nội dung |
|:---|:---|
| **Tổng quan** | Câu mô tả cái sắp mất bằng lời người dùng · phân bố theo thư mục / theo năm / theo đuôi · **12 ảnh thật lấy ngẫu nhiên**, kèm dòng RB-43 · huy hiệu ⛨ |
| **Danh sách (n)** | Bảng ảo hóa. Cột: Ngày · Kích thước · Đường dẫn · **Giữ lại**. Ô lọc chỉ đổi cái đang hiện (RB-41). Chế độ trùng lặp: bảng cặp `xóa / giữ` + `SHA-256 khớp toàn bộ nội dung ✔`. Chế độ cache: thêm cột `Mục` |
| **Lớn nhất** | 30 tệp lớn nhất + ngày |
| **Đáng chú ý (n)** | **Cổng chính của chốt (b)** khi n > 0. Nhóm: tệp sửa trong 7 ngày · tệp > 1 GB · đuôi lạ so với phần còn lại · thư mục chỉ xuất hiện một lần · **tệp có đuôi ảnh/video nằm trong một mục cache** (mới, bịt B2) · **tệp là bản duy nhất còn lại sau lần dọn trùng lặp** (mới, bịt A13) · mục 🟡 phải tải lại từ mạng · mục chưa kiểm chứng. Mỗi nhóm có nút `Giữ lại tất cả` |

| Trạng thái | Trông ra sao |
|:---|:---|
| `⏳` | Header + skeleton; tab ghi `Danh sách (đang xếp… 62%)`. Nút `Tiếp tục` **mờ trong lúc dựng** |
| `✖` mất tệp giữa chừng | Băng: `n tệp trong danh sách đã biến mất khỏi đĩa từ lúc quét. Chúng sẽ được ghi là KHÔNGCÒN chứ không tính vào thành tích.` Không quét lại tự động |
| `🔒` | Dòng không đọc nổi metadata ghi `không đọc được — sẽ bỏ qua`, **không đếm vào tổng** |
| **hết hiệu lực** | Cả màn hình phủ lớp mờ + `Kết quả quét này đã hết hiệu lực. [Quét lại]`. Nút `Tiếp tục` **biến mất**, không chỉ mờ |
| **bắt gặp bản sao lưu** | Băng đỏ đầu bảng + nút `Bỏ n tệp thuộc bản sao lưu ra khỏi lô` (RB-65) |

**Ngân sách khung hình (đo ở 150.000 tệp):** định dạng chuỗi 1 lần trong luồng quét ~30 ms · dựng chỉ mục sắp xếp ~12 ms ngoài luồng · cột tìm kiếm dựng lười ~45 ms · lọc ~8 ms chống dội 150 ms · vẽ bảng **chỉ 34 dòng thấy được, <1 ms**. Bộ nhớ ≈ 40 MB.

**Ảnh xem trước:** giải mã ngoài luồng UI, tối đa 12 ảnh / 4 luồng / 8 MB đầu mỗi tệp, chỉ giải mã sau khi cuộn đứng yên 120 ms, thu về 128 px, LRU 64 ảnh. Nhận dạng bằng **magic bytes**, không bằng đuôi (Zalo lưu `.jxl` và tệp **không đuôi**). Không đọc được → ô `?`, **không bao giờ ẩn tệp đó khỏi danh sách**.

## 5.7 M1 — Trang xác nhận xóa

Một trang, không phải hộp thoại nổi. Nội dung theo thứ tự:

1. `Sắp xóa vĩnh viễn` — số tệp · dung lượng · loại dữ liệu · **gốc tuyệt đối** · `Không qua Thùng rác.`
2. Ba tệp lớn nhất + tệp mới nhất trong lô, **kèm ảnh thu nhỏ thật** (RB-44)
3. Khối ⛨ sao lưu: huy hiệu + **độ phủ xác minh** (RB-62) + `[Sao lưu trước — n GB, khoảng m phút]`
4. Ô tích mang số liệu nếu không có sao lưu (RB-31)
5. Thẻ ⚠ bản chụp (CÓ / CHƯA BIẾT) + `[Vì sao ổ đĩa không rộng thêm?]` — **dẫn tới trang giải thích, không dẫn tới thao tác phá hủy** (RB-80)
6. Dòng riêng nếu lô chứa **bản duy nhất còn lại** sau lần dọn trùng lặp (RB-112)
7. Dòng nhận diện tài khoản (ảnh thu nhỏ) nếu máy có nhiều tài khoản
8. Ô gõ cụm từ (R4) — `gõ tay, không dán được`
9. Đúng hai nút: `[Xóa vĩnh viễn 12.417 tệp · 9,71 GB]` góc dưới-trái · `[Hủy — không đụng gì]` góc dưới-phải

**Máy trạng thái nút Xóa** — chỉ hai trạng thái: bật, hoặc xám kèm lý do đọc được + nút sửa ngay.

| Lý do xám | Chữ hiện cạnh nút | Nút sửa |
|:---|:---|:---|
| Chưa quét | Chưa có kết quả quét. | `[Quét]` |
| Bộ lọc / tập mục đã đổi | Bộ lọc đã đổi — kết quả cũ không dùng được. | `[Quét lại]` |
| Quá 2 giờ (15 phút với trùng lặp) | Kết quả đã quét 2 giờ 14 phút trước. | `[Quét lại]` |
| Chưa mở danh sách | Hãy mở tab Đáng chú ý và xem hết trước khi xóa. | `[Mở danh sách]` |
| Chính sách BẮT BUỘC, chưa có sao lưu sạch | Chính sách hiện tại là bắt buộc sao lưu. | `[Sao lưu ngay]` |
| Sao lưu có lỗi | Sao lưu chưa sạch: 3 tệp chép lỗi, 1 tệp xác minh lỗi. | `[Xem tệp lỗi]` `[Chép tiếp phần thiếu]` |
| **Không còn thấy bản sao lưu** | Không còn đọc được bản sao lưu ở E:\… — cắm lại ổ hoặc sao lưu lại. | `[Đo lại]` `[Sao lưu lại]` |
| Chưa gõ đúng cụm từ | Còn thiếu — gõ đúng `XÓA` (hoặc `XOÁ`, `XOA`). | — |
| Chưa tick ô bắt buộc | Còn 1 ô chưa xác nhận. | — |
| Zalo đang chạy | Zalo đang chạy. | `[Đóng Zalo]` |
| Không đủ quyền | 2 mục cần quyền quản trị. | `[Mở lại với quyền quản trị]` |
| Đang có lượt xóa chạy | Đang xóa… | `[Dừng]` |

## 5.8 Đang xóa · Kết quả

**Đang xóa:** tiến độ theo tệp · sáu ô đếm sống · byte đã xóa (kèm `đây chưa phải dung lượng ổ đĩa`) · ETA · tệp đang làm · nhật ký 100 dòng cuối · câu `Dừng lại là an toàn: công cụ dừng GIỮA hai tệp.` · nút nhãn **`Dừng lại`** (không phải "Hủy" — trong lúc đang xóa, "Hủy" bị hiểu thành "hoàn tác").

**Kết quả — ba biến thể:**

| Biến thể | Bố cục |
|:---|:---|
| `◎` Hoàn tất | Con số to nhất = **ổ đĩa thay đổi** + hai mốc trước/sau · thanh `% về tới ổ đĩa` · thẻ giải thích khi lệch (RB-86) hoặc **thẻ "không quy được trách nhiệm"** (RB-85) hoặc **thẻ "9,72 GB đã đi đâu?"** khi sao lưu cùng ổ (RB-91) · sáu ô đếm · thư mục rỗng đã dọn · thời gian · `[Mở bản kê những tệp đã xóa]` `[Mở nhật ký]` |
| `⏹` Hủy giữa chừng | Ba con số (RB-93). Nút nổi bật nhất là **`[Xem n tệp chưa đụng tới]`**, không phải nút tiếp tục. **Không có nút "Tiếp tục phần còn lại"** |
| `◐` Một phần | Thẻ giải thích cho từng loại khác 0, trong đó `KHÔNGĐỌCĐƯỢCĐƯỜNGDẪN` phải nói thẳng: *"n tệp không mở được đường dẫn — công cụ CHƯA xóa chúng, chúng vẫn còn trên máy."* và `KHÔNGCÒNBẢNGIỮ`: *"Đã giữ lại n tệp vì bản đối chiếu của chúng không còn trên máy — nếu xóa, bạn đã mất cả hai bản."* |

Ba lựa chọn xử lý bản chụp **mở khóa tại đây** (RB-80).

## 5.9 SL — Sao lưu

| Trạng thái | Trông ra sao |
|:---|:---|
| Chọn đích | Bảng ổ đĩa **ba trạng thái** (RB-64). Đích bị cấm → nút xám kèm lý do (RB-63). Đích cùng ổ → dòng cảnh báo dung lượng chỉ đổi chỗ |
| Mức xác minh | Hai radio mô tả bằng **độ phủ** (RB-62). Ổ tháo rời/ổ mạng → mặc định 100% |
| `▶` | Hai pha có tiến trình riêng: chép, rồi xác minh. `Dừng lại` ở cả hai pha |
| `✖` không đủ chỗ | Chặn tại chỗ, **chưa tạo thư mục nào**. Đo lại ngay trước byte đầu tiên, biên `max(2 GB, 10%)` |
| `✖` hết chỗ giữa chừng | **Dừng ngay ở lần chép hỏng đầu tiên** (RB-70) → **M7** |
| `⏹` | Tệp chép dở bị xóa. `Bản này KHÔNG mở khóa bước xóa.` Manifest giữ `TrangThai = "đang chạy"` |
| `◎` | `Đã sao lưu · khớp kích thước n/n · đối chiếu nội dung m/n (x%)` + đường dẫn tuyệt đối |
| `◐` | **M7**: hành động mặc định và nổi bật nhất là `Chỉ xóa 9.100 tệp đã sao lưu sạch · 7,10 GB` (RB-71) |

## 5.10 KP — Khôi phục

Thẻ bản sao lưu **hai cột** (bản kê / đếm trên đĩa lúc này) — RB-68. Mặc định `Giữ tệp hiện có`.

| Trạng thái | Trông ra sao |
|:---|:---|
| `∅` | `Không tìm thấy bản sao lưu HOÀN CHỈNH nào do công cụ này tạo ra.` + liệt kê bản dở dang + ô nhập đường dẫn thủ công |
| đo chỗ | **Ba con số tách riêng** (RB-78). Thiếu chỗ → bốn nút ngay tại chỗ, **và đổi đích thì cờ ghi-đè + câu đã gõ bị hủy** (RB-77) |
| `✖` đích ngoài gốc | Băng vàng + **ép chế độ Giữ tệp hiện có** (RB-79) |
| `◐` hết chỗ giữa chừng | `Đã khôi phục n/m tệp. Bản sao lưu vẫn nguyên vẹn.` + kết quả xác minh (RB-76) |
| xong | **BQ chuyển sang ĐÃ HỦY**, lý do: *"Bạn vừa khôi phục n tệp vào đúng thư mục này. Danh sách quét lúc 14:32 giờ đang trỏ vào cả những tệp vừa cứu về. Phải quét lại."* (RB-21 nguyên nhân 8) |

## 5.11 NC-2 — Bản chụp hệ thống

Phần giải thích + hai bảng số đo **hiện đầy đủ kể cả khi chưa có quyền quản trị**. Đầu ra `vssadmin` in nguyên văn.

| Trạng thái | Trông ra sao |
|:---|:---|
| Đang giữ kết quả quét / chưa dọn lần nào | **Ba lựa chọn phá hủy XÁM** kèm lý do RB-80 |
| Mở khóa (từ trang Kết quả) | Ba lựa chọn hoạt động; xóa toàn bộ đòi `XÓA HẾT BẢN CHỤP` + **liệt kê ngày từng bản** |
| `🔒` | Kiến thức không cần quyền → vẫn hiện; phần tình trạng thay bằng `[Mở lại với quyền quản trị]` + cảnh báo mất kết quả quét |
| `✖` vssadmin lỗi / rỗng | **CHƯA BIẾT**, in nguyên văn, `[Chép đầu ra]`. **Không bao giờ dịch thành "KHÔNG có bản chụp"** (RB-88) |
| `▶` | Nút mờ + `Đang chạy vssadmin…` + `Thao tác này không dừng giữa chừng được.` |

## 5.12 Ba màn hình phụ

| Màn hình | Nội dung | Trạng thái đáng chú ý |
|:---|:---|:---|
| **NC-3 Vùng bảo vệ** | 15 luật `tất cả` + 8 luật `gốc` + **các thư mục sao lưu tự thêm** (RB-65), cột "hậu quả nếu xóa", ô thử dán đường dẫn. Câu mô tả sửa theo RB-110 | `✖` không xác định được `ZaloData` → chỉ hiện phần ngoài Zalo |
| **NC-4 Lịch sử & nhật ký** | Bảng các lần dọn (thời gian, chế độ, đã xóa, thu về thật, **hoàn tất / hủy / kết thúc bất thường**). Cảnh báo nhật ký chứa đường dẫn đầy đủ. Xóa nhật ký cũ: **X2, xem trước, phân loại theo loại nhật ký, mặc định chỉ `daxoa_*`** | `∅` `Chưa có lần dọn nào được ghi nhận.` |
| **NC-5 Cài đặt** | Chính sách sao lưu (3 lựa chọn kèm hậu quả bằng lời) · bộ lọc nâng cao (bản nháp) · bộ lọc đã lưu · tài khoản Zalo · **Chỉ định thư mục ZaloData qua đúng một hàm hai lớp chặn cứng** (M-05) · thông tin bản dựng | Đổi bất kỳ bộ lọc nào → BQ chuyển ĐÃ HỦY, có băng xác nhận |

---

# 6. BẢNG PHÂN MỨC RỦI RO VÀ DẠNG XÁC NHẬN

## 6.1 Hai thang đo

**Thang rủi ro — đo bằng khả năng dựng lại**

| Mức | Định nghĩa | Câu hỏi kiểm tra |
|:-:|:---|:---|
| **R0** | Không đụng dữ liệu. Chỉ đọc, đo, báo cáo | Đóng công cụ ngay lúc này thì đĩa có khác gì không? |
| **R1** | Máy tự dựng lại **tại chỗ**, không cần mạng | Xóa xong mở lại ứng dụng là có lại chứ? |
| **R2** | Dựng lại được **nhưng có giá**: tải từ mạng, gián đoạn ứng dụng đang mở, hoặc để lại thay đổi tồn tại sau khi đóng công cụ | Người dùng phải trả bằng băng thông, thời gian, hay một phiên đang dở? |
| **R3** | Mất dữ liệu thật, còn **một bản thứ hai đã xác minh** hoặc còn đường lui khác | Nếu lớp xác minh sai thì mất vĩnh viễn chứ? Có → R3 |
| **R4** | **Mất vĩnh viễn, không đường lui trên máy** | Có thứ gì trên máy này dựng lại được nó không? |

> **Lô hỗn hợp lấy mức cao nhất.** Không trung bình, không lấy mức của mục chiếm nhiều byte nhất.

**Thang xác nhận**

| Dạng | Nội dung |
|:-:|:---|
| **X0** | Không hỏi gì. R0 và **mọi hướng an toàn** (hủy, dừng, quay lại) |
| **X1** | **Một nút ma sát tĩnh**: nhãn động từ + số lượng + dung lượng; không phải nút mặc định; không bind Enter; cách nút an toàn ≥96 dip; khóa mồi 600 ms tính từ lúc nút bật; đòi một lần nhả. **Ở mức này, mọi hành vi ghi-đĩa ngoài xóa tệp trong danh sách đều bị tắt cứng** |
| **X2** | **Hộp hậu quả**: liệt kê từng mục kèm *cái mất đi*; ô tick **mang số liệu** |
| **X3** | **Bằng chứng bắt buộc**: nút xám tới khi đã mở và đọc hết bảng cái-sắp-mất trong đúng lượt quét này + ô tick `Tôi đã xem danh sách` |
| **X4** | **X3 + gõ tay cụm từ.** Ba kiểu viết, phân biệt hoa thường, cấm dán, Enter bị nuốt |
| **X5** | **Chặn cứng**: giao diện không có đường dẫn tới hành động; lệnh lọt xuống lõi thì lõi từ chối và ghi nhật ký |

## 6.2 Bảng phân mức đầy đủ — 28 hành động

| # | Hành động | Cái mất đi | Khôi phục? | R | **Xác nhận chốt** | So với CLI |
|:-:|:---|:---|:---|:-:|:---|:---|
| 1 | Xóa dữ liệu Zalo theo bộ lọc | Ảnh, video, voice, tệp thật | Không, trừ khi có sao lưu | **R4** | **X4** `XÓA` + xem trước bắt buộc + huy hiệu ⛨ đo lại | đúng mức |
| 2 | Xóa dữ liệu Zalo theo mốc thời gian | như trên | Không | **R4** | **X4**, giống hệt #1 | đúng mức |
| 3 | Xóa mà bỏ qua sao lưu (chính sách HOI) | Đường lui duy nhất | Không | **R4** | Gộp vào **cùng trang X4**, là **ô tích mang số liệu**, không phải nút | **đã sửa** (M-02) |
| 4 | Xóa khi sao lưu chưa sạch | Đúng những tệp chưa sao lưu tốt | Không | **R4** | **X4** `TÔI CHẤP NHẬN MẤT` + **liệt kê tên tệp lỗi đầy đủ**; và **hành động mặc định là "chỉ xóa phần đã sao lưu sạch"** ở X4 thường | **đã sửa** (RB-71, -73) |
| 5 | Xóa bản trùng lặp | Bản thứ hai trong `resource\` | Không, nhưng bản giữ lại đã đối chiếu SHA-256 **và được stat lại ngay trước khi xóa** | **R3** | **X3** · bảng cặp *xóa/giữ* · không gõ chữ · **kiểm lại keeper từng tệp** | **đã siết** (RB-111) |
| 6 | Xóa cache ứng dụng Zalo | Cache, Code Cache, GPUCache, media\update, media\temp | Có | **R1** | **X1**, nâng lên **X3** nếu > 1.000 tệp hoặc > 1 GB | **đã siết** (QĐ-14) |
| 7 | Xóa cache hệ thống — mục XANH đã kiểm chứng | Cache dựng lại tại chỗ | Có | **R1** | **X1**, nâng **X3** theo ngưỡng tuyệt đối | **đã siết** |
| 8 | Xóa cache hệ thống — mục VÀNG | Phải tải lại từ mạng | Có, tốn băng thông | **R2** | **X2** + hiện **tổng dung lượng sẽ phải tải lại** | thiếu ở CLI |
| 9 | Xóa mục `verified:false` | Chưa rõ mất gì | Chưa biết | **R2** | **X2** + tick **riêng từng mục** | gần đúng |
| 10 | Xóa mục có `warning` | Theo mô tả của mục | Tùy | **R2** | **X2**; `*`/nhóm/`Ctrl+A` **không bao giờ chọn trúng** | đúng |
| 11 | "Cứ dọn hết" khi ứng dụng đang chạy | Cache đang dùng; ứng dụng tải lại ngay | Có, nhưng vô ích | **R2** | **X2**, lựa chọn *"bỏ các mục đó ra"* **được chọn sẵn**; **không kéo theo quyền cắt cụt** | đã sửa (RB-104) |
| 12 | Xóa `%TEMP%` (ngưỡng 24 giờ) | Tệp làm việc của ứng dụng đang mở | **Không** | **R2** | **X2** + nói rõ ngưỡng 24 giờ **ngay trong hộp**; **loại khỏi phạm vi cắt cụt** | đã sửa (RB-102) |
| 13 | **Cắt cụt tệp bị khóa về 0 byte** | Nội dung tệp đang bị giữ | Không | **R2** | **M6 riêng, mặc định TẮT, mặc định không chọn mục nào**, có tên tệp + tên tiến trình. **X5 chặn cứng** cho dữ liệu Zalo thật, bản trùng lặp, `%TEMP%` | **đã đảo mặc định** (M-07) |
| 14 | Dọn thư mục rỗng | Thư mục rỗng | Có | **R1** | Một dòng trong hộp xác nhận + công tắc; **tắt cứng ở X1** | đã sửa |
| 15 | **Khôi phục — ghi đè tệp đã tồn tại** | **Bản hiện tại** của tệp đích | **Không** | **R4** | **X4** `GHI ĐÈ` + **ba con số tách riêng** + cổng bắt buộc xem danh sách sẽ bị đè; **cờ hủy khi đổi đích** | **sai lệch nặng nhất của CLI, đã sửa** |
| 16 | Khôi phục sang thư mục khác | Có thể đè tệp lạ trùng tên | Không | **R3** | **X3** + **ép chế độ Giữ tệp hiện có**, không cho chọn ghi đè | đã siết |
| 17 | Khôi phục bản sao lưu có `SourceRoot = C:\` | Rải cache cũ đè lên cả ổ | Không | **R4** | **X5 — chặn cứng.** Chỉ mở thư mục cho người dùng tự chép | mới |
| 18 | **Hạ trần shadow storage** | Windows xóa bản chụp cũ cho vừa trần | Không | **R4** | **X3**; trần mới < mức đang dùng → tự nâng **X4**. Khóa cho tới sau khi dọn xong | đã sửa |
| 19 | Xóa bản chụp cũ nhất | Một điểm khôi phục | Không | **R3** | **X3** + hiện **ngày** của bản sắp mất. Khóa tới sau khi dọn xong | đã siết |
| 20 | Xóa toàn bộ bản chụp | Mọi điểm khôi phục + Previous Versions | Không | **R4** | **X4** `XÓA HẾT BẢN CHỤP` + **liệt kê ngày từng bản** + dòng RB-81. **Không đếm ngược.** Khóa tới sau khi dọn xong | đã sửa |
| 21 | Đóng Zalo (buộc dừng sau 7 giây) | Tin nhắn đang soạn, phiên đang mở, có thể hỏng CSDL | Không | **R2** | **X2**, nhãn nói ra bước buộc dừng; **bỏ buộc-dừng khỏi luồng tự động**; lựa chọn an toàn là mặc định | **đã sửa** |
| 22 | Xóa nhật ký cũ hơn N ngày | **Bằng chứng duy nhất** về những gì đã xóa | Không | **R2** | **X2**: số tệp khớp **phân loại theo loại nhật ký**, mặc định chỉ `daxoa_*` | thiếu ở CLI |
| 23 | Đổi chính sách sao lưu sang "Không hỏi" | Hạ ma sát cho **mọi lần xóa về sau** | Có | **R2** | **X2** + nhãn chính sách **thường trực** trên trang xóa | thiếu ở CLI |
| 24 | Mở lại với quyền quản trị | Mở rộng bán kính + **mất kết quả quét** | — | **R2** | **X1** + cảnh báo mất kết quả quét + **dấu hiệu nâng quyền nhìn thấy suốt phiên** | đã siết |
| 25 | Sao lưu — ghi vào thư mục đích | Chiếm chỗ ổ đích | Có | **R1** | **X1**, giữ kiểm chỗ trước khi chép byte nào, **cộng chặn cứng vị trí đích** | **đã siết** |
| 26 | **Chỉ định thư mục ZaloData thủ công** | Không mất gì trực tiếp, **nhưng đổi toàn bộ phạm vi của mọi lượt xóa sau** | — | **R3** | **X3** + hai lớp chặn cứng + dòng đỏ thường trực trên BQ | **mới, từ B3** |
| 27 | Xuất CSV / ghi nhật ký | Rò rỉ đường dẫn đầy đủ nếu đem chia sẻ | — | **R2** (riêng tư) | **X1** + cảnh báo in **trong chính tệp xuất ra**; nút "Mở thư mục chứa", **không có nút gửi đi đâu** | thiếu ở CLI |
| 28 | Đổi bộ lọc / đổi tài khoản → hủy kết quả quét | Công sức một lượt quét | Có | **R0** | **X0 — cấm hỏi.** Hỏi ở đây chính là chỗ dạy phản xạ bấm "Đồng ý" | đúng |

## 6.3 Mười nguyên tắc phân xử

| # | Nguyên tắc |
|:-:|:---|
| **N1** | Mức xác nhận đo bằng **khả năng dựng lại**, không bằng dung lượng. 40 GB cache npm nhẹ hơn 200 MB ảnh cưới |
| **N2** | Lô hỗn hợp lấy mức **cao nhất** |
| **N3** | Mỗi hành động đúng **MỘT cửa**, cửa đó chứa đủ mọi điều kiện. Ba hộp nối tiếp không tạo ba lần suy nghĩ, nó tạo một thói quen bấm ba lần |
| **N4** | **Ma sát phải mang thông tin.** Ô tick "Tôi hiểu 4 mục phải tải lại 3,2 GB" mang thông tin; "Tôi hiểu rủi ro" thì không |
| **N5** | **Gõ chữ chỉ dành cho R4.** Bảo vệ giá trị của tín hiệu quan trọng hơn tăng số lượng tín hiệu |
| **N6** | Bù bằng thời gian chỉ khi không bù được bằng bằng chứng. **Sau M-08, không còn ca nào thỏa điều kiện này** |
| **N7** | **Ma sát chỉ đặt trên hướng phá hủy.** Hủy, Dừng, Quay lại luôn là đường rẻ nhất |
| **N8** | **Không mức nào được tắt vĩnh viễn.** Người dùng đổi được chính sách sao lưu, không đổi được cửa xác nhận |
| **N9** | Con số trong hộp xác nhận **phải kèm nguồn và tuổi** |
| **N10** | Trạng thái "không thể xóa" hiện **ở NÚT**, không ở hộp thoại sau khi bấm |
| **N11** *(mới)* | **Cấm mọi nhãn khẳng định NGUYÊN NHÂN khi mã chỉ quan sát được KẾT QUẢ** (từ M-11) |
| **N12** *(mới)* | **Cấm mọi câu trấn an mà công cụ không chứng minh được.** Câu hứa phải thu về đúng phạm vi nó chứng minh được (từ C1, C4) |

## 6.4 Hai mươi mẫu giao diện bị CẤM

| # | Cấm | Thay bằng |
|:-:|:---|:---|
| 1 | Nút "Dọn ngay" / "Tối ưu một chạm" / "Boost" | Quét → xem → xác nhận. Luôn luôn |
| 2 | Thông báo "Hoàn tác" sau khi xóa | Trang kết quả + đường dẫn nhật ký + `[Khôi phục từ bản sao lưu]` **nếu** có bản sao lưu |
| 3 | Ô "Đừng hỏi tôi nữa" / ghi nhớ xác nhận | Chỉ chính sách sao lưu đổi được |
| 4 | Nút phá hủy là nút mặc định, hoặc bind Enter | Không có nút mặc định |
| 5 | Đếm ngược rồi tự động chạy | Bỏ đếm ngược hẳn |
| 6 | Khay hệ thống, khởi động cùng Windows, kiểm tra cập nhật ngầm | Đóng cửa sổ là hết tiến trình |
| 7 | Thông báo chủ động "Máy bạn có 12 GB rác" | Con số chỉ hiện khi người dùng mở công cụ |
| 8 | "Sức khỏe máy 42%", đồng hồ đo, điểm số, màu nháy | Số byte thật, số tệp thật, ngày tháng thật |
| 9 | Tick sẵn "chọn tất cả", mục rủi ro chọn mặc định | Mặc định **không chọn gì** |
| 10 | Thanh trượt "mức độ dọn dẹp Nhẹ/Vừa/Mạnh" | Danh sách mục, mỗi mục có dung lượng và mô tả mất gì |
| 11 | Biểu tượng thùng rác trên từng dòng bảng | Một nút hành động ở cuối trang cho cả lô đã xem |
| 12 | Phím tắt tới hành động xóa | Không có |
| 13 | Kéo–thả tệp/thư mục vào công cụ | Không hỗ trợ |
| 14 | Nhiều hộp thoại giống nhau nối tiếp | Một cửa duy nhất (N3) |
| 15 | Nút phá hủy dạng khối đỏ đặc, to nhất màn hình | Viền đỏ, chữ đỏ, nền trong suốt |
| 16 | Nhãn nút mơ hồ ("Dọn", "Áp dụng", "OK", "Xóa kết quả quét đang giữ") | Động từ + tân ngữ + số lượng + dung lượng |
| 17 | Hộp thoại xuất hiện dưới con trỏ | Khóa mồi 600 ms tính từ lúc nút bật + luật "phải nhả một lần" |
| 18 | Chỉ hiện tổng dung lượng, không hiện danh sách | Bảng danh sách là công dân hạng nhất |
| 19 | Ẩn trạng thái/chính sách sao lưu | Nhãn thường trực cạnh nút Xóa |
| 20 | Chỉ khoe "Đã giải phóng 15,05 GB" | Ba con số + dòng chênh lệch + lối đi tới trang bản chụp |
| **21** *(mới)* | **Nút "Ẩn ảnh xem trước"** | Làm mờ, hiện rõ khi rê chuột |
| **22** *(mới)* | **Toast tự tắt cho kết quả xóa**; hộp thoại tiến trình không hủy được | Kết quả nằm lại tới khi người dùng tự rời đi |
| **23** *(mới)* | **Chữ "SẠCH" đứng một mình**; "Xác minh: 0 lỗi" không kèm ngày và độ phủ | Độ phủ thật, tính lại mỗi lần vẽ |

---

# 7. KẾT QUẢ PHẢN BIỆN — 36 ĐƯỜNG TẤN CÔNG

**Tổng kết:** 36 đường · **10 CHẾT NGƯỜI** · **19 NẶNG** · **7 TRUNG BÌNH**. Sau vòng này: **28 đã bịt trong thiết kế** · **6 bịt một phần, còn rủi ro tồn dư đã ghi rõ** · **2 chưa bịt được, cần việc ngoài giao diện**.

## 7.1 Lăng kính "Người dùng vội và vô ý" (A1–A13)

| Mã | Đường tấn công | Mức | Bịt bằng | Trạng thái |
|:---|:---|:-:|:---|:---|
| **A1** | Huy hiệu "ĐÃ SAO LƯU · SẠCH" nói dối: USB đã rút, không ai kiểm lại lúc bấm Xóa | 🔴 CHẾT NGƯỜI | RB-59, RB-60, RB-61 (bộ mặt thứ sáu), QĐ-21 | ✅ **Đã bịt** |
| **A2** | Khôi phục bị ngắt để lại tệp chép dở; lần sau "Bỏ qua" giữ luôn tệp hỏng vĩnh viễn | 🔴 CHẾT NGƯỜI | RB-74, RB-75, RB-76 | ✅ **Đã bịt** |
| **A3** | Bản giữ lại chỉ kiểm lúc quét; người dùng xóa hội thoại trong Zalo giữa quét và xóa → mất cả hai bản | 🔴 CHẾT NGƯỜI | RB-111, RB-24 (ngưỡng 15 phút + hủy khi Zalo khởi động lại) | ✅ **Đã bịt** |
| **A4** | Trang chủ dụ người dùng phá bản chụp System Restore TRƯỚC khi dọn | 🔴 CHẾT NGƯỜI | QĐ-25, RB-80, RB-81, M-08 | ✅ **Đã bịt** — rủi ro tồn dư: sau khi dọn xong người dùng vẫn xóa được bản chụp; đó là **thứ tự đúng**, chấp nhận |
| **A5** | Mất điện giữa sao lưu → 8 GB không có manifest → công cụ tuyên bố "chưa có bản sao lưu nào" | 🟠 NẶNG | RB-67, RB-69 | ✅ **Đã bịt** |
| **A6** | Khôi phục xong bấm Xóa; kết quả quét cũ vẫn treo, BQ đi theo cả vào màn Khôi phục | 🟠 NẶNG | RB-21 (nguyên nhân 8), RB-20 (M-04), RB-19 | ✅ **Đã bịt** |
| **A7** | Cổng "đã xem" hai ghế định nghĩa mâu thuẫn → 5 cú bấm xóa 14,6 GB | 🟠 NẶNG | M-01, RB-27, RB-28, RB-42 | ✅ **Đã bịt** |
| **A8** | Khóa 400 ms chỉ tính từ lúc vẽ trang và chỉ chặn chuột; Space giữ để cuộn kích hoạt nút vừa bật | 🟠 NẶNG | RB-50, RB-51, RB-52, RB-53 | ✅ **Đã bịt** |
| **A9** | Hộp xác nhận hai bước, nút giữa là "Xóa luôn, không sao lưu" | 🟠 NẶNG | M-02, RB-30, RB-31, RB-32 | ✅ **Đã bịt** |
| **A10** | "Đóng Zalo giúp tôi" buộc dừng sau 7 giây, có thể hỏng chính CSDL tin nhắn | 🟠 NẶNG | RB-108, RB-109, RB-110 | ✅ **Đã bịt** |
| **A11** | Mất điện giữa lúc xóa → nhật ký, bằng chứng duy nhất, có thể trống | 🟠 NẶNG | QĐ-26, RB-95, RB-96, RB-97, RB-98 | ✅ **Đã bịt** |
| **A12** | Máy hai tài khoản: hộp chọn chỉ có dãy 19 chữ số → dọn nhầm tài khoản | 🟡 TRUNG BÌNH | §5.3 (6 ảnh thu nhỏ + ngày), RB-44, dòng nhận diện trong M1 | ⚠️ **Bịt một phần** — nếu hai tài khoản có nội dung ảnh giống nhau (cùng nhóm chat), ảnh thu nhỏ không phân biệt được. Xem §9 câu hỏi 9 |
| **A13** | Dọn trùng lặp trước, dọn theo mốc sau → bước hai xóa chính các bản giữ lại | 🟡 TRUNG BÌNH | RB-112, RB-90, bỏ dòng `→ Bắt đầu từ đây` | ✅ **Đã bịt** |

## 7.2 Lăng kính "Săn lỗi TRẠNG THÁI" (B1–B9)

| Mã | Đường tấn công | Mức | Bịt bằng | Trạng thái |
|:---|:---|:-:|:---|:---|
| **B1** | Thư mục sao lưu nằm trong chính thư mục sắp quét → bản sao lưu tự sát ở lượt sau; đường dẫn độc hại được ghi nhớ thành nút bấm sẵn | 🔴 CHẾT NGƯỜI | QĐ-22, RB-63, RB-65, RB-66, RB-91 | ✅ **Đã bịt** |
| **B2** | Sao lưu vào `%TEMP%` → bị cửa xác nhận **nhẹ nhất** của công cụ (X1, một nhấp) xóa mất | 🔴 CHẾT NGƯỜI | QĐ-22, RB-63, RB-65, **QĐ-14** (ngưỡng tuyệt đối), nhóm mới trong tab Đáng chú ý | ✅ **Đã bịt** |
| **B3** | "Chỉ định thư mục ZaloData" không kiểm dấu hiệu Zalo → biến Documents thành ZaloDownloads, mọi nhãn nói dối để củng cố nhầm lẫn | 🔴 CHẾT NGƯỜI | M-05, hành động #26 trong bảng phân mức, §5.5 (dòng đỏ thường trực + đường dẫn tuyệt đối) | ✅ **Đã bịt** |
| **B4** | Ổ đích đầy giữa sao lưu → không phát hiện, chép lỗi 3.318 tệp, rồi thiết kế đẩy vào `TÔI CHẤP NHẬN MẤT` thay vì mời xóa tập con đã sao lưu sạch | 🔴 CHẾT NGƯỜI | RB-70, RB-71, RB-72, RB-73, M7 | ✅ **Đã bịt** |
| **B5** | Cờ ghi-đè và câu `GHI ĐÈ` đã gõ vẫn sống sau khi đổi thư mục đích; con số ghi-đè bị gộp và che | 🔴 CHẾT NGƯỜI | RB-77, RB-78, RB-79 | ✅ **Đã bịt** |
| **B6** | "Tiếp tục phần còn lại" hồi sinh một lượt quét đã tiêu thụ, mang theo cờ đã-xem cũ | 🟠 NẶNG | M-03 (bỏ hẳn nút), RB-23, RB-24 | ✅ **Đã bịt** |
| **B7** | Bỏ tick một mục cache không hủy kết quả quét — tập mục không nằm trong hash bộ lọc | 🟠 NẶNG | RB-11, RB-12, RB-25 | ✅ **Đã bịt** |
| **B8** | Nút "Mở bản dòng lệnh" phá khóa một-cửa-sổ → hai tiến trình cùng thao tác trên một tập tệp | 🟠 NẶNG | QĐ-05, RB-07, RB-08, RB-19, RB-73 | ⚠️ **Bịt một phần — phụ thuộc việc sửa `ZaloCleanup.ps1`.** Nếu bản `.ps1` không lấy khóa chung thì **không được ship nút này** (RB-07) |
| **B9** | Cắt cụt về 0 byte ở mức X1 — X1 theo định nghĩa không có hộp xác nhận nào để công bố | 🟠 NẶNG | M-07, RB-101, RB-102, RB-103, RB-104, RB-105, M6 | ✅ **Đã bịt** |

## 7.3 Lăng kính "Săn HIỂU NHẦM" (C1–C14)

| Mã | Đường tấn công | Mức | Bịt bằng | Trạng thái |
|:---|:---|:-:|:---|:---|
| **C1** | "ĐÃ SAO LƯU · SẠCH" cấp bảo chứng cho 12.418 tệp sau khi chỉ đọc nội dung 50 tệp (0,4%) | 🔴 CHẾT NGƯỜI | QĐ-24, RB-62, RB-24 | ⚠️ **Bịt một phần** — nhãn đã nói thật, nhưng người dùng vẫn được quyền chọn mức mẫu. Câu hỏi mặc định để ngỏ, xem §9 câu 5 |
| **C2** | Sao lưu trong thư mục đang dọn | 🔴 CHẾT NGƯỜI | **Trùng B1** | ✅ Đã bịt |
| **C3** | Thẻ bản sao lưu là ảnh chụp quá khứ (đọc từ JSON) được trình bày như hiện trạng | 🔴 CHẾT NGƯỜI | RB-68, RB-69, RB-59 | ✅ **Đã bịt** |
| **C4** | "An toàn nhất — không mất tấm ảnh nào" là lời hứa về **byte**, người dùng đọc thành lời hứa về **Zalo** | 🟠 NẶNG | RB-90, N12, ô tick mang thông tin thay ô tick chung, bỏ `→ Bắt đầu từ đây` | ❌ **CHƯA BỊT ĐƯỢC HOÀN TOÀN.** Công cụ không đọc `Database\` (vùng bảo vệ) nên **không thể kiểm chứng** Zalo còn hiện được ảnh hay không. Giao diện đã thu câu hứa về đúng phạm vi chứng minh được, nhưng **câu trả lời thật đòi một lượt kiểm chứng tận nơi** — xem §9 câu 9 |
| **C5** | Hủy giữa chừng xong, BQ vẫn khoe con số trước khi xóa | 🟠 NẶNG | RB-23 (trạng thái ĐÃ DÙNG DỞ), RB-29 | ✅ **Đã bịt** |
| **C6** | "Chọn tất cả (bỏ qua mục có cảnh báo)" quét sạch cả mục VÀNG, chưa kiểm chứng, có `MinAge`, đang chạy | 🟠 NẶNG | RB-39, RB-40 | ✅ **Đã bịt** |
| **C7** | Xóa < 500 MB mà không thu được dung lượng thì màn hình kết quả **im lặng hoàn toàn** | 🟠 NẶNG | M-09, RB-86 | ✅ **Đã bịt** |
| **C8** | Con số to nhất màn hình là con số công cụ không kiểm soát và không quy trách nhiệm được (có thể âm) | 🟡 TRUNG BÌNH | RB-85, RB-91 | ✅ **Đã bịt** |
| **C9** | "vssadmin không trả về dữ liệu" hiện thành "KHÔNG có bản chụp" + câu hứa màu xanh | 🟠 NẶNG | RB-88, RB-87, RB-89 | ✅ **Đã bịt** |
| **C10** | Giao diện giới thiệu cắt cụt nhẹ hơn hẳn chú thích trong chính mã nguồn | 🟠 NẶNG | RB-103, RB-101 (đảo mặc định) | ✅ **Đã bịt** |
| **C11** | Nhãn "Đã không còn từ trước" khẳng định một nguyên nhân mà mã chưa hề kiểm; tệp **vẫn còn trên máy** | 🟡 TRUNG BÌNH | M-11, RB-119, N11 | ✅ **Đã bịt** |
| **C12** | "Đóng Zalo giúp tôi" đặt cạnh lời hứa tuyệt đối rằng CSDL bất khả xâm phạm | 🟠 NẶNG | **Trùng A10** + RB-110 | ✅ Đã bịt |
| **C13** | 12 ảnh ngẫu nhiên tạo cảm giác "tôi đã xem" cho 12.418 tệp; cổng mở khóa gắn sai tab | 🟠 NẶNG | M-01, RB-43, RB-42 | ✅ **Đã bịt** |
| **C14** | Sao lưu cùng ổ: xóa 9,72 GB, ổ C không rộng thêm byte nào, không lời giải thích | 🟡 TRUNG BÌNH | RB-64, RB-91, RB-61 (bộ mặt `CÙNG Ổ`) | ✅ **Đã bịt** |

## 7.4 Rủi ro tồn dư — nói thẳng là chưa xong

| # | Rủi ro còn lại | Vì sao chưa bịt được bằng giao diện | Việc phải làm |
|:-:|:---|:---|:---|
| **T-1** | **C4 — không kiểm chứng được Zalo còn hiện ảnh sau khi xóa `resource\`** | `Database\` nằm trong vùng bảo vệ; công cụ không đọc và không được đọc | **Một lượt kiểm chứng tận nơi trên máy thật**, có ảnh chụp trước/sau. Cho tới lúc đó: bỏ `→ Bắt đầu từ đây`, giữ câu hứa thu hẹp |
| **T-2** | **B8 — khóa liên tiến trình đòi sửa bản PowerShell** | Bản `.ps1` hiện không có khóa nào | Sửa `.ps1` lấy cùng mutex; nếu không thì **không ship nút "Mở bản dòng lệnh"** |
| **T-3** | **C1 — mức xác minh mẫu vẫn là 0,4%** | Xác minh 100% tốn ~6 phút cho 9,72 GB; ép 100% có thể khiến người dùng bỏ sao lưu hoàn toàn | Quyết định mặc định — §9 câu 5 |
| **T-4** | **A12 — nhận diện tài khoản** | Không đọc được tên hiển thị nếu nó nằm trong `Database\` | Xem có nguồn nào ngoài vùng bảo vệ đọc được tên/avatar không — §9 câu 9 |
| **T-5** | **ĐM-01 — trình đọc màn hình đi trọn vòng** | egui + AccessKit là **nền móng, chưa hoàn chỉnh**: không live region, không hộp thoại gốc, bảng không phơi quan hệ hàng–cột, đồ họa vẽ tay vô hình | Bù bằng thiết kế (§8), và giữ ĐM-08 làm đường lui. Chấp nhận là **cổng mức 2**, không phải mức 1 |
| **T-6** | **Ảnh xem trước cần bộ giải mã JPEG XL** | Zalo lưu `.jxl` và tệp không đuôi | Đo kích thước exe khi thêm bộ giải mã — §9 câu 10. Nếu không chấp nhận được thì ảnh `.jxl` hiện ô `?` và ma sát mạnh nhất bị yếu đi |
| **T-7** | **SmartScreen / diệt virus / ký số** | Ngoài phạm vi giao diện | Mục 8 của brief — §9 câu 1 |

---

# 8. DANH MỤC KIỂM TRA TIẾP CẬN

Ba mức cổng. **Mức 1 chặn hẳn bản phát hành, không thương lượng.**

## 8.1 Ba phép thử nếu chỉ được chọn ba

| # | Phép thử | ĐẠT khi |
|:-:|:---|:---|
| **1** | **Giữ phím Enter liên tục** từ Trang chủ, đi qua chọn nguồn · quét · xem danh sách, tới lúc trang xác nhận xóa mở, giữ thêm 5 giây. Lặp lại với **Space** và với **chuột nhấp liên tục vào tọa độ nút Xóa** | **0 tệp biến mất** cả ba lần |
| **2** | **Ảnh chụp greyscale** màn hình cache hệ thống, 3 người thử phân loại mức rủi ro | **33/33 đúng** |
| **3** | Gõ **`XOÁ`** bằng Unikey đặt dấu kiểu mới | Nút Xóa **bật** |

## 8.2 Tiếng Việt (TV)

| Mã | Yêu cầu | ĐẠT khi | Cổng |
|:---|:---|:---|:-:|
| TV-01 | Ba kiểu viết cụm xác nhận | `XÓA`, `XOÁ`, `XOA` đều mở khóa; `xóa` thường và `XÓAA` đều không | **1** |
| TV-02 | Chuẩn hóa NFC trước khi so | `nfd("XÓA")` được chấp nhận | **1** |
| TV-03 | Chuẩn hóa **chỉ để hiển thị** | `normalize_nfc()` không xuất hiện trong `core::` | **1** |
| TV-04 | Phông phủ đủ tiếng Việt | 134 chữ cái tiền tổ hợp + toàn bộ bảng chuỗi, `has_glyphs` true hết. Chạy trong CI, chặn merge | **1** |
| TV-05 | Không phụ thuộc phông máy | Chạy trên máy sạch không cài thêm phông | 2 |
| TV-06 | Chữ HOA có dấu không bị cắt | `Ổ Ữ Ỡ Ẫ Ặ ĐẦY — XÓA HẾT BẢN CHỤP` đủ hai tầng dấu ở 100/125/150/200% | 2 |
| TV-07 | Chiều cao dòng ≥ 1,45 × cỡ chữ | Một hằng số duy nhất trong theme | 3 |
| TV-08 | Nút và nhãn tự co giãn | Chế độ thử "chuỗi dài +40%", không chỗ nào tràn hay `…` | 3 |
| TV-09 | Bảng chuỗi tách riêng | `grep` chữ có dấu trong `core::`/`ui::` rỗng ngoài bảng chuỗi | 3 |
| TV-10 | Cụm xác nhận **không dịch được** | Test đột biến: đổi hằng số → test đỏ | **1** |
| TV-11 | Xưng hô nhất quán "bạn" | Rà thủ công một lượt | 3 |
| TV-12 | Nhật ký và CSV có BOM UTF-8 | Mở bằng Notepad trên Win 10 21H2 đúng chữ có dấu | 2 |
| TV-13 | Nhãn trạng thái nhật ký giữ nguyên byte | So SHA-256 hai nhật ký sinh từ cùng kịch bản (trừ hai nhãn mới của M-11) | 2 |

## 8.3 Vùng miền (VM)

| Mã | Yêu cầu | ĐẠT khi | Cổng |
|:---|:---|:---|:-:|
| VM-01 | Định dạng số cố định kiểu Việt | Chạy trên en-US và vi-VN cho chuỗi **giống hệt** | **1** |
| VM-02 | So sánh chuỗi/đường dẫn **ordinal** | Bộ test chạy dưới `tr-TR` không đổi kết quả | **1** |
| VM-03 | Ngày `dd/MM/yyyy` cố định | So đầu ra ở 3 locale | 2 |
| VM-04 | Ô ngày nhận 4 dạng; sai thì giữ nguyên | Test bảng 12 đầu vào, gồm `32/13/2025` và rỗng | 2 |
| VM-05 | Ô số từ chối `5,5` và `5.5` kèm giải thích | Test bảng: `5`→ok, `5,5`→lỗi, `5.5`→lỗi, `-1`→lỗi, ` 5 `→ok | **1** |
| VM-06 | Tham số `vssadmin` bất biến | Ghi lệnh thật đã gọi vào nhật ký | 2 |
| VM-07 | CSV mở đúng cột bằng Excel vi-VN | BOM + `sep=;` + `;`. Đếm cột = đúng số cột | 2 |
| VM-08 | Tên tệp sinh ra chỉ ASCII | Liệt kê `logs\` sau một lượt chạy đủ | 3 |
| VM-09 | `vssadmin` in nguyên văn | Chạy trên Windows tiếng Việt vẫn có nội dung | 2 |
| VM-10 | Cảnh báo trước UAC bằng tiếng Việt | Thử trên Windows en-US | 2 |

## 8.4 Chỉ dùng bàn phím (BP)

| Mã | Yêu cầu | ĐẠT khi | Cổng |
|:---|:---|:---|:-:|
| BP-01 | Mọi hành động làm được bằng bàn phím | Rút chuột, chạy trọn kịch bản chọn nguồn → quét → xem → sao lưu → xóa → xem nhật ký | **1** |
| BP-02 | Thứ tự Tab theo thứ tự đọc, nút phá hủy **cuối cùng** | Ghi 100% chặng Tab từng màn, so với thứ tự mong đợi | 2 |
| BP-03 | Vòng tiêu điểm ≥ 2 px, tương phản ≥ 3:1 với **cả hai** nền kề | Đo ở 4 tổ hợp theme | 2 |
| BP-04 | Hộp thoại giam tiêu điểm, nền vô hiệu hóa | Nhấn Tab 30 lần, mọi tiêu điểm nằm trong hộp thoại | **1** |
| **BP-05** | **Mười điều của trang xác nhận xóa** (§8.5) | Phép thử giữ Enter/Space/chuột | **1** |
| BP-06 | Esc luôn là Hủy | Thử 5 trạng thái ô nhập | 2 |
| BP-07 | Không phím tắt nào tới xóa | Gửi `Delete` ở mọi màn, 0 tệp mất | **1** |
| BP-08 | Esc dừng được thao tác đang chạy, nhật ký ghi "đã hủy giữa chừng" | Xóa 5.000 tệp trong sandbox, Esc giữa chừng, kiểm dòng tổng kết | **1** |
| BP-09 | Alt+F4 và ✕ khi đang xóa đi vào đúng đường dừng an toàn | Thử cả hai | 2 |
| BP-10 | `Ctrl+A` **không** chọn mục có `warning` | `Ctrl+A` rồi đếm | 2 |
| BP-11 | F6 / Shift+F6 nhảy giữa các khu vực lớn | Số phím để tới nút Quét từ lúc mở: **≤ 5** | 2 |
| **BP-12** *(mới)* | **Space trong vùng danh sách chỉ cuộn, không kích hoạt widget** | Giữ Space 10 giây ở màn Xem trước → 0 tệp bị xóa, 0 nút bị kích hoạt | **1** |

## 8.5 BP-05 — Mười điều của trang xác nhận xóa

1. **Không có nút mặc định.** Enter không kích hoạt gì, bất kể tiêu điểm ở đâu.
2. Ô nhập cụm từ **không submit khi Enter**.
3. Nút xóa **vô hiệu** tới khi chuỗi (đã NFC, đã cắt khoảng trắng, **phân biệt hoa thường**) khớp một trong ba kiểu viết.
4. Tiêu điểm mở vào **ô nhập**. Thứ tự Tab: ô nhập → **Hủy** → Xóa. Nút xóa dựng **cuối cùng**.
5. **Khóa mồi 600 ms tính từ MỖI LẦN nút chuyển sang bật**, áp cho cả chuột lẫn bàn phím.
6. **Bỏ sự kiện phím tự lặp.** Chỉ chấp nhận một lần nhấn **trọn vẹn** (key-down *và* key-up cùng xảy ra khi trang đang mở). Phím giữ từ màn trước không tính.
7. Esc = Hủy, luôn luôn.
8. Không phím tắt, không mnemonic nào trỏ vào nút xóa.
9. **Chặn dán** (`Ctrl+V`, chuột phải) và cụm từ in trên nhãn **không bôi đen sao chép được**.
10. Sau khi bấm Xóa: nút chuyển `Đang xóa… (Esc để dừng)` và không nhận thêm lần bấm nào.

## 8.6 Trình đọc màn hình (ĐM)

| Mã | Yêu cầu | ĐẠT khi | Cổng |
|:---|:---|:---|:-:|
| ĐM-01 | Kịch bản mù đi trọn vòng với NVDA, **bịt màn hình** | Không đứt mắt xích nào từ nghe dung lượng → quét → nghe số tệp → mở danh sách → nghe **toàn văn** cảnh báo → gõ `XÓA` → nghe nút đã bật → xóa → nghe kết quả | 2 |
| ĐM-02 | Mở hộp thoại là được đọc ngay | Trong 1 giây: tên hộp thoại + số tệp + dung lượng + cảnh báo nặng nhất | 2 |
| ĐM-03 | Mọi widget có tên và vai trò | Accessibility Insights: 0 phần tử thiếu Name, 0 sai ControlType. Xuất báo cáo, đính kèm bản phát hành | 2 |
| ĐM-04 | Trạng thái bị chặn được **thông báo**, không chỉ hiện ra | Sau sự kiện chặn, phần tử đang có tiêu điểm là khối giải thích | 2 |
| ĐM-05 | Thông báo an toàn không tự biến mất | Rà mã: không có bộ đếm tự đóng | 2 |
| ĐM-06 | Nút vô hiệu nói được lý do | Đọc "Xóa vĩnh viễn, nút, không khả dụng — cần gõ đúng chữ XÓA" | 2 |
| ĐM-07 | Có phím đọc tiến độ theo yêu cầu | Thử với NVDA | 2 |
| **ĐM-08** | **Phát hiện trình đọc màn hình và mở đường lui** | `SPI_GETSCREENREADER` bật → dải thông báo + `[Mở bản dòng lệnh]` (theo RB-07/-08) | **1** |
| ĐM-09 | Không tự chuyển tiêu điểm khi không cần | 60 giây thao tác, 0 lần đổi tiêu điểm ngoài ý muốn | 2 |

> **Bản dòng lệnh là đường tiếp cận chính thức cho người khiếm thị, không phải tác dụng phụ tình cờ.** Console của Windows phơi văn bản ra UIA rất tốt. Ghi điều này vào tài liệu phát hành.

## 8.7 DPI và nhiều màn hình (DPI)

| Mã | Yêu cầu | ĐẠT khi | Cổng |
|:---|:---|:---|:-:|
| DPI-01 | Manifest **PerMonitorV2** | Resource Hacker đọc được manifest; chữ sắc ở 150% | **1** |
| DPI-02 | Sắc nét ở 100/125/150/200% | Chụp `Ổ Ữ Ẫ`, phóng 400%, biên sắc | 2 |
| DPI-03 | Kéo sang màn DPI khác | Vẽ lại đúng cỡ trong ≤ 1 khung, bố cục không vỡ | 2 |
| DPI-04 | Vừa 1366×768 @125% | Duyệt hết mọi màn hình ở kích thước tối thiểu, không cuộn ngang | **1** |
| DPI-05 | Kích thước tối thiểu có ràng buộc (940×560) | Kéo tới cực tiểu, nút Hủy vẫn thấy, khoảng cách nút ≥ 48 dip | 2 |
| DPI-06 | Tôn trọng cỡ chữ Trợ năng của Windows | Đặt 150%, mở app, chữ to theo. Có thanh chỉnh 80–200% | 2 |
| DPI-07 | Vị trí cửa sổ khôi phục an toàn | Lưu vị trí trên màn 2, rút cáp, mở lại → về giữa màn chính | 2 |
| DPI-08 | Trang/hộp xác nhận mở đúng chỗ | Canh giữa **cửa sổ cha**, cùng màn hình, chặn cửa sổ cha | **1** |
| DPI-09 | Biểu tượng vector hoặc phông biểu tượng | Phóng 200%, biên không răng cưa | 3 |

## 8.8 Màu, tối, mù màu (MAU)

**Ba lớp mã hóa bắt buộc cho mỗi mức rủi ro:**

| Mức | ① Chữ (bắt buộc) | ② Ký hiệu (bắt buộc) | ③ Màu (phụ trợ) |
|:---|:---|:---|:---|
| Xanh | **An toàn — không mất dữ liệu** | `●` tròn đặc | xanh lá |
| Vàng | **Cần cân nhắc — phải tải lại từ mạng** | `▲` tam giác | vàng hổ phách |
| Đỏ | **Dữ liệu thật — mất vĩnh viễn** | `■` vuông + viền dày 2 px | đỏ |

| Mã | Yêu cầu | ĐẠT khi | Cổng |
|:---|:---|:---|:-:|
| MAU-01 | Bỏ hết màu vẫn hiểu | Greyscale, 3 người thử phân loại **33/33** | **1** |
| MAU-02 | Qua ba dạng mù màu | Mọi cặp trạng thái đối lập vẫn phân biệt được | 2 |
| MAU-03 | Tương phản chữ ≥ 4,5:1 (lớn ≥ 3:1), cả sáng và tối | Bảng đo tự động trong CI | 2 |
| MAU-04 | Tương phản viền ≥ 3:1 | Cùng công cụ | 2 |
| MAU-05 | Không đỏ bão hòa `#FF0000` trên nền tối | Đo tương phản | 2 |
| MAU-06 | Theme tối là bảng màu riêng, không phải đảo màu | Rà theme: hai bảng độc lập | 2 |
| MAU-07 | Theo theme Windows, có 3 lựa chọn | Đổi theme, app đổi trong ≤ 2 giây | 2 |
| MAU-08 | Hỗ trợ **Chế độ tương phản cao** | Bật High Contrast Black, chụp màn | 2 |
| MAU-09 | Nút phá hủy khác nút Hủy bằng **chữ + biểu tượng + vị trí** | Ảnh greyscale, người thử chỉ đúng nút Hủy | **1** |
| MAU-10 | Không dùng màu để báo xong/lỗi | Đọc bản greyscale | 2 |
| MAU-11 | Không nhấp nháy quá 3 lần/giây | Rà mã hoạt ảnh | 3 |

## 8.9 Từ ngữ — 28 chỗ phải thay

| # | Hiện tại | Thay bằng |
|:-:|:---|:---|
| 1 | `X Xóa kết quả quét đang giữ` | `Xóa vĩnh viễn 12.400 tệp đã quét (không qua Thùng rác)` + nút riêng `Bỏ kết quả quét này` |
| 2 | Shadow Copy · shadow storage · copy-on-write | `Bản chụp hệ thống (System Restore)`. Bỏ hẳn "copy-on-write" khỏi giao diện |
| 3 | `Hạ trần shadow storage` | `Giảm dung lượng Windows dành cho bản chụp` |
| 4 | `cache` | `tệp tạm — ứng dụng tự tạo lại khi cần` |
| 5 | `SHA-256` / `hash` / `băm` | `đối chiếu toàn bộ nội dung (SHA-256)` |
| 6 | `hash không khớp` | `nội dung bản chép khác bản gốc — không tin được` |
| 7 | `Khử trùng lặp` / dedup | `Bản sao thừa Zalo tự tạo` |
| 8 | `CẮTCỤT` | `Đã làm rỗng (tệp đang bị khóa; tên còn lại, nội dung đã trống)` |
| 9 | `BIẾNMẤT` | **Tách hai:** `Đã không còn` / `Không đọc được đường dẫn — CHƯA xóa, vẫn còn trên máy` |
| 10 | `THẤTBẠI` | `Không xóa được (đang bị ứng dụng khác giữ)` |
| 11 | `VÙNGBẢOVỆ` | Nhật ký giữ token; giao diện `Bị vùng bảo vệ chặn` |
| 12 | `junction` / `symbolic link` / `reparse point` | `lối tắt trỏ sang thư mục khác` |
| 13 | `tiến trình` (`đang chạy: node`) | `ứng dụng đang mở: Node.js` |
| 14 | `Controlled Folder Access` | `Tính năng chống mã độc tống tiền của Windows Defender đang bật — có thể chặn một số thao tác xóa` |
| 15 | `Windows đang tắt hỗ trợ đường dẫn dài` | Chuyển vào mục `Thông tin kỹ thuật` |
| 16 | `%LOCALAPPDATA%\npm-cache` | Đường dẫn thật; biến môi trường làm dòng phụ |
| 17 | `catalog.json` | `Danh mục vị trí cần dọn (tệp catalog.json)` |
| 18 | `Hồ sơ bộ lọc` | `Bộ lọc đã lưu` |
| 19 | `Xuất CSV` | `Lưu danh sách ra tệp Excel (.csv)` |
| 20 | `chưa kiểm chứng tận nơi` | `chưa được thử trên máy thật — hãy tự kiểm trước khi chọn` |
| 21 | `băng thông` | `dung lượng mạng` |
| 22 | `xác minh hai mức` | `kiểm lại bản đã chép: đối chiếu kích thước toàn bộ + nội dung 50 tệp mẫu (0,4%)` |
| 23 | `nâng quyền` / `elevate` / `admin` | `Mở lại với quyền quản trị (Windows sẽ hỏi bạn xác nhận)` |
| 24 | `bộ lọc không tự mở rộng phạm vi` | `Nhập sai thì công cụ giữ nguyên lựa chọn cũ, không tự chọn thêm gì` |
| 25 | `Gõ (không đuôi)` | Ô đánh dấu `Tệp không có phần đuôi — video Zalo thường như vậy` |
| 26 | `A,12,15` · `ok` · `*` · `-` · `admin` | Ô đánh dấu và nút bấm |
| 27 | `Ctrl+C bất cứ lúc nào` | `Nhấn Esc hoặc bấm Dừng — phần đã xóa vẫn được ghi vào nhật ký` |
| 28 | `Đo dung lượng trống thật` | `Kiểm lại ổ đĩa sau khi xóa — con số này mới là dung lượng bạn thực sự lấy về` |

**Ba câu phải viết lại hoàn toàn:**

> **Sự thật phản trực giác về bản chụp** — dòng đầu tiên khi số liệu không khớp:
> **Đã xóa 12,96 GB nhưng ổ đĩa chỉ rộng thêm 0,04 GB.**
> Công cụ không hỏng, và tệp đã mất thật. Windows đang giữ lại nội dung cũ để dành cho điểm khôi phục System Restore, nên dung lượng chỉ đổi chủ chứ chưa trả về ổ đĩa. Thư mục ĐÃ co lại đúng 12,96 GB.
> → `[Xử lý bản chụp để lấy dung lượng về]` · `[Để nguyên, tôi cần điểm khôi phục]`

> **Vùng bảo vệ** — thay câu "chặn cứng ở tầng code":
> Công cụ không bao giờ XÓA tệp trong `Database` và `Partitions` của Zalo. Không bộ lọc nào, kể cả bộ lọc do bạn tự đặt, chạm được vào chúng.
> *(Bỏ cụm "ngoài tầm với" — bước đóng Zalo là một đường công cụ vẫn chạm tới chúng.)*

> **Trước lệnh xóa dữ liệu thật:**
> Đây là ảnh và video thật của bạn, không phải tệp tạm. Chúng sẽ bị xóa hẳn, không vào Thùng rác, không lấy lại được. Ảnh quá hạn lưu trên máy chủ Zalo sẽ mất vĩnh viễn.

## 8.10 Bộ dữ liệu thử chuẩn — dùng chung cho mọi mục

**Chuỗi hiển thị**
```
Ổ Ữ Ỡ Ẫ Ặ Ộ Ợ Ề Ể Ỗ — ĐẦY Ứ HỰ                (dấu hai tầng, chữ HOA)
XÓA · XOÁ · XOA                                (ba kiểu đặt dấu)
TÔI CHẤP NHẬN MẤT · GHI ĐÈ · XÓA HẾT BẢN CHỤP
Bắt buộc sao lưu sạch mới cho xóa              (nhãn chính sách dài nhất)
chỉ tệp cũ hơn 24 giờ · đang chạy: node,chrome · cần quyền quản trị ·
chưa kiểm chứng tận nơi · có cảnh báo          (5 nhãn trên MỘT dòng, ps1:1268-1274)
```

**Đường dẫn**
```
NFC : ...\resource\1234\7594809871497_0987_1234_a1b2c3.jxl
NFD : cùng đường dẫn trên, dạng tổ hợp         (kiểm TV-03)
Dài : đường dẫn 265 ký tự                      (kiểm tiền tố \\?\)
```

**Số và ngày**
```
149.309 tệp · 37,21 GB · 12,96 GB · 0,04 GB · −1,20 GB · 41 MB/s
01/08/2026 · 31/12/2025
Hợp lệ  : 5 · 0 · 100
Từ chối : 5,5 · 5.5 · -1 · 5 GB · năm
```

**Môi trường phải chạy thử**
```
Windows 11 vi-VN @125% · Windows 11 en-US @100% · Windows 10 21H2 1366×768 @100%
Hai màn hình 125% + 200% · High Contrast Black · Cỡ chữ Trợ năng 150%
NVDA 2024+ · Narrator · Unikey kiểu dấu cũ · kiểu dấu mới · Unicode tổ hợp
```

---

# 9. VIỆC CHƯA QUYẾT — CÂU HỎI CHO CHỦ DỰ ÁN

| # | Câu hỏi | Vì sao phải hỏi trước | Nó chặn cái gì |
|:-:|:---|:---|:---|
| **1** | **Có ngân sách chứng chỉ ký mã hằng năm không?** | Chứng chỉ tự ký không làm SmartScreen im lặng; exe không ký + giao diện + xóa hàng loạt + gọi `vssadmin` là hồ sơ điển hình của báo động giả | Toàn bộ mục 8 của brief; T-7 |
| **2** | **Windows 10 có nằm trong phạm vi hỗ trợ không?** | Đã không còn chặn việc chọn khung (egui thắng ở mọi cột kể cả khi chỉ nhắm Win 11), nhưng vẫn quyết định mức đầu tư kiểm thử 1366×768 và ca thiếu `segoeui` | Kế hoạch kiểm thử |
| **3** | **Chấp nhận bao nhiêu crate phụ thuộc, và có sửa README bỏ dòng "0 phụ thuộc" không?** | egui kéo theo **113 crate**. Để dòng quảng cáo đó đứng lại là nói dối | README, tài liệu phát hành |
| **4** | **Có sửa `ZaloCleanup.ps1` NGAY BÂY GIỜ ba việc không?** ① `Test-ConfirmPhrase` thêm `XOÁ` + NFC ② đổi nhãn `X` ở `Show-AdvancedMenu`:2765 và README ③ thêm khóa liên tiến trình | ① và ② là **lỗi đang sống trên máy người dùng thật**, không phải chuyện của bản Rust. ③ chặn RB-07 | T-2; và người dùng hiện tại |
| **5** | **Mặc định mức xác minh sao lưu: mẫu 50 tệp (0,4%) hay toàn bộ (100%)?** | 100% mất ~6 phút cho 9,72 GB. Ép 100% có thể khiến người dùng bỏ sao lưu hẳn — tệ hơn nhiều. Thiết kế hiện đang: **mẫu là mặc định nhưng nhãn nói thật độ phủ; ổ tháo rời/ổ mạng thì đảo sang 100%** | T-3, RB-62 |
| **6** | **Ngưỡng tuổi kết quả quét: 2 giờ có quá dài không?** | Thiết kế đã hạ xuống 15 phút cho chế độ trùng lặp và 0 cho lượt đã tiêu thụ một phần. Còn lại giữ 2 giờ | RB-24 |
| **7** | **Có ship nút "Mở bản dòng lệnh" cho lớp trợ năng không, nếu `.ps1` chưa lấy khóa chung?** | Thiết kế trả lời **KHÔNG** (RB-07). Cần xác nhận vì nó đánh đổi với ĐM-08 | T-2, T-5 |
| **8** | **Máy thiếu `segoeui`/`arial`/`tahoma` (Windows N, LTSC gọn) thì xử lý sao?** | Hiện thiết kế là báo lỗi rõ ràng rồi dừng. Phương án khác: nhúng một phông tự do phủ Latin Extended Additional, tốn thêm vài trăm KB | RB-02, kích thước exe |
| **9** | **Ai kiểm chứng tận nơi hai việc này trên máy thật?** ① Zalo còn hiện ảnh trong hội thoại sau khi xóa `resource\` hay không ② có nguồn nào **ngoài vùng bảo vệ** đọc được tên/avatar tài khoản không | ① là T-1, đang chặn dòng `→ Bắt đầu từ đây` trên thẻ khuyến nghị đầu tiên. ② là T-4 | T-1, T-4 |
| **10** | **Có chấp nhận thêm bộ giải mã JPEG XL vào exe không?** | Zalo lưu `.jxl` và tệp không đuôi. Không có bộ giải mã thì **ma sát mạnh nhất của giao diện (12 ảnh thật + ảnh thu nhỏ trong cửa xác nhận) bị vô hiệu** cho phần lớn ảnh | T-6, kích thước exe |
| **11** | **Hai bản dùng chung `catalog.json` / `settings.json` / `profiles.json` / `logs\` hay mỗi bản một bộ?** | Brief mục 0 đòi trả lời. Khóa chung (QĐ-05) giải quyết được ca hai bản cùng mở, nhưng chưa giải quyết ca hai bản ghi `settings.json` lệch phiên bản | Kế hoạch port |
| **12** | **Có bỏ hẳn nút `[Tiếp tục phần còn lại]` không?** — thiết kế đã chốt BỎ (M-03), nhưng đây là chỗ duy nhất bản chốt lấy đi một tiện ích người dùng có thể muốn | Người dùng bấm Dừng để NHÌN, không phải để tạm nghỉ. Nếu chủ dự án muốn giữ, điều kiện tối thiểu đã ghi ở M-03 phương án 2 | Màn hình `⏹` |

---

# 10. BA VIỆC PHẢI LÀM TRƯỚC KHI VIẾT DÒNG GIAO DIỆN ĐẦU TIÊN

| # | Việc | Vì sao gấp |
|:-:|:---|:---|
| **1** | Sửa `Test-ConfirmPhrase` (`ZaloCleanup.ps1:189`) — thêm `XOÁ`, `XOÁ HẾT BẢN CHỤP`, chuẩn hóa NFC trước khi so. Thêm test bảng 8 đầu vào | **Lỗi đang sống trên máy người dùng thật.** Người gõ Unikey đặt dấu kiểu mới không xóa được và sẽ kết luận công cụ hỏng |
| **2** | Sửa nhãn `X` trong `Show-AdvancedMenu` (`ps1:2765`) và trong README | **Nhãn nguy hiểm nhất trong toàn bộ giao diện hiện tại.** Người Việt đọc "Xóa kết quả quét đang giữ" là "bỏ kết quả quét đi"; nó gọi `Invoke-Delete` và xóa vĩnh viễn tệp trên đĩa |
| **3** | Đưa `Invoke-Backup` về ngang `Invoke-Restore` ở hai điểm: bắt `ERROR_DISK_FULL` và dừng ngay; bỏ trần 200 dòng nhật ký lỗi (`ps1:1593`) | Đây là hai chỗ khiến B4 thành CHẾT NGƯỜI, và cả hai đều sửa được ngay trên bản PowerShell |

---

*Hết bản thiết kế chốt. Mọi mã RB-xx, QĐ-xx, M-xx, A/B/C-xx, T-xx trong tài liệu này là mã chính thức để phiên lập kế hoạch trích dẫn.*