//! Vùng bảo vệ — lớp chặn cuối cùng trước khi một tệp bị xóa.
//!
//! Hai mức: `tất cả` chặn cả cây bên dưới, `gốc` chỉ chặn khi nhắm thẳng vào
//! chính nó. Thư mục gốc còn phải kiểm CHIỀU NGƯỢC: nhận một thư mục *chứa*
//! vùng bảo vệ cũng nguy hiểm y như nhận chính vùng bảo vệ.
//!
//! So chuỗi phải dùng **ordinal**, không theo vùng miền (`R-11`) — công cụ có
//! phép thử chạy dưới `vi-VN`, nơi bảng chữ so sánh khác.
//!
//! Mốc **M1**. Cổng: chạy lại bộ so sánh 57.144 đầu vào của phiên trước, lần
//! này đối chiếu PowerShell với Rust, phải ra 0 khác biệt.
