//! Mức rủi ro, và **ba lớp mã hóa bắt buộc** của `MAU-01`.
//!
//! # Vì sao màu là lớp phụ trợ, không phải lớp chính
//!
//! Cổng `MAU-01` là cổng **mức 1**: bỏ hết màu vẫn phải phân loại đúng — ba
//! người thử nhìn ảnh chụp greyscale và xếp đúng 33/33 màn hình. Người mù màu
//! đỏ-lục chiếm khoảng 8% nam giới, và họ cũng xóa dữ liệu.
//!
//! Nên mỗi mức mang **ba lớp**: chữ, ký hiệu, rồi mới tới màu. Hai lớp đầu là
//! bắt buộc; bỏ lớp màu đi thì màn hình vẫn đọc được nguyên nghĩa.
//!
//! Gom vào một chỗ để phép thử hỏi được: hai mức dùng chung một chữ, hay chung
//! một ký hiệu, là lớp mã hóa ấy biến mất mà nhìn màn hình không thấy.

use crate::phong::bieu_tuong;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MucRuiRo {
    /// Không mất dữ liệu — cache, bản trùng lặp đã xác minh.
    AnToan,
    /// Mất thì phải tải lại từ mạng, tốn băng thông chứ không mất hẳn.
    CanNhac,
    /// Dữ liệu thật. Mất là mất vĩnh viễn.
    NguyHiem,
}

impl MucRuiRo {
    /// Lớp ① — **chữ**. Bắt buộc, và phải đọc được không cần nhìn gì khác.
    pub fn chu(self) -> &'static str {
        match self {
            MucRuiRo::AnToan => "An toàn — không mất dữ liệu",
            MucRuiRo::CanNhac => "Cần cân nhắc — phải tải lại từ mạng",
            MucRuiRo::NguyHiem => "Dữ liệu thật — mất vĩnh viễn",
        }
    }

    /// Lớp ② — **ký hiệu**. Bắt buộc. Chỉ lấy từ bảng đã kiểm có glyph thật.
    pub fn ky_hieu(self) -> char {
        match self {
            MucRuiRo::AnToan => bieu_tuong::AN_TOAN,
            MucRuiRo::CanNhac => bieu_tuong::CAN_NHAC,
            MucRuiRo::NguyHiem => bieu_tuong::NGUY_HIEM,
        }
    }

    /// Lớp ③ — **màu**. Phụ trợ. Bỏ hẳn lớp này thì màn hình vẫn phải đọc được.
    ///
    /// Không dùng đỏ bão hòa `#FF0000` trên nền tối (`MAU-05`): nó rung mắt và
    /// tương phản kém hơn vẻ ngoài.
    pub fn mau(self, nen_toi: bool) -> [u8; 3] {
        match (self, nen_toi) {
            (MucRuiRo::AnToan, false) => [0x1B, 0x5E, 0x20],
            (MucRuiRo::AnToan, true) => [0x81, 0xC7, 0x84],
            (MucRuiRo::CanNhac, false) => [0x8A, 0x5A, 0x00],
            (MucRuiRo::CanNhac, true) => [0xFF, 0xCC, 0x66],
            (MucRuiRo::NguyHiem, false) => [0xA5, 0x14, 0x14],
            (MucRuiRo::NguyHiem, true) => [0xFF, 0x8A, 0x80],
        }
    }

    /// Mức rủi ro của một loại quét. Đây là chỗ **duy nhất** ánh xạ hai thứ đó.
    pub fn tu_loai_quet(loai: &str) -> MucRuiRo {
        match loai {
            "BẢN TRÙNG LẶP" => MucRuiRo::AnToan,
            "CACHE ZALO" => MucRuiRo::AnToan,
            "CACHE HỆ THỐNG" => MucRuiRo::CanNhac,
            // Không biết thì ngã về phía NẶNG. Cùng lý do với `muc_xac_nhan`
            // bên bản dòng lệnh: quên một dòng là dữ liệu thật đi qua cửa nhẹ.
            _ => MucRuiRo::NguyHiem,
        }
    }

    pub const TAT_CA: [MucRuiRo; 3] = [MucRuiRo::AnToan, MucRuiRo::CanNhac, MucRuiRo::NguyHiem];
}

/// Độ chói tương đối theo WCAG, để đo tương phản.
fn choi(c: [u8; 3]) -> f64 {
    let f = |v: u8| {
        let s = v as f64 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2])
}

