# Dọn dẹp Zalo — bản phát hành

Ba tệp, dùng tệp nào là tùy bạn.

| Tệp | Là gì | Khi nào dùng |
|:---|:---|:---|
| `zalo-gui.exe` | Bản đồ họa | Muốn nhìn thấy ảnh sắp mất trước khi xóa |
| `zalo-cli.exe` | Bản dòng lệnh | Dùng trình đọc màn hình, hoặc thích chạy trong console |
| `ZaloCleanup.ps1` | Bản PowerShell, **mã nguồn đọc thẳng được** | Muốn tự đọc từng dòng trước khi chạy |

Cả ba đi qua **cùng một lõi an toàn** và cho **cùng kết quả** — có bộ test chạy
được cả ba để chứng minh điều đó sau mỗi commit.

---

## Trước khi chạy: công cụ này xóa vĩnh viễn

**Không qua Thùng rác. Không khôi phục được.** Ảnh và video quá hạn lưu trên máy
chủ Zalo sẽ mất hẳn.

Ba thứ luôn chặn đường bạn, và không tắt được:

1. **Không quét thì không xóa được.** Không có nút "dọn ngay".
2. **Không xem danh sách tệp sắp mất thì nút xóa không bật.**
3. **Phải gõ tay chữ `XÓA`.** Dán không được, và chữ thường không tính.

`Database` và `Partitions` — tin nhắn và phiên đăng nhập — bị **chặn cứng ở tầng
mã**. Không bộ lọc nào, kể cả bộ lọc do bạn tự đặt, chạm được vào chúng.

---

## Kiểm chứng tệp bạn vừa tải

Đây là phần thay thế cho việc đọc mã nguồn, thứ bạn mất khi dùng `.exe`.

**Bước 1 — so mã băm.** Mở PowerShell tại thư mục vừa tải:

```powershell
Get-FileHash .\zalo-gui.exe -Algorithm SHA256
```

So với dòng tương ứng trong `SHA256SUMS.txt`. Khác một ký tự là tệp không phải
bản chúng tôi dựng.

**Bước 2 — nếu bạn muốn chắc hơn nữa: đọc `ZaloCleanup.ps1`.**

Chúng tôi **đã thử** làm cho bạn dựng lại `.exe` và ra đúng từng byte, và **chưa
làm được**. Nói thẳng thay vì để bạn tự phát hiện:

| Đã bịt được | Chưa bịt được |
|:---|:---|
| Dấu thời gian trong đầu tệp PE | Thư viện CRT của Visual Studio là **đầu vào của bản dựng**, và không ghim được từ kho mã. Máy chủ CI đổi phiên bản Visual Studio theo từng lượt chạy |
| Đường dẫn tuyệt đối lọt vào tệp nhị phân | `zalo-gui.exe` còn **không tất định trên cùng một máy**: build script của thư viện đồ họa sinh mã theo thứ tự không cố định |

Đo được: `zalo-cli.exe` dựng lại **giống hệt** qua nhiều lượt và nhiều thư mục
trên cùng một máy — nhưng máy khác thì khác. `zalo-gui.exe` thì khác ngay ở lần
dựng thứ hai trên chính máy đó.

Nghĩa là `SHA256SUMS.txt` **chỉ** chứng minh tệp bạn tải về đúng là tệp máy chủ
CI đã dựng, không bị sửa trên đường. Nó **không** thay được việc đọc mã nguồn.

**Muốn kiểm chứng đến tận cùng thì dùng `ZaloCleanup.ps1`.** Nó là văn bản thuần,
bạn đọc được từng dòng, và nó làm đúng những việc như hai bản kia — có bộ test
lái được cả ba để chứng minh điều đó sau mỗi commit.

---

## Windows sẽ cảnh báo, và đây là lý do

Tệp `.exe` này **chưa được ký số**. Khi mở, SmartScreen sẽ hiện bảng xanh
*"Windows protected your PC"*, và hộp thoại UAC sẽ ghi **Unknown publisher**.

Chúng tôi nói thẳng thay vì bảo bạn "cứ bấm Run":

- **Chứng chỉ ký mã tốn phí hằng năm**, và dự án này chưa quyết định chi.
- **Chứng chỉ tự ký không giải quyết được gì** — SmartScreen vẫn cảnh báo.
- Ngay cả khi đã ký, chứng chỉ mới vẫn phải **tích lũy uy tín qua lượt tải**,
  nên người tải sớm vẫn gặp cảnh báo.

Một exe không ký, có giao diện, xóa hàng loạt tệp và gọi tới Volume Shadow Copy
là **hồ sơ điển hình** của báo động giả từ phần mềm diệt virus. Cảnh báo ấy
không sai — nó chỉ nói rằng Windows không biết ai làm ra tệp này.

**Nếu điều đó làm bạn ngại, hãy dùng `ZaloCleanup.ps1`.** Nó là văn bản thuần,
bạn đọc được từng dòng, và nó làm đúng những việc như hai bản kia.

---

## Công cụ này không tự chạy

Không Scheduled Task, không dịch vụ nền, không hook, không kết nối mạng. Nó chỉ
chạy khi bạn tự mở, và tắt là hết.

Nhật ký nằm trong `logs\` cạnh tệp thực thi. **Nhật ký chứa đường dẫn đầy đủ tới
từng tệp trên máy bạn** — đừng gửi nó cho ai mà chưa đọc lại.
