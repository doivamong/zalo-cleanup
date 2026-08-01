//! Duyệt cây thư mục.
//!
//! **KHÔNG đi xuyên reparse point** (`R-09`). `walkdir` mặc định không theo
//! symlink, nhưng junction NTFS không phải symlink — phải dựng junction thật
//! rồi đo, giống hệt cách đã kiểm bộ duyệt PowerShell. Không đạt thì tự viết
//! bằng `windows-sys`, tốn thêm chừng 60 dòng.
//!
//! Đếm được lỗi truy cập chứ không nuốt. Mốc **M2**.