/// Tỷ lệ tương phản giữa hai màu, theo WCAG 2.1.
pub fn tuong_phan(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (x, y) = (choi(a), choi(b));
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **MAU-01, cổng mức 1.** Ba mức phải phân biệt được **không cần màu**.
    #[test]
    fn ba_muc_phan_biet_duoc_khong_can_mau() {
        let mut chu: Vec<&str> = MucRuiRo::TAT_CA.iter().map(|m| m.chu()).collect();
        chu.sort_unstable();
        chu.dedup();
        assert_eq!(chu.len(), 3, "hai mức đang dùng chung một câu chữ");

        let mut kh: Vec<char> = MucRuiRo::TAT_CA.iter().map(|m| m.ky_hieu()).collect();
        kh.sort_unstable();
        kh.dedup();
        assert_eq!(kh.len(), 3, "hai mức đang dùng chung một ký hiệu");
    }

    /// Chữ phải nói ra HẬU QUẢ, không chỉ dán nhãn mức độ. "Nguy hiểm" không
    /// cho người dùng biết cái gì sắp mất; "mất vĩnh viễn" thì có.
    #[test]
    fn chu_noi_ra_hau_qua_chu_khong_chi_dan_nhan() {
        assert!(MucRuiRo::NguyHiem.chu().contains("vĩnh viễn"));
        assert!(MucRuiRo::CanNhac.chu().contains("tải lại"));
        assert!(MucRuiRo::AnToan.chu().contains("không mất"));
    }

    /// Loại quét lạ phải ngã về phía nặng — cùng lý do với `muc_xac_nhan`.
    #[test]
    fn loai_quet_la_nga_ve_phia_nang() {
        assert_eq!(MucRuiRo::tu_loai_quet("DỮ LIỆU ZALO"), MucRuiRo::NguyHiem);
        assert_eq!(MucRuiRo::tu_loai_quet(""), MucRuiRo::NguyHiem);
        assert_eq!(MucRuiRo::tu_loai_quet("gì đó mới"), MucRuiRo::NguyHiem);
        assert_eq!(MucRuiRo::tu_loai_quet("BẢN TRÙNG LẶP"), MucRuiRo::AnToan);
    }

    /// **MAU-03.** Tương phản chữ ≥ 4,5:1 với nền tương ứng, cả sáng lẫn tối.
    #[test]
    fn mau_du_tuong_phan_o_ca_hai_theme() {
        const NEN_SANG: [u8; 3] = [0xFF, 0xFF, 0xFF];
        const NEN_TOI: [u8; 3] = [0x1E, 0x1E, 0x1E];
        for m in MucRuiRo::TAT_CA {
            let s = tuong_phan(m.mau(false), NEN_SANG);
            assert!(s >= 4.5, "{m:?} trên nền sáng chỉ {s:.2}:1, cần ≥ 4,5");
            let t = tuong_phan(m.mau(true), NEN_TOI);
            assert!(t >= 4.5, "{m:?} trên nền tối chỉ {t:.2}:1, cần ≥ 4,5");
        }
    }

    /// **MAU-05.** Không đỏ bão hòa trên nền tối.
    #[test]
    fn khong_do_bao_hoa_tren_nen_toi() {
        assert_ne!(MucRuiRo::NguyHiem.mau(true), [0xFF, 0x00, 0x00]);
    }

    /// **MAU-06.** Theme tối là bảng màu riêng, không phải đảo màu của bảng sáng.
    #[test]
    fn theme_toi_la_bang_mau_rieng_khong_phai_dao_mau() {
        for m in MucRuiRo::TAT_CA {
            let (s, t) = (m.mau(false), m.mau(true));
            assert_ne!(s, t);
            let dao = [255 - s[0], 255 - s[1], 255 - s[2]];
            assert_ne!(t, dao, "{m:?} ở theme tối chỉ là màu sáng bị đảo");
        }
    }

    /// Công thức tương phản phải đúng ở hai đầu đã biết, nếu không mọi phép đo
    /// ở trên đều vô nghĩa.
    #[test]
    fn cong_thuc_tuong_phan_dung_o_hai_dau_da_biet() {
        let den = [0, 0, 0];
        let trang = [255, 255, 255];
        assert!((tuong_phan(den, trang) - 21.0).abs() < 0.01);
        assert!((tuong_phan(trang, trang) - 1.0).abs() < 0.01);
    }
}
