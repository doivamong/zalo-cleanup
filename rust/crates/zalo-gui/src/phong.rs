//! Phông chữ.
//!
//! # Vì sao đây là chuyện an toàn, không phải chuyện thẩm mỹ
//!
//! Người dùng phải **gõ đúng chữ `XÓA`** để xác nhận một lượt xóa vĩnh viễn.
//! Nếu phông thiếu glyph, chữ ấy hiện thành ô vuông rỗng — và người ta gõ theo
//! thứ họ nhìn thấy. Một công cụ xóa dữ liệu mà không hiện nổi cụm từ xác nhận
//! của chính nó là một công cụ đang lừa người dùng.
//!
//! Phông mặc định của egui **không có dấu tiếng Việt**, nên `default_fonts` bị
//! tắt hẳn ở `Cargo.toml`.
//!
//! # Chuỗi dự phòng, và vì sao không được dừng
//!
//! Hội đồng chốt: không tìm được phông thì báo lỗi rồi dừng. **Quyết định cuối
//! không theo** (`docs/quyet-dinh.md` §Q8) — một công cụ xóa dữ liệu mà chết vì
//! thiếu phông thì tệ hơn là chạy bằng phông thay thế.
//!
//! Thứ tự: `segoeui` → `arial` → `tahoma` → **phông nhúng sẵn**. Chốt chặn cuối
//! luôn có mặt, nên nhánh "không có phông nào" không tồn tại.

/// Phông nhúng sẵn — chốt chặn cuối, luôn có mặt.
///
/// DejaVu Sans, giấy phép Bitstream Vera (xem `assets/DejaVuSans.LICENSE.txt`).
/// Giá 756 KB trong tệp thực thi, và đó là cái giá đúng: xem chú thích đầu tệp.
pub const PHONG_NHUNG: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");

/// Tên phông hệ thống thử theo thứ tự. Segoe UI có mặt ở mọi bản Windows kể cả
/// bản N — bản N chỉ gỡ tính năng đa phương tiện, không gỡ phông hệ thống.
pub const CHUOI_DU_PHONG: [&str; 3] = ["segoeui.ttf", "arial.ttf", "tahoma.ttf"];

/// Mọi ký hiệu giao diện được phép dùng — và **chỉ** những ký hiệu này.
///
/// # Vì sao phải có bảng, không gõ thẳng ký hiệu vào chỗ vẽ
///
/// Thiết kế của hội đồng dùng `⛨` làm huy hiệu vùng bảo vệ. Phông nhúng **không
/// có glyph đó**, nên nó sẽ hiện thành ô vuông rỗng — và một huy hiệu an toàn
/// hiện thành ô vuông rỗng thì tệ hơn là không có huy hiệu nào.
///
/// Phát hiện được là nhờ hỏi thẳng phông chứ không nhờ nhìn màn hình. Gom vào
/// một bảng rồi cho phép thử quét cả bảng thì lỗi loại này không quay lại được:
/// thêm một ký hiệu mới mà phông thiếu là bộ test đỏ ngay.
///
/// Ba ký hiệu mức rủi ro là **bắt buộc** theo `MAU-01` — bỏ hết màu vẫn phải
/// phân loại đúng, nên chữ và ký hiệu gánh phần đó, màu chỉ là lớp phụ trợ.
pub mod bieu_tuong {
    /// Mức xanh — an toàn, không mất dữ liệu.
    pub const AN_TOAN: char = '\u{25CF}'; // ●
    /// Mức vàng — cần cân nhắc, phải tải lại từ mạng.
    pub const CAN_NHAC: char = '\u{25B2}'; // ▲
    /// Mức đỏ — dữ liệu thật, mất vĩnh viễn.
    pub const NGUY_HIEM: char = '\u{25A0}'; // ■
    /// Huy hiệu vùng bảo vệ. **Thay cho `⛨` của hội đồng**, vốn không có glyph.
    pub const VUNG_BAO_VE: char = '\u{2298}'; // ⊘
    /// Cảnh báo chung.
    pub const CANH_BAO: char = '\u{26A0}'; // ⚠
    /// Đã xong.
    pub const XONG: char = '\u{2713}'; // ✓
    /// Hỏng.
    pub const HONG: char = '\u{2716}'; // ✖
    /// Mũi tên dẫn hướng.
    pub const DAN_HUONG: char = '\u{2192}'; // →

