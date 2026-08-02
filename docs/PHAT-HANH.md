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

**Bước 2 — tự dựng lại.** Mã băm chỉ chứng minh tệp không bị sửa sau khi dựng.
Muốn chắc rằng chính bản dựng ấy đến từ mã nguồn này thì tự dựng lấy:

```powershell
git clone https://github.com/doivamong/zalo-cleanup
cd zalo-cleanup
powershell -NoProfile -ExecutionPolicy Bypass -File rust\tools\dung-phat-hanh.ps1
```

Mã băm in ra phải **trùng khít** với `SHA256SUMS.txt`. Build được làm cho tái lập
được: dấu thời gian trong đầu tệp PE bị vô hiệu, và mọi đường dẫn tuyệt đối bị
ánh xạ về tên cố định — nên máy bạn ra đúng byte như máy chủ CI.

Cần đúng phiên bản Rust ghim trong `rust-toolchain.toml`; `rustup` tự cài nó.

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
