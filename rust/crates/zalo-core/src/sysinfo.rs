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
//!
//! # Vì sao gọi thẳng Win32 thay vì thêm crate
//!
//! Std không phơi ra dung lượng trống. Dùng `GetDiskFreeSpaceExW` chứ không
//! phải `GetDiskFreeSpaceW`: hàm cũ trả về số cluster dạng 32 bit và **tràn ở ổ
//! lớn**, hàm mới trả thẳng số byte 64 bit.
//!
//! Lấy `lpFreeBytesAvailableToCaller` chứ không lấy tổng byte trống của ổ — đó
//! đúng là thứ `Get-PSDrive ... .Free` của bản PowerShell trả về, và cũng là con
//! số đúng khi ổ có đặt hạn ngạch. Mốc **M3**.

use std::path::Path;

/// Dung lượng còn trống mà người dùng hiện tại được phép dùng, tính bằng byte.
///
/// Trả về `-1` khi không hỏi được — **giống hệt** bản PowerShell, nơi bên gọi
/// dựa vào dấu âm để biết mà bỏ qua dòng dung lượng. Đừng đổi thành `0`: không
/// hỏi được và hết sạch chỗ là hai chuyện hoàn toàn khác nhau, mà một trong hai
/// sẽ làm màn hình chính báo ổ đĩa đã đầy.
pub fn byte_trong(duong_dan: &str) -> i64 {
    #[cfg(windows)]
    {
        let goc = match goc_o_dia(duong_dan) {
            Some(g) => g,
            None => return -1,
        };
        let mut rong: u64 = 0;
        let mut tong: u64 = 0;
        let mut trong: u64 = 0;
        let mut w: Vec<u16> = goc.encode_utf16().collect();
        w.push(0);
        // SAFETY: `w` là chuỗi UTF-16 kết thúc bằng NUL, còn sống suốt lời gọi,
        // và ba con trỏ ra đều trỏ vào biến cục bộ đã khởi tạo.
        let ok = unsafe { GetDiskFreeSpaceExW(w.as_ptr(), &mut rong, &mut tong, &mut trong) };
        if ok == 0 {
            return -1;
        }
        if rong > i64::MAX as u64 {
            return i64::MAX;
        }
        rong as i64
    }
    #[cfg(not(windows))]
    {
        let _ = duong_dan;
        -1
    }
}

#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct SystemTime16 {
    nam: u16,
    thang: u16,
    thu: u16,
    ngay: u16,
    gio: u16,
    phut: u16,
    giay: u16,
    mili: u16,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available_to_caller: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
    fn GetLocalTime(lp_system_time: *mut SystemTime16);
    fn GetSystemTime(lp_system_time: *mut SystemTime16);
}

/// Chênh lệch giữa giờ địa phương và giờ UTC, tính bằng giây.
///
/// Đây là **bọc an toàn** để [`crate::thoigian`] làm lịch mà không cần một dòng
/// `unsafe` nào — quy tắc Q1 của kế hoạch chỉ cho phép `unsafe` ở mô-đun này.
///
/// Đo bằng hiệu của hai đồng hồ chứ không đọc thông tin múi giờ: cách này đúng
/// với cả múi lệch 30 và 45 phút, mà không phải diễn giải quy tắc giờ mùa hè.
/// Đổi lại, độ lệch lấy được là của **thời điểm hiện tại** — tệp nằm bên kia
/// ranh giới giờ mùa hè có thể lệch một giờ. Với công cụ chia dữ liệu theo mốc
/// 6 và 12 tháng thì sai số ấy không đổi được kết luận nào.
pub fn lech_gio_dia_phuong() -> i64 {
    #[cfg(windows)]
    {
        let mut dp = SystemTime16::default();
        let mut utc = SystemTime16::default();
        // SAFETY: cả hai con trỏ đều trỏ vào biến cục bộ đã khởi tạo, đúng kích
        // thước `SYSTEMTIME`, và hàm chỉ ghi vào chúng.
        unsafe {
            GetLocalTime(&mut dp);
            GetSystemTime(&mut utc);
        }
        let g = |t: &SystemTime16| -> i64 {
            crate::thoigian::ngay_tu_lich(t.nam as i32, t.thang as u32, t.ngay as u32) * 86_400
                + t.gio as i64 * 3600
                + t.phut as i64 * 60
                + t.giay as i64
        };
        // Làm tròn về bội của 15 phút: hai lời gọi cách nhau vài micro giây có
        // thể rơi vào hai giây khác nhau, mà mọi múi giờ trên đời đều là bội của
        // 15 phút. Không làm tròn thì độ lệch thỉnh thoảng lệch đi một giây.
        let tho = g(&dp) - g(&utc);
        ((tho as f64 / 900.0).round() as i64) * 900
    }
    #[cfg(not(windows))]
    {
        0
    }
}

