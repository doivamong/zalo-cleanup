//! Lịch: ngày tháng, mốc tuổi, định dạng `dd/MM/yyyy`.
//!
//! Std không có lịch, và dự án không thêm crate ngày tháng cho một việc gói gọn
//! trong hai công thức đã biết. Mô-đun này **không có một dòng `unsafe` nào** —
//! phần duy nhất cần hệ điều hành là độ lệch múi giờ, lấy qua bọc an toàn
//! [`crate::sysinfo::lech_gio_dia_phuong`].
//!
//! # Vì sao phải theo giờ địa phương
//!
//! `LastWriteTime` của .NET là giờ **địa phương**, và bản PowerShell so mốc tuổi
//! bằng chính nó. So bằng giờ UTC là lệch nguyên một múi giờ — ở Việt Nam là
//! bảy tiếng, đủ để một tệp nhảy sang mốc khác.
//!
//! # `lui_thang` phải cắt ngày cho khớp .NET
//!
//! `AddMonths(-6)` của .NET **cắt** ngày về ngày cuối tháng đích khi tháng đó
//! ngắn hơn: 31/08 lùi sáu tháng ra 28/02, không phải 03/03. Làm khác đi là một
//! ngày dữ liệu bị xếp nhầm mốc, mỗi năm vài lần. Mốc **M3**.

use std::time::{SystemTime, UNIX_EPOCH};

/// Một ngày trên lịch, không kèm giờ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ngay {
    pub nam: i32,
    pub thang: u32,
    pub ngay: u32,
}

