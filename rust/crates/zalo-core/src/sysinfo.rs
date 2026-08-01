//! Dung lượng trống, loại ổ, tiến trình, nâng quyền, Volume Shadow Copy.
//!
//! Chỗ duy nhất được phép dùng `unsafe`.
//!
//! **Dung lượng phải đo bằng chỗ trống thật của ổ đĩa**, không bao giờ bằng
//! tổng byte đã xóa (`R-12`). Đo được: xóa 12,96 GB chỉ thu về 0,04 GB khi máy
//! còn bản chụp System Restore, vì VSS chép khối cũ sang shadow storage.
//!
//! `vssadmin` bị bản địa hóa theo ngôn ngữ Windows — **không parse theo từ khóa
//! tiếng Anh**, in nguyên văn như bản PowerShell. Mốc **M2/M4**.
