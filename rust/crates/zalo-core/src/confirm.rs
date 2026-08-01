//! Cụm từ xác nhận.
//!
//! Tiếng Việt có hai kiểu đặt dấu đều đúng chính tả cho cùng một chữ: `XÓA` và
//! `XOÁ`. Bộ gõ đặt dấu kiểu nào là do người dùng chọn chứ không phải do họ gõ
//! sai. Bỏ dấu thanh rồi so với dạng không dấu, nhưng **vẫn phân biệt hoa
//! thường** — chữ thường phải bị từ chối, ma sát của bước xác nhận cuối cùng
//! không được nới ra.
//!
//! Đúng bốn cụm trong toàn dự án, không có cụm thứ năm. Mốc **M1**.

use unicode_normalization::UnicodeNormalization;

/// Bốn cụm xác nhận, dạng `(có dấu, không dấu)`. Hợp đồng với bản PowerShell.
pub const CUM_XAC_NHAN: [(&str, &str); 4] = [
    ("XÓA", "XOA"),
    ("TÔI CHẤP NHẬN MẤT", "TOI CHAP NHAN MAT"),
    ("GHI ĐÈ", "GHI DE"),
    ("XÓA HẾT BẢN CHỤP", "XOA HET BAN CHUP"),
];

/// Ký tự có phải dấu tổ hợp không.
///
/// Sau khi tách NFD, mọi dấu của tiếng Việt — sắc, huyền, hỏi, ngã, nặng, mũ,
/// trăng, râu — đều rơi vào khoảng `U+0300..U+036F`; râu (`U+031B`, cho `ư` và
/// `ơ`) cũng nằm trong đó.
///
/// Bản PowerShell lọc theo phạm trù Unicode `NonSpacingMark`, rộng hơn khoảng
/// này. Khác biệt chỉ lộ ra với chữ viết ngoài tiếng Việt, mà bốn cụm xác nhận
/// thì cố định và toàn tiếng Việt. Bộ đối chiếu song song kiểm đúng chỗ này.
fn la_dau_to_hop(c: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&c)
}

/// Bỏ dấu thanh, **giữ nguyên chữ hoa chữ thường**.
///
/// Chữ `Đ` và `đ` KHÔNG bị đụng tới: chúng là chữ cái riêng chứ không phải chữ
/// mang dấu thanh, nên NFD không tách chúng ra. Đúng như mong muốn.
pub fn bo_dau_thanh(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let khong_dau: String = s.nfd().filter(|c| !la_dau_to_hop(*c)).collect();
    khong_dau.nfc().collect()
}

