//! Những giá trị là **hợp đồng giữa bản PowerShell và bản Rust**.
//!
//! Hai bản chạy song song trên cùng một máy, cùng một tập tệp, cùng một thư mục
//! sao lưu. Mỗi giá trị dưới đây mà lệch nhau là một cách hỏng âm thầm:
//! khóa lệch thì hai bản cùng chạy được, tên tệp bản kê lệch thì bản này không
//! khôi phục được bản sao lưu của bản kia.
//!
//! Phép thử ở cuối tệp **đọc thẳng `ZaloCleanup.ps1`** và bắt lỗi nếu lệch. Đó
//! là lưới đỡ duy nhất chống chuyện hai bản trôi xa nhau trong im lặng.

/// Tên mutex khóa "một tiến trình một lúc".
///
/// Phạm vi `Local` nên khóa theo từng phiên đăng nhập — đúng ý đồ, vì hai người
/// dùng trên cùng một máy có dữ liệu Zalo riêng.
///
/// Ràng buộc `R-16`. Bản PowerShell đã lấy đúng tên này từ commit `ac172e0`.
pub const LOCK_NAME: &str = r"Local\ZaloCleanup.singleton";

/// Tên tệp bản kê nằm trong mỗi thư mục sao lưu. Khôi phục sống nhờ tệp này.
pub const BACKUP_MANIFEST_FILE: &str = "_zalocleanup_backup.json";

/// Giá trị trường `Version` trong bản kê sao lưu.
///
/// # Đây là một cái bẫy, đọc kỹ trước khi "sửa cho đúng"
///
/// Công cụ là **v5** nhưng bản kê ghi **4**. Đó là hiện trạng đã phát hành, và
/// người dùng đang có bản sao lưu thật mang số 4 trên máy. Bản Rust phải ghi
/// đúng số 4. Đổi nó là đổi hợp đồng mà không được gì.
pub const BACKUP_MANIFEST_VERSION: u32 = 4;

/// Trường `Tool` trong bản kê sao lưu.
pub const BACKUP_MANIFEST_TOOL: &str = "ZaloCleanup";

/// Tên thư mục con của mỗi lần sao lưu, theo `yyyyMMdd_HHmmss`.
/// Bên trong giữ nguyên đường dẫn tương đối so với gốc quét.
pub const BACKUP_RUN_DIR_FORMAT: &str = "%Y%m%d_%H%M%S";

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Đọc mã nguồn bản PowerShell để đối chiếu hợp đồng.
    ///
    /// Tệp lưu UTF-8 **có BOM** — bắt buộc với PowerShell 5.1, thiếu BOM là mọi
    /// chữ tiếng Việt vỡ. BOM không ảnh hưởng phép tìm chuỗi con ở đây.
    fn doc_ban_powershell() -> String {
        let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../ZaloCleanup.ps1");
        std::fs::read_to_string(&p)
            .unwrap_or_else(|e| panic!("không đọc được {}: {e}", p.display()))
    }

    #[test]
    fn ten_khoa_khop_voi_ban_powershell() {
        let ps = doc_ban_powershell();
        assert!(
            ps.contains(LOCK_NAME),
            "Tên khóa lệch. Bản Rust dùng {LOCK_NAME:?} nhưng không thấy chuỗi đó \
             trong ZaloCleanup.ps1. Lệch tên khóa nghĩa là hai bản CÙNG CHẠY ĐƯỢC \
             trên một tập tệp — xem R-16."
        );
    }

    #[test]
    fn ten_tep_ban_ke_khop_voi_ban_powershell() {
        let ps = doc_ban_powershell();
        assert!(
            ps.contains(BACKUP_MANIFEST_FILE),
            "Tên tệp bản kê lệch. Lệch tên nghĩa là bản này không khôi phục được \
             bản sao lưu của bản kia."
        );
    }

    #[test]
    fn so_hieu_ban_ke_van_la_bon() {
        let ps = doc_ban_powershell();
        let mong_doi =
            format!("Tool = '{BACKUP_MANIFEST_TOOL}'; Version = {BACKUP_MANIFEST_VERSION}");
        assert!(
            ps.contains(&mong_doi),
            "Không thấy {mong_doi:?} trong ZaloCleanup.ps1. Công cụ là v5 nhưng bản \
             kê ghi Version = 4; đó là hiện trạng đã phát hành và người dùng đang \
             có bản sao lưu mang số đó. Đừng sửa cho đúng."
        );
    }
}