/// Gốc của ổ chứa đường dẫn, ví dụ `C:\`. `None` nếu đường dẫn không có gốc.
fn goc_o_dia(duong_dan: &str) -> Option<String> {
    // Đường dẫn UNC phải nhận ra bằng tay, y như bản PowerShell — không dựng
    // được `DriveInfo` từ `\\máy\chiasẻ`.
    if duong_dan.starts_with("\\\\") {
        let phan: Vec<&str> = duong_dan.trim_start_matches('\\').splitn(3, '\\').collect();
        if phan.len() >= 2 && !phan[0].is_empty() && !phan[1].is_empty() {
            return Some(format!("\\\\{}\\{}\\", phan[0], phan[1]));
        }
        return None;
    }
    let b = duong_dan.as_bytes();
    if b.len() >= 2 && b[1] == b':' && (b[0] as char).is_ascii_alphabetic() {
        return Some(format!("{}:\\", b[0] as char));
    }
    None
}

/// Nhãn ổ để hiển thị, ví dụ `C`. Tương ứng `Get-DriveLabel`.
pub fn nhan_o_dia(duong_dan: &str) -> String {
    match goc_o_dia(duong_dan) {
        Some(g) if g.as_bytes().get(1) == Some(&b':') => g[..1].to_string(),
        _ => o_he_thong().trim_end_matches(':').to_string(),
    }
}

/// Ổ hệ thống, ví dụ `C:`. Không giả định Windows nằm ở ổ C.
pub fn o_he_thong() -> String {
    match std::env::var("SystemDrive") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => "C:".to_string(),
    }
}

/// Thư mục gốc của ổ hệ thống, ví dụ `C:\`.
pub fn goc_he_thong() -> String {
    o_he_thong() + "\\"
}

/// Danh sách gốc của mọi ổ đĩa cố định đang gắn, ví dụ `["C:\\", "D:\\"]`.
///
/// Dùng để đi tìm bản sao lưu. Thử mở từng chữ cái chứ không gọi
/// `GetLogicalDrives`: ổ không sẵn sàng thì `read_dir` hỏng và ta bỏ qua, đúng
/// thứ bản PowerShell làm khi lọc `Where-Object { $null -ne $_.Free }`.
pub fn cac_o_dia() -> Vec<String> {
    let mut ra = Vec::new();
    for c in b'A'..=b'Z' {
        let g = format!("{}:\\", c as char);
        if byte_trong(&g) >= 0 && std::fs::read_dir(&g).is_ok() {
            ra.push(g);
        }
    }
    ra
}

/// Thư mục chứa tệp thực thi đang chạy — nơi đặt `settings.json`, `catalog.json`
/// và `logs\`.
///
/// Tương ứng `$PSScriptRoot` của bản PowerShell. Hai bản **phải cùng trỏ về một
/// chỗ** thì mới dùng chung được tệp cấu hình, nên bản Rust khi đem ra đối chiếu
/// được đặt cạnh `ZaloCleanup.ps1`.
pub fn thu_muc_cong_cu() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goc_o_dia_doc_dung_cac_dang_duong_dan() {
        assert_eq!(goc_o_dia(r"C:\a\b").as_deref(), Some(r"C:\"));
        assert_eq!(goc_o_dia(r"d:\").as_deref(), Some(r"d:\"));
        assert_eq!(
            goc_o_dia(r"\\may\chiase\thu").as_deref(),
            Some(r"\\may\chiase\")
        );
        assert_eq!(goc_o_dia("khong_co_goc"), None);
        assert_eq!(goc_o_dia(""), None);
        assert_eq!(goc_o_dia(r"\\chi_co_may"), None);
    }

    #[test]
    fn nhan_o_dia_tra_ve_mot_chu_cai() {
        assert_eq!(nhan_o_dia(r"C:\a"), "C");
        assert_eq!(nhan_o_dia(r"d:\x\y"), "d");
    }

    /// Đường dẫn hỏng phải trả `-1` chứ không phải `0`. Xem chú thích ở hàm.
    #[test]
    fn khong_hoi_duoc_thi_tra_am_mot_chu_khong_tra_khong() {
        assert_eq!(byte_trong("khong_phai_duong_dan"), -1);
        assert_eq!(byte_trong(""), -1);
    }

    #[cfg(windows)]
    #[test]
    fn o_he_thong_co_dung_luong_trong_duong() {
        let n = byte_trong(&goc_he_thong());
        assert!(n > 0, "ổ hệ thống phải báo được dung lượng trống, nhận {n}");
    }

    #[cfg(windows)]
    #[test]
    fn liet_ke_duoc_it_nhat_o_he_thong() {
        let ds = cac_o_dia();
        assert!(
            ds.iter().any(|d| d.eq_ignore_ascii_case(&goc_he_thong())),
            "không thấy ổ hệ thống trong {ds:?}"
        );
    }
}