/// Câu trả lời có khớp cụm xác nhận không.
///
/// Nhận mọi kiểu đặt dấu, nhưng **vẫn phân biệt hoa thường**: `xóa` bị từ chối.
/// Nới chuyện đặt dấu là sửa một lỗi thật; nới chuyện hoa thường là mài mòn ma
/// sát của bước xác nhận cuối cùng trước khi dữ liệu biến mất vĩnh viễn.
pub fn khop_cum_xac_nhan(tra_loi: &str, co_dau: &str, khong_dau: &str) -> bool {
    if tra_loi == co_dau || tra_loi == khong_dau {
        return true;
    }
    bo_dau_thanh(tra_loi) == khong_dau
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nhan_moi_kieu_dat_dau_nhung_van_phan_biet_hoa_thuong() {
        // Bảng này khớp từng dòng với bảng trong ZaloCleanup.Tests.ps1.
        let ca: &[(&str, bool, &str)] = &[
            ("XÓA", true, "dấu kiểu cũ"),
            ("XOÁ", true, "dấu kiểu mới — ca từng gây lỗi thật"),
            ("XOA", true, "không dấu"),
            ("xóa", false, "chữ thường"),
            ("xoá", false, "chữ thường, dấu kiểu mới"),
            ("Xóa", false, "viết hoa nửa vời"),
            ("XÓA ", false, "thừa dấu cách"),
            ("XÓAA", false, "thừa chữ"),
            ("", false, "rỗng"),
            ("CÓ", false, "chữ khác"),
        ];
        for (vao, mong_doi, vi_sao) in ca {
            assert_eq!(
                khop_cum_xac_nhan(vao, "XÓA", "XOA"),
                *mong_doi,
                "{vao:?} ({vi_sao})"
            );
        }
    }

    #[test]
    fn cum_nhieu_chu_cung_nhan_moi_kieu_dat_dau() {
        assert!(khop_cum_xac_nhan(
            "XOÁ HẾT BẢN CHỤP",
            "XÓA HẾT BẢN CHỤP",
            "XOA HET BAN CHUP"
        ));
        assert!(khop_cum_xac_nhan(
            "TOI CHAP NHAN MAT",
            "TÔI CHẤP NHẬN MẤT",
            "TOI CHAP NHAN MAT"
        ));
        assert!(!khop_cum_xac_nhan(
            "tôi chấp nhận mất",
            "TÔI CHẤP NHẬN MẤT",
            "TOI CHAP NHAN MAT"
        ));
    }

    #[test]
    fn giu_nguyen_chu_d_gach_ngang() {
        assert_eq!(bo_dau_thanh("ĐÃ XÓA"), "ĐA XOA");
        assert_eq!(bo_dau_thanh("GHI ĐÈ"), "GHI ĐE");
    }

    #[test]
    fn bo_dau_thanh_khong_doi_chuoi_khong_dau() {
        assert_eq!(bo_dau_thanh(""), "");
        assert_eq!(bo_dau_thanh("XOA"), "XOA");
        assert_eq!(
            bo_dau_thanh("C:\\Windows\\System32"),
            "C:\\Windows\\System32"
        );
    }

    #[test]
    fn bon_cum_deu_tu_khop_voi_chinh_no() {
        for (co_dau, khong_dau) in CUM_XAC_NHAN {
            assert!(khop_cum_xac_nhan(co_dau, co_dau, khong_dau), "{co_dau}");
            assert!(
                khop_cum_xac_nhan(khong_dau, co_dau, khong_dau),
                "{khong_dau}"
            );
        }
    }

    /// Ghim một chỗ phản trực giác, và ghim luôn rằng bản Rust khớp bản PowerShell.
    ///
    /// `Đ` là **chữ cái riêng**, không phải chữ mang dấu thanh, nên bỏ dấu KHÔNG
    /// đưa nó về `D`. Hệ quả: với `GHI ĐÈ`, đường bỏ dấu chỉ ra tới `GHI ĐE` chứ
    /// không tới `GHI DE`, nên người dùng phải gõ đúng một trong hai dạng đã khai.
    ///
    /// Đã đối chiếu tận nơi với bản PowerShell: cả hai đều cho `GHI ĐE`.
    #[test]
    fn chu_d_gach_ngang_khong_phai_dau_thanh_nen_khong_ve_d_thuong() {
        assert_eq!(bo_dau_thanh("GHI ĐÈ"), "GHI ĐE");
        assert_ne!(bo_dau_thanh("GHI ĐÈ"), "GHI DE");

        // Ba cụm không chứa Đ thì bỏ dấu ra đúng dạng không dấu đã khai.
        assert_eq!(bo_dau_thanh("XÓA"), "XOA");
        assert_eq!(bo_dau_thanh("TÔI CHẤP NHẬN MẤT"), "TOI CHAP NHAN MAT");
        assert_eq!(bo_dau_thanh("XÓA HẾT BẢN CHỤP"), "XOA HET BAN CHUP");

        // Nên với GHI ĐÈ, hai dạng đã khai vẫn nhận, còn dạng lai thì không.
        assert!(khop_cum_xac_nhan("GHI ĐÈ", "GHI ĐÈ", "GHI DE"));
        assert!(khop_cum_xac_nhan("GHI DE", "GHI ĐÈ", "GHI DE"));
        assert!(!khop_cum_xac_nhan("GHI ĐE", "GHI ĐÈ", "GHI DE"));
    }
}