    /// Cả bảng, để phép thử quét được.
    pub const TAT_CA: [char; 8] = [
        AN_TOAN,
        CAN_NHAC,
        NGUY_HIEM,
        VUNG_BAO_VE,
        CANH_BAO,
        XONG,
        HONG,
        DAN_HUONG,
    ];
}

/// Nguồn của phông đang dùng, để màn hình "về công cụ" nói được sự thật.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NguonPhong {
    HeThong(String),
    Nhung,
}

/// Nạp **chuỗi** phông: phông hệ thống trước, phông nhúng luôn đứng cuối.
///
/// # Vì sao là chuỗi chứ không phải một phông
///
/// Bản đầu chọn đúng một phông: hệ thống nếu nó phủ đủ chữ Việt, không thì
/// phông nhúng. Chạy thử thì màn hình hiện `? Xong.` — dấu `✓` thành dấu hỏi.
///
/// Đo tận nơi: **Segoe UI phủ đủ 134 chữ cái tiếng Việt nhưng thiếu bốn trên
/// tám ký hiệu** của bảng — `⊘ ⚠ ✓ ✖`. Phép kiểm cũ chỉ hỏi chữ cái nên nó qua,
/// rồi ký hiệu hiện thành ô vuông rỗng đúng ở những chỗ nói về an toàn.
///
/// Chuỗi phông sửa cả hai đầu: chữ vẫn là phông hệ thống quen mắt, còn glyph nào
/// thiếu thì rơi xuống phông nhúng. egui thử theo đúng thứ tự trong danh sách.
///
/// Phông nhúng **luôn** có mặt ở cuối chuỗi, nên nhánh "không có phông nào"
/// không tồn tại.
pub fn nap() -> (Vec<(String, Vec<u8>)>, NguonPhong) {
    let thu_muc = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    let mut chuoi: Vec<(String, Vec<u8>)> = Vec::new();
    let mut nguon = NguonPhong::Nhung;
    for ten in CHUOI_DU_PHONG {
        let p = std::path::Path::new(&thu_muc).join("Fonts").join(ten);
        if let Ok(b) = std::fs::read(&p) {
            // Phông hệ thống chỉ cần phủ đủ CHỮ CÁI; phần ký hiệu để phông nhúng
            // lo. Đòi nó phủ cả ký hiệu là loại luôn Segoe UI, tức đổi lấy một
            // giao diện lạ mắt để giải quyết một việc mà chuỗi dự phòng đã giải.
            if !b.is_empty() && du_chu_viet(&b) {
                chuoi.push((ten.to_string(), b));
                nguon = NguonPhong::HeThong(ten.to_string());
                break;
            }
        }
    }
    chuoi.push(("nhung".to_string(), PHONG_NHUNG.to_vec()));
    (chuoi, nguon)
}

/// Phông này có đủ glyph cho **toàn bộ bảng ký hiệu** không.
pub fn du_ky_hieu(byte: &[u8]) -> bool {
    use ab_glyph::{Font, FontRef};
    match FontRef::try_from_slice(byte) {
        Ok(f) => bieu_tuong::TAT_CA.iter().all(|c| f.glyph_id(*c).0 != 0),
        Err(_) => false,
    }
}

/// **134 chữ cái tiếng Việt tiền tổ hợp không thuộc ASCII.**
///
/// Mười hai nguyên âm `a ă â e ê i o ô ơ u ư y`, mỗi nguyên âm sáu dạng (không
/// dấu và năm thanh), nhân hai kiểu hoa thường, cộng `đ`/`Đ`, trừ đi mười hai
/// chữ vốn đã là ASCII. Sinh ra bằng bảng chứ không gõ tay: gõ tay 134 ký tự có
/// dấu là mời một lỗi chính tả nằm im trong chính phép thử canh chính tả.
pub fn chu_cai_tieng_viet() -> Vec<char> {
    // Mỗi hàng: sáu dạng của một nguyên âm, theo thứ tự không dấu · huyền · sắc
    // · hỏi · ngã · nặng.
    const BANG: [&str; 12] = [
        "aàáảãạ",
        "ăằắẳẵặ",
        "âầấẩẫậ",
        "eèéẻẽẹ",
        "êềếểễệ",
        "iìíỉĩị",
        "oòóỏõọ",
        "ôồốổỗộ",
        "ơờớởỡợ",
        "uùúủũụ",
        "ưừứửữự",
        "yỳýỷỹỵ",
    ];
    let mut ra: Vec<char> = Vec::new();
    for hang in BANG {
        for c in hang.chars() {
            ra.push(c);
            for h in c.to_uppercase() {
                ra.push(h);
            }
        }
    }
    ra.push('đ');
    ra.push('Đ');
    ra.retain(|c| !c.is_ascii());
    ra.sort_unstable();
    ra.dedup();
    ra
}

