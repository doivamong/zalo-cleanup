//! Xóa · sao lưu · khôi phục · dọn thư mục rỗng.
//!
//! Nhận kết quả quét **đã được [`crate::gate`] duyệt**, không tự quyết.
//!
//! **Không bao giờ xóa đệ quy** (`R-10`): giữa lúc kết luận thư mục rỗng và lúc
//! ra lệnh xóa có một khe hở, tiến trình khác kịp ghi tệp vào đó thì xóa đệ quy
//! cuốn luôn tệp ấy mà không qua lớp kiểm vùng bảo vệ.
//!
//! **Chỉ đếm là đã xóa khi tệp thật sự biến mất** (`R-13`).
//!
//! Bản sao lưu phải tương thích ngược — xem [`crate::contract`]. Mốc **M4**.
