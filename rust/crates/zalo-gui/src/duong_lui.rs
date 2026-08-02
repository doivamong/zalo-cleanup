//! `ĐM-08` — đường lui sang bản dòng lệnh.
//!
//! # Bản dòng lệnh là đường tiếp cận CHÍNH THỨC, không phải tác dụng phụ
//!
//! Hội đồng ghi thẳng điều đó. Console của Windows phơi văn bản ra UIA rất tốt,
//! còn egui với AccessKit thì mới là **nền móng**: chưa có live region, chưa có
//! hộp thoại gốc, bảng không phơi quan hệ hàng–cột, đồ họa vẽ tay thì vô hình.
//!
//! Nên khi phát hiện trình đọc màn hình, giao diện phải **nói ra** và mở sẵn
//! đường lui. Đây là mục cổng mức 1.
//!
//! # Vì sao chỉ THÊM đường lui, không bao giờ bớt gì
//!
//! `SPI_GETSCREENREADER` là cờ do chính trình đọc màn hình bật lên, và không
//! phải trình nào cũng bật. Nên phép dò này chỉ được dùng để **thêm** một lối
//! đi. Dùng nó để đổi hành vi — ẩn bớt màn hình, đơn giản hóa giao diện — là
//! phạt người dùng vì một phép dò có thể sai.

/// Tên tệp bản dòng lệnh, tìm ngay cạnh tệp thực thi đồ họa.
pub const TEN_BAN_DONG_LENH: &str = "zalo-cli.exe";

/// Câu nói ra khi thấy trình đọc màn hình. Là câu **giải thích**, không phải
/// lời xin lỗi: người dùng cần biết vì sao nên chọn bản kia, chứ không cần biết
/// rằng công cụ này thấy tiếc.
pub const LOI_NHAN: &str = "Phát hiện trình đọc màn hình. Bản dòng lệnh của công cụ này \
đọc màn hình tốt hơn hẳn — nó là đường tiếp cận chính thức, không phải bản rút gọn.";

/// Đường dẫn tới bản dòng lệnh, nếu nó nằm cạnh tệp thực thi hiện tại.
pub fn tim_ban_dong_lenh() -> Option<std::path::PathBuf> {
    let p = zalo_core::sysinfo::thu_muc_cong_cu().join(TEN_BAN_DONG_LENH);
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}

/// Trạng thái dải thông báo đường lui.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuongLui {
    /// Không thấy trình đọc màn hình — không hiện gì.
    Khong,
    /// Thấy trình đọc màn hình **và** có bản dòng lệnh cạnh bên.
    Co(std::path::PathBuf),
    /// Thấy trình đọc màn hình nhưng **không tìm ra** bản dòng lệnh.
    ///
    /// Vẫn phải hiện dải thông báo, và nói thật là không mở hộ được. Im lặng ở
    /// đây là để người dùng ngồi trước một giao diện họ đọc không nổi mà không
    /// biết có đường khác.
    CoNhungThieuTep,
}

impl DuongLui {
    pub fn do_hien_tai() -> Self {
        if !zalo_core::sysinfo::co_trinh_doc_man_hinh() {
            return DuongLui::Khong;
        }
        match tim_ban_dong_lenh() {
            Some(p) => DuongLui::Co(p),
            None => DuongLui::CoNhungThieuTep,
        }
    }

    pub fn nen_hien(&self) -> bool {
        !matches!(self, DuongLui::Khong)
    }

    /// Câu hiện trên dải thông báo.
    pub fn cau(&self) -> String {
        match self {
            DuongLui::Khong => String::new(),
            DuongLui::Co(_) => LOI_NHAN.to_string(),
            DuongLui::CoNhungThieuTep => format!(
                "{LOI_NHAN} Không tìm thấy {TEN_BAN_DONG_LENH} cạnh tệp này, \
                 nên phải mở tay."
            ),
        }
    }

    /// Mở bản dòng lệnh trong một cửa sổ console riêng.
    pub fn mo(&self) -> bool {
        match self {
            DuongLui::Co(p) => std::process::Command::new("cmd")
                .args(["/c", "start", "", &p.to_string_lossy()])
                .spawn()
                .is_ok(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Không thấy trình đọc màn hình thì **không hiện gì** — dải thông báo luôn
    /// hiện là một dải thông báo người ta học cách không đọc.
    #[test]
    fn khong_thay_trinh_doc_thi_khong_hien_gi() {
        assert!(!DuongLui::Khong.nen_hien());
        assert_eq!(DuongLui::Khong.cau(), "");
        assert!(!DuongLui::Khong.mo(), "không được mở gì khi chưa cần");
    }

    /// Thiếu tệp thì **vẫn phải hiện**, và nói thật là không mở hộ được.
    ///
    /// Đây là ngã dễ bỏ quên nhất: im lặng ở đây nghĩa là người dùng ngồi trước
    /// một giao diện họ đọc không nổi mà không biết có đường khác.
    #[test]
    fn thieu_tep_van_hien_va_noi_that() {
        let d = DuongLui::CoNhungThieuTep;
        assert!(d.nen_hien());
        assert!(d.cau().contains(TEN_BAN_DONG_LENH));
        assert!(d.cau().contains("mở tay"));
        assert!(!d.mo(), "không có tệp mà vẫn báo mở được");
    }

    #[test]
    fn co_duong_lui_thi_hien_va_noi_ro_ly_do() {
        let d = DuongLui::Co(std::path::PathBuf::from(r"C:\x\zalo-cli.exe"));
        assert!(d.nen_hien());
        assert!(d.cau().contains("đường tiếp cận chính thức"));
    }

    /// Câu nhắn phải **giải thích**, không phải xin lỗi hay hạ thấp bản kia.
    /// "Bản rút gọn" là cách nhanh nhất để người ta không bấm vào.
    #[test]
    fn cau_nhan_giai_thich_chu_khong_ha_thap_ban_dong_lenh() {
        assert!(LOI_NHAN.contains("chính thức"));
        assert!(LOI_NHAN.contains("không phải bản rút gọn"));
        assert!(!LOI_NHAN.contains("xin lỗi"));
    }

    /// Phép dò không được **bớt** thứ gì — chỉ thêm một lối đi. Bốn ngã của
    /// `do_hien_tai` phải luôn nằm trong ba trạng thái đã định nghĩa.
    #[test]
    fn do_hien_tai_khong_hoang() {
        let d = DuongLui::do_hien_tai();
        assert!(matches!(
            d,
            DuongLui::Khong | DuongLui::Co(_) | DuongLui::CoNhungThieuTep
        ));
    }
}
