//! SHA-256 toàn tệp và chữ ký nhanh.
//!
//! Đo trên máy thật: đĩa cho 41 MB/s một luồng và 53 MB/s tám luồng, trong khi
//! SHA-256 chạy 840 MB/s khi dữ liệu đã ở trong RAM. **Nút cổ chai là đĩa,
//! không phải CPU** — đừng đặt kỳ vọng vào việc băm nhanh hơn.
//!
//! Tệp từ 128 KB trở xuống thì chữ ký nhanh đã đọc trọn cả tệp, nên nó CHÍNH
//! LÀ SHA-256 toàn tệp; đừng đọc lại lần nữa. Mốc **M2**.