/// Phông này có đủ glyph cho toàn bộ chữ cái tiếng Việt không.
pub fn du_chu_viet(byte: &[u8]) -> bool {
    thieu_glyph(byte).map(|v| v.is_empty()).unwrap_or(false)
}

/// Các chữ cái tiếng Việt mà phông **thiếu** glyph. `None` nghĩa là không đọc
/// được phông.
pub fn thieu_glyph(byte: &[u8]) -> Option<Vec<char>> {
    use ab_glyph::{Font, FontRef};
    let f = FontRef::try_from_slice(byte).ok()?;
    Some(
        chu_cai_tieng_viet()
            .into_iter()
            .filter(|c| f.glyph_id(*c).0 == 0)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Con số 134 là của hội đồng. Nếu bảng sinh ra khác đi thì hoặc bảng sai,
    /// hoặc cách hiểu "chữ cái tiếng Việt" đã đổi — cả hai đều phải xem lại
    /// bằng tay chứ không sửa con số cho khớp.
    #[test]
    fn dung_134_chu_cai_tieng_viet() {
        let v = chu_cai_tieng_viet();
        assert_eq!(v.len(), 134, "sinh ra {} chữ, không phải 134", v.len());
        assert!(v.contains(&'Ổ'));
        assert!(v.contains(&'ữ'));
        assert!(v.contains(&'Đ'));
        assert!(!v.contains(&'a'), "chữ ASCII không được nằm trong bảng");
    }

    /// **TV-04, cổng MỨC 1.** Phông nhúng sẵn phải phủ đủ tiếng Việt — nó là
    /// chốt chặn cuối, nên nó hỏng thì không còn gì đỡ.
    #[test]
    fn phong_nhung_phu_du_tieng_viet() {
        let thieu = thieu_glyph(PHONG_NHUNG).expect("không đọc được phông nhúng");
        assert!(
            thieu.is_empty(),
            "phông nhúng thiếu {} chữ: {:?}",
            thieu.len(),
            thieu
        );
    }

    /// Mọi chuỗi trong giao diện phải hiện được, không riêng bảng chữ cái. Cụm
    /// xác nhận là chuỗi quan trọng nhất trong cả công cụ.
    #[test]
    fn phong_nhung_hien_duoc_cum_xac_nhan_va_chuoi_nang_nhat() {
        use ab_glyph::{Font, FontRef};
        let f = FontRef::try_from_slice(PHONG_NHUNG).unwrap();
        for s in [
            "XÓA",
            "XOÁ",
            "XOA",
            "TÔI CHẤP NHẬN MẤT",
            "Ổ Ữ Ỡ Ẫ Ặ ĐẦY — XÓA HẾT BẢN CHỤP",
            "Dữ liệu thật — mất vĩnh viễn",
            "Vùng bảo vệ đã chặn 157 tệp",
        ] {
            for c in s.chars() {
                assert_ne!(f.glyph_id(c).0, 0, "phông thiếu {c:?} trong chuỗi {s:?}");
            }
        }
    }

    /// Mọi ký hiệu trong bảng phải có glyph thật.
    ///
    /// Phép thử này bắt được `⛨` của hội đồng ngay lần chạy đầu — phông nhúng
    /// không có nó, nên huy hiệu vùng bảo vệ sẽ hiện thành ô vuông rỗng. Thêm
    /// một ký hiệu mới mà phông thiếu thì bộ test đỏ ngay tại đây.
    #[test]
    fn moi_ky_hieu_trong_bang_deu_co_glyph_that() {
        use ab_glyph::{Font, FontRef};
        let f = FontRef::try_from_slice(PHONG_NHUNG).unwrap();
        for c in bieu_tuong::TAT_CA {
            assert_ne!(
                f.glyph_id(c).0,
                0,
                "phông thiếu ký hiệu {c:?} (U+{:04X}) — nó sẽ hiện thành ô vuông rỗng",
                c as u32
            );
        }
    }

    /// Ba mức rủi ro phải là **ba ký hiệu khác nhau**. Trùng nhau thì lớp mã
    /// hóa thứ hai của `MAU-01` biến mất mà nhìn màn hình không thấy.
    #[test]
    fn ba_muc_rui_ro_dung_ba_ky_hieu_khac_nhau() {
        let b = [
            bieu_tuong::AN_TOAN,
            bieu_tuong::CAN_NHAC,
            bieu_tuong::NGUY_HIEM,
        ];
        let mut s = b.to_vec();
        s.sort_unstable();
        s.dedup();
        assert_eq!(s.len(), 3, "hai mức rủi ro đang dùng chung một ký hiệu");
    }

    /// Phông rác thì phải bị từ chối, không được nhận rồi vẽ ra ô vuông.
    #[test]
    fn phong_hong_bi_tu_choi() {
        assert!(!du_chu_viet(b"khong phai phong"));
        assert!(thieu_glyph(b"khong phai phong").is_none());
        assert!(!du_chu_viet(&[]));
    }

    /// Nạp phải luôn ra được một chuỗi dùng được, và phông nhúng **luôn** ở cuối.
    #[test]
    fn nap_luon_co_phong_nhung_o_cuoi_chuoi() {
        let (chuoi, _) = nap();
        assert!(!chuoi.is_empty());
        assert_eq!(
            chuoi.last().unwrap().0,
            "nhung",
            "phông nhúng phải là chốt chặn cuối của chuỗi"
        );
        for (ten, b) in &chuoi {
            assert!(!b.is_empty(), "phông {ten} rỗng");
        }
    }

    /// **Cả chuỗi gộp lại** phải phủ đủ chữ cái VÀ đủ ký hiệu.
    ///
    /// Đây là phép thử bắt được lỗi thật: Segoe UI phủ đủ 134 chữ cái nhưng
    /// thiếu bốn trên tám ký hiệu, nên bản một-phông hiện `? Xong.` thay vì
    /// `✓ Xong.`. Hỏi từng phông một thì không thấy; hỏi cả chuỗi thì thấy.
    #[test]
    fn ca_chuoi_gop_lai_phu_du_chu_cai_va_ky_hieu() {
        use ab_glyph::{Font, FontRef};
        let (chuoi, _) = nap();
        let fonts: Vec<FontRef> = chuoi
            .iter()
            .filter_map(|(_, b)| FontRef::try_from_slice(b).ok())
            .collect();
        assert!(!fonts.is_empty());
        let co = |c: char| fonts.iter().any(|f| f.glyph_id(c).0 != 0);

        let thieu_chu: Vec<char> = chu_cai_tieng_viet()
            .into_iter()
            .filter(|c| !co(*c))
            .collect();
        assert!(thieu_chu.is_empty(), "chuỗi phông thiếu chữ: {thieu_chu:?}");

        let thieu_kh: Vec<char> = bieu_tuong::TAT_CA.into_iter().filter(|c| !co(*c)).collect();
        assert!(
            thieu_kh.is_empty(),
            "chuỗi phông thiếu ký hiệu: {thieu_kh:?}"
        );
    }

    /// Phông nhúng một mình phải phủ đủ **cả hai** — nó là chốt chặn cuối, nên
    /// nó thiếu thì không còn gì đỡ.
    #[test]
    fn phong_nhung_mot_minh_phu_du_ca_chu_lan_ky_hieu() {
        assert!(du_chu_viet(PHONG_NHUNG));
        assert!(du_ky_hieu(PHONG_NHUNG));
    }

    /// Đo tận nơi và ghim lại: **Segoe UI thiếu ký hiệu**. Ngày nào Microsoft
    /// bổ sung chúng thì phép thử này đỏ, và người sửa đọc được lý do chuỗi
    /// phông tồn tại trước khi gỡ nó đi.
    #[test]
    fn segoe_ui_phu_du_chu_nhung_thieu_ky_hieu() {
        let p =
            std::path::Path::new(&std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into()))
                .join("Fonts")
                .join("segoeui.ttf");
        let b = match std::fs::read(&p) {
            Ok(b) => b,
            Err(_) => {
                eprintln!("CHÚ Ý: máy này không có segoeui.ttf nên bỏ qua.");
                return;
            }
        };
        assert!(du_chu_viet(&b), "Segoe UI phải phủ đủ chữ cái tiếng Việt");
        assert!(
            !du_ky_hieu(&b),
            "Segoe UI giờ đã phủ đủ ký hiệu — đọc lại chú thích ở `nap` trước khi gỡ chuỗi phông"
        );
    }
}
