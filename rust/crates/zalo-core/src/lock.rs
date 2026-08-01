//! Khóa một tiến trình một lúc, dùng chung với bản PowerShell.
//!
//! Tên khóa ở [`crate::contract::LOCK_NAME`] là hợp đồng giữa hai bản (`R-16`).
//!
//! Ba chi tiết bản PowerShell đã chốt, bản Rust phải theo: mutex bị bỏ rơi do
//! tiến trình trước chết được xử lý như **đã nhận khóa** chứ không phải lỗi;
//! dựng khóa thất bại thì **không chặn người dùng**; và tệp khóa mang PID để
//! câu thông báo nói được ai đang giữ. Mốc **M1**.