/// Số ngày kể từ 01/01/1970 của một ngày trên lịch.
///
/// Thuật toán lịch civil của Howard Hinnant — đúng với mọi năm dương lịch, kể
/// cả quy tắc năm nhuận 100/400 mà cách làm ngây thơ hay sai.
pub fn ngay_tu_lich(nam: i32, thang: u32, ngay: u32) -> i64 {
    let y = nam as i64 - if thang <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = thang as i64;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + ngay as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Ngược của [`ngay_tu_lich`].
pub fn lich_tu_ngay(mut z: i64) -> Ngay {
    z += 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    Ngay {
        nam: (y + if m <= 2 { 1 } else { 0 }) as i32,
        thang: m as u32,
        ngay: d as u32,
    }
}

/// Số ngày của một tháng, có tính năm nhuận.
pub fn so_ngay_trong_thang(nam: i32, thang: u32) -> u32 {
    match thang {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (nam % 4 == 0 && nam % 100 != 0) || nam % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

impl Ngay {
    /// Lùi `n` tháng, cắt ngày về cuối tháng đích y như `AddMonths` của .NET.
    pub fn lui_thang(self, n: u32) -> Ngay {
        let tong = self.nam as i64 * 12 + (self.thang as i64 - 1) - n as i64;
        let nam = tong.div_euclid(12) as i32;
        let thang = tong.rem_euclid(12) as u32 + 1;
        let ngay = self.ngay.min(so_ngay_trong_thang(nam, thang));
        Ngay { nam, thang, ngay }
    }

    /// Nửa đêm của ngày này, tính theo giờ địa phương, quy về mốc UTC.
    pub fn nua_dem(self) -> SystemTime {
        let giay_dia_phuong = ngay_tu_lich(self.nam, self.thang, self.ngay) * 86_400;
        let giay_utc = giay_dia_phuong - crate::sysinfo::lech_gio_dia_phuong();
        moc_tu_giay(giay_utc)
    }

    /// `dd/MM/yyyy` — đúng dạng bản PowerShell in ra.
    pub fn dinh_dang(self) -> String {
        format!("{:02}/{:02}/{:04}", self.ngay, self.thang, self.nam)
    }
}

fn moc_tu_giay(giay: i64) -> SystemTime {
    if giay >= 0 {
        UNIX_EPOCH + std::time::Duration::from_secs(giay as u64)
    } else {
        UNIX_EPOCH - std::time::Duration::from_secs((-giay) as u64)
    }
}

fn giay_tu_moc(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

/// Hôm nay theo giờ địa phương. Tương ứng `(Get-Date).Date`.
pub fn hom_nay() -> Ngay {
    let giay = giay_tu_moc(SystemTime::now()) + crate::sysinfo::lech_gio_dia_phuong();
    lich_tu_ngay(giay.div_euclid(86_400))
}

/// Ngày địa phương của một mốc thời gian — dùng cho `LastWriteTime` của tệp.
pub fn ngay_dia_phuong(t: SystemTime) -> Ngay {
    let giay = giay_tu_moc(t) + crate::sysinfo::lech_gio_dia_phuong();
    lich_tu_ngay(giay.div_euclid(86_400))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moc_lich_da_biet() {
        assert_eq!(ngay_tu_lich(1970, 1, 1), 0);
        assert_eq!(ngay_tu_lich(1970, 1, 2), 1);
        assert_eq!(ngay_tu_lich(1969, 12, 31), -1);
        assert_eq!(ngay_tu_lich(2000, 3, 1), 11017);
        // Mọi con số ở đây lấy từ chính .NET, không tự nhẩm:
        //   ([datetime]'2026-08-02' - [datetime]'1970-01-01').TotalDays
        assert_eq!(ngay_tu_lich(2026, 8, 2), 20667);
    }

    #[test]
    fn di_roi_ve_ra_dung_cho_cu() {
        for z in [-30000i64, -1, 0, 1, 11017, 20668, 50000] {
            let n = lich_tu_ngay(z);
            assert_eq!(ngay_tu_lich(n.nam, n.thang, n.ngay), z, "hỏng ở {z}");
        }
    }

    /// Quy tắc nhuận 100/400 là chỗ cách làm ngây thơ hay sai.
    #[test]
    fn nam_nhuan_theo_dung_quy_tac_tram_va_bon_tram() {
        assert_eq!(so_ngay_trong_thang(2024, 2), 29);
        assert_eq!(so_ngay_trong_thang(2025, 2), 28);
        assert_eq!(so_ngay_trong_thang(1900, 2), 28, "1900 KHÔNG nhuận");
        assert_eq!(so_ngay_trong_thang(2000, 2), 29, "2000 CÓ nhuận");
    }

    /// `AddMonths` của .NET cắt ngày về cuối tháng đích. Xem chú thích đầu tệp.
    #[test]
    fn lui_thang_cat_ngay_giong_dotnet() {
        let n = Ngay {
            nam: 2026,
            thang: 8,
            ngay: 31,
        };
        assert_eq!(
            n.lui_thang(6),
            Ngay {
                nam: 2026,
                thang: 2,
                ngay: 28
            }
        );
        let n = Ngay {
            nam: 2024,
            thang: 8,
            ngay: 31,
        };
        assert_eq!(
            n.lui_thang(6),
            Ngay {
                nam: 2024,
                thang: 2,
                ngay: 29
            },
            "năm nhuận phải ra 29/02"
        );
    }

    #[test]
    fn lui_thang_qua_ranh_gioi_nam() {
        let n = Ngay {
            nam: 2026,
            thang: 3,
            ngay: 15,
        };
        assert_eq!(
            n.lui_thang(12),
            Ngay {
                nam: 2025,
                thang: 3,
                ngay: 15
            }
        );
        assert_eq!(
            n.lui_thang(15),
            Ngay {
                nam: 2024,
                thang: 12,
                ngay: 15
            }
        );
    }

    #[test]
    fn dinh_dang_dung_dang_dd_mm_yyyy() {
        assert_eq!(
            Ngay {
                nam: 2025,
                thang: 4,
                ngay: 1
            }
            .dinh_dang(),
            "01/04/2025"
        );
        assert_eq!(
            Ngay {
                nam: 2025,
                thang: 11,
                ngay: 20
            }
            .dinh_dang(),
            "20/11/2025"
        );
    }

    #[test]
    fn hom_nay_nam_trong_khoang_hop_ly() {
        let h = hom_nay();
        assert!(h.nam >= 2024 && h.nam <= 2100, "năm lạ: {h:?}");
        assert!(h.thang >= 1 && h.thang <= 12);
        assert!(h.ngay >= 1 && h.ngay <= 31);
    }

    /// Mốc nửa đêm địa phương phải đổi về UTC rồi so lại ra đúng ngày cũ.
    #[test]
    fn nua_dem_dia_phuong_di_roi_ve_khong_lech_ngay() {
        let n = Ngay {
            nam: 2025,
            thang: 3,
            ngay: 10,
        };
        assert_eq!(ngay_dia_phuong(n.nua_dem()), n);
    }
}
