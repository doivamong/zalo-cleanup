//! Định dạng chữ và số ra màn hình.
//!
//! Mọi chuỗi ở đây là **hợp đồng với bộ test đầu-cuối**: các phép thử so thẳng
//! vào những câu này, nên đổi một chữ là đổi hành vi quan sát được của công cụ.
//!
//! # Một chỗ hai bản CỐ Ý khác nhau, và vì sao
//!
//! `'{0:N2}'` của PowerShell đổi theo vùng miền: `1,234.56` ở en-US nhưng
//! `1.234,56` ở vi-VN. Rust không có khái niệm vùng miền, nên bản này **luôn**
//! in kiểu en-US.
//!
//! Đây là khác biệt thật, đã biết, và cố ý chấp nhận ở mốc M3 vì:
//!
//! - Nó chỉ đụng tới **dấu phân cách**, không đụng tới con số.
//! - Bộ test đọc số bằng `Get-ReportedCount`, vốn bóc sạch ký tự không phải chữ
//!   số đúng để độc lập vùng miền — nên cổng M3 không bị nó che mắt.
//! - Không phép quyết định nào của công cụ đọc lại chuỗi đã định dạng.
//!
//! Ghi ra đây thay vì lặng lẽ để đó: một khác biệt không được viết xuống là một
//! khác biệt sẽ bị phát hiện lại từ đầu vào lúc bất tiện nhất.

/// Số nguyên kèm dấu phân cách hàng nghìn — tương ứng `'{0:N0}'`.
pub fn so(n: i64) -> String {
    let am = n < 0;
    let mut s = n.unsigned_abs().to_string();
    let mut ra = String::with_capacity(s.len() + s.len() / 3 + 1);
    while s.len() > 3 {
        let cat = s.split_off(s.len() - 3);
        ra.insert_str(0, &format!(",{cat}"));
    }
    ra.insert_str(0, &s);
    if am {
        ra.insert(0, '-');
    }
    ra
}

/// Số thực kèm dấu phân cách và đúng số chữ số thập phân — `'{0:N1}'`, `'{0:N2}'`.
fn so_le(x: f64, le: usize) -> String {
    let t = format!("{x:.le$}");
    match t.split_once('.') {
        Some((nguyen, thap)) => {
            let n: i64 = nguyen.parse().unwrap_or(0);
            format!("{}.{}", so(n), thap)
        }
        None => so(t.parse().unwrap_or(0)),
    }
}

/// Dung lượng dễ đọc. Tương ứng `Show-Size`.
///
/// Ngưỡng và số chữ số thập phân phải khớp từng bậc với bản PowerShell — bộ test
/// so thẳng chuỗi `0 B` ở nhánh cuối.
pub fn co(byte: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = 1024 * 1024;
    const GB: i64 = 1024 * 1024 * 1024;
    if byte >= GB {
        return format!("{} GB", so_le(byte as f64 / GB as f64, 2));
    }
    if byte >= MB {
        return format!("{} MB", so_le(byte as f64 / MB as f64, 1));
    }
    if byte >= KB {
        return format!("{} KB", so_le(byte as f64 / KB as f64, 0));
    }
    format!("{byte} B")
}

/// Tiêu đề có hai đường kẻ. Tương ứng `Write-Title`.
pub fn tieu_de(chu: &str) {
    let ke = "─".repeat(62);
    println!();
    println!("{ke}");
    println!("  {chu}");
    println!("{ke}");
}

/// Căn trái trong `rong` ô **đếm theo ký tự**, không theo byte.
///
/// `'{1,-16}'` của PowerShell đếm ký tự. Chữ Việt có dấu chiếm nhiều byte hơn
/// một ô, nên căn theo `len()` của Rust là cột lệch hẳn đi ở mọi dòng có dấu.
pub fn trai(s: &str, rong: usize) -> String {
    let n = s.chars().count();
    if n >= rong {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(rong - n))
    }
}

/// Căn phải, cũng đếm theo ký tự.
pub fn phai(s: &str, rong: usize) -> String {
    let n = s.chars().count();
    if n >= rong {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(rong - n), s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn so_co_dau_phan_cach_hang_nghin() {
        assert_eq!(so(0), "0");
        assert_eq!(so(7), "7");
        assert_eq!(so(999), "999");
        assert_eq!(so(1000), "1,000");
        assert_eq!(so(20000), "20,000");
        assert_eq!(so(1234567), "1,234,567");
        assert_eq!(so(-4321), "-4,321");
    }

    /// Bậc cuối phải ra đúng `0 B`: bộ test có một phép thử so thẳng chuỗi này
    /// để bắt lỗi hiện mốc thời gian rỗng.
    #[test]
    fn co_khop_tung_bac_voi_show_size() {
        assert_eq!(co(0), "0 B");
        assert_eq!(co(512), "512 B");
        assert_eq!(co(1023), "1023 B");
        assert_eq!(co(1024), "1 KB");
        assert_eq!(co(2048), "2 KB");
        assert_eq!(co(1024 * 1024), "1.0 MB");
        assert_eq!(co(1024 * 1024 * 3 / 2), "1.5 MB");
        assert_eq!(co(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(co(1024 * 1024 * 1024 * 5 / 2), "2.50 GB");
    }

    #[test]
    fn co_lon_van_co_dau_phan_cach() {
        assert_eq!(co(1024 * 1024 * 1024 * 1234), "1,234.00 GB");
    }

    /// Căn cột phải đếm ký tự, không đếm byte — chữ Việt có dấu là hai, ba byte.
    #[test]
    fn can_cot_dem_ky_tu_chu_khong_dem_byte() {
        assert_eq!(trai("Cũ hơn 12 tháng", 16).chars().count(), 16);
        assert_eq!(trai("abc", 5), "abc  ");
        assert_eq!(phai("abc", 5), "  abc");
        assert_eq!(phai("Trước năm 2026", 16).chars().count(), 16);
        assert_eq!(trai("dài hơn ô", 3), "dài hơn ô", "quá ô thì giữ nguyên");
    }
}
