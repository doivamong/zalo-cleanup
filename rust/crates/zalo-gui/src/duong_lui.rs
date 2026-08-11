//! `ĐM-08` — đường lui sang bản dòng lệnh.
//!
//! # Bản dòng lệnh là đường tiếp cận CHÍNH THỨC, không phải tác dụng phụ
//!
//! Hội đồng ghi thẳng điều đó. Console của Windows phơi văn bản ra UIA rất tốt,
//! còn egui với AccessKit thì mới là **nền móng**: chưa có live region, chưa có
//! hộp thoại gốc, bảng không phơi quan hệ hàng–cột, đồ họa vẽ tay thì vô hình.
//!
//! # Vì sao BỎ HẲN phép dò — `Q15`, phương án C
//!
//! Bản đầu chỉ mở đường lui **khi dò thấy** trình đọc màn hình, qua cờ
//! `SPI_GETSCREENREADER`. Đem đo thì phép dò ấy hỏng ở đúng ca quan trọng nhất:
//!
//! | | |
//! |---|---|
//! | Narrator chạy liên tục 20 giây, không nâng quyền | cờ vẫn **`False`** |
//! | `UiaClientsAreListening()` | **`True`** kể cả khi không có gì chạy |
//! | `HKLM\…\Accessibility\Configuration` | Windows **không ghi** |
//!
//! Nghĩa là người dùng Narrator — trình đọc màn hình có sẵn của chính Windows,
//! và là thứ người ta gặp đầu tiên khi chưa cài gì — **không bao giờ thấy đường
//! lui**. Một mục cổng mức 1 chỉ chạy đúng cho một phần người dùng.
//!
//! Nên đường lui giờ **luôn có mặt**, không hỏi han gì. Không còn phép dò thì
//! không còn ca dò trượt.
//!
//! # Vì sao làm được điều đó bây giờ mà trước thì không
//!
//! `RB-07` cấm ship nút này **chừng nào chưa có khóa một tiến trình một lúc** —
//! không phải cấm trên nguyên tắc. Không có khóa thì cái nút là một cỗ máy đẻ
//! tiến trình, và hai bản cùng xóa trên một tập tệp là mối đe dọa `B8`, hội
//! đồng xếp **NẶNG**. `Q7` chốt thứ tự và ghi rõ *"thứ tự này là bắt buộc"*:
//! khóa trước, nút sau.
//!
//! Khóa ấy vừa được làm ở [`zalo_core::lock`]. Nên điều kiện của `RB-07` đã
//! thỏa, và `Q7` nói chính xác rằng lúc này nút **nên** ship.
//!
//! # Cuộc BÀN GIAO, không phải sinh sản
//!
//! `RB-08`: nhả khóa → khởi chạy console → **tự thoát**. Thiếu vế nhả khóa thì
//! bản dòng lệnh mở lên, thấy khóa còn người giữ, rồi từ chối chạy — tức cái
//! nút giao cho người dùng một công cụ không mở được.

/// Tên tệp bản dòng lệnh, tìm ngay cạnh tệp thực thi đồ họa.
pub const TEN_BAN_DONG_LENH: &str = "zalo-cli.exe";

/// Câu nói ra khi thấy trình đọc màn hình. Là câu **giải thích**, không phải
/// lời xin lỗi: người dùng cần biết vì sao nên chọn bản kia, chứ không cần biết
/// rằng công cụ này thấy tiếc.
///
/// Không còn nói "phát hiện trình đọc màn hình" nữa: sau `Q15` thì chẳng còn
/// phép dò nào, và một câu nói mình vừa phát hiện ra điều gì đó trong khi không
/// hề dò là một câu nói dối.
pub const LOI_NHAN: &str = "Dùng trình đọc màn hình, hay chỉ dùng bàn phím? Bản dòng lệnh của \
công cụ này đọc màn hình tốt hơn hẳn — nó là đường tiếp cận chính thức, không phải bản rút gọn.";

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
    /// Có bản dòng lệnh nằm cạnh tệp thực thi.
    Co(std::path::PathBuf),
    /// **Không tìm ra** bản dòng lệnh cạnh bên.
    ///
    /// Vẫn phải hiện, và nói thật là không mở hộ được. Im lặng ở đây là để người
    /// dùng ngồi trước một giao diện họ đọc không nổi mà không biết có đường
    /// khác.
    CoNhungThieuTep,
}

impl DuongLui {
    /// Không còn dò gì nữa — chỉ hỏi bản dòng lệnh có nằm cạnh không.
    ///
    /// Xem chú thích đầu tệp: phép dò cũ bỏ sót Narrator, và bỏ sót đúng nhóm
    /// người mà `ĐM-08` sinh ra để phục vụ.
    pub fn do_hien_tai() -> Self {
        match tim_ban_dong_lenh() {
            Some(p) => DuongLui::Co(p),
            None => DuongLui::CoNhungThieuTep,
        }
    }

    /// Luôn hiện. Giữ lại hàm này để chỗ gọi khỏi phải đoán, và để ngày nào có
    /// người định dựng lại một phép dò thì họ phải sửa ở đây chứ không rải rác.
    pub fn nen_hien(&self) -> bool {
        true
    }

    /// Câu hiện trên dải thông báo.
    pub fn cau(&self) -> String {
        match self {
            DuongLui::Co(_) => LOI_NHAN.to_string(),
            DuongLui::CoNhungThieuTep => format!(
                "{LOI_NHAN} Không tìm thấy {TEN_BAN_DONG_LENH} cạnh tệp này, \
                 nên phải mở tay."
            ),
        }
    }

    /// Bàn giao sang bản dòng lệnh: **nhả khóa → khởi chạy → gọi bên ngoài tự
    /// thoát** (`RB-08`).
    ///
    /// Khóa phải nhả **trước** khi khởi chạy, không phải sau. Bản dòng lệnh xin
    /// khóa ngay lúc khởi động; còn người giữ thì nó in "đã có một bản đang mở"
    /// rồi thoát, và người dùng nhận được một cửa sổ console chớp lên rồi tắt.
    ///
    /// Trả `true` nghĩa là đã khởi chạy được — chỗ gọi phải đóng cửa sổ theo.
    pub fn ban_giao(&self, khoa: &mut Option<zalo_core::lock::Khoa>) -> bool {
        let p = match self {
            DuongLui::Co(p) => p.clone(),
            DuongLui::CoNhungThieuTep => return false,
        };
        nha_roi_chay(khoa, || {
            std::process::Command::new("cmd")
                .args(["/c", "start", "", &p.to_string_lossy()])
                .spawn()
                .is_ok()
        })
    }
}

/// Nhả khóa **rồi mới** chạy. Thứ tự hai bước này là cả nội dung của `RB-08`.
///
/// Tách ra khỏi [`DuongLui::ban_giao`] để phép thử hỏi được đúng câu *"lúc gọi
/// tới bước chạy thì khóa đã nhả chưa"* — mà **không phải đẻ ra tiến trình
/// nào**. Bản đầu của phép thử ấy gọi thẳng `ban_giao` với một đường dẫn giả;
/// `cmd /c start` vẫn chạy thật, in lỗi ra, và làm nghẽn cả lượt chạy cổng.
/// Một phép thử đơn vị đẻ tiến trình là một phép thử sẽ treo ở đâu đó.
fn nha_roi_chay(khoa: &mut Option<zalo_core::lock::Khoa>, chay: impl FnOnce() -> bool) -> bool {
    if let Some(k) = khoa {
        k.nha();
    }
    *khoa = None;
    chay()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **`Q15` phương án C.** Đường lui luôn có mặt, không hỏi han gì.
    ///
    /// Phép thử này thay cho phép thử cũ `khong_thay_trinh_doc_thi_khong_hien_gi`.
    /// Nó bị bỏ vì tiền đề của nó — "dò được thì mới hiện" — chính là chỗ hỏng:
    /// Narrator chạy 20 giây mà cờ `SPI_GETSCREENREADER` không hề lên, nên "chỉ
    /// hiện khi dò thấy" nghĩa là **không bao giờ hiện** với người dùng Narrator.
    #[test]
    fn duong_lui_luon_co_mat_khong_can_do_gi() {
        for d in [
            DuongLui::Co(std::path::PathBuf::from(r"C:\x\zalo-cli.exe")),
            DuongLui::CoNhungThieuTep,
        ] {
            assert!(d.nen_hien(), "{d:?} phải luôn hiện");
            assert!(!d.cau().is_empty());
        }
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
        let mut khong_khoa = None;
        assert!(
            !d.ban_giao(&mut khong_khoa),
            "không có tệp mà vẫn báo bàn giao được"
        );
    }

    /// **`RB-08`.** Bàn giao phải **nhả khóa** rồi mới khởi chạy.
    ///
    /// Không nhả trước thì bản dòng lệnh mở lên, xin khóa, thấy còn người giữ,
    /// in "đã có một bản đang mở" rồi thoát — người dùng nhận được một cửa sổ
    /// console chớp lên rồi tắt, và kết luận nút ấy hỏng.
    #[test]
    #[cfg(windows)]
    fn ban_giao_nha_khoa_truoc_khi_khoi_chay() {
        let mut khoa = match zalo_core::lock::vao("phep-thu-ban-giao") {
            zalo_core::lock::KetQuaKhoa::DiTiep(k) => Some(k),
            zalo_core::lock::KetQuaKhoa::BanKhacDangMo(_) => return,
        };
        // Bước "chạy" là một closure chỉ ghi lại **khóa đã nhả chưa vào đúng
        // lúc nó được gọi**. Không đẻ tiến trình nào, mà vẫn hỏi được đúng câu
        // mà `RB-08` quan tâm.
        let mut da_nha_luc_chay = None;
        let r = nha_roi_chay(&mut khoa, || {
            da_nha_luc_chay = Some(true);
            true
        });
        assert!(r);
        assert_eq!(
            da_nha_luc_chay,
            Some(true),
            "bước chạy không hề được gọi — bàn giao đứt giữa chừng"
        );
        assert!(
            khoa.is_none(),
            "khóa chưa nhả lúc bước chạy được gọi — bản dòng lệnh sẽ bị chính ta chặn"
        );
    }

    /// Ngã **thiếu tệp** không được nhả khóa: chưa bàn giao được cho ai thì
    /// đừng buông thứ mình đang giữ.
    #[test]
    #[cfg(windows)]
    fn thieu_tep_thi_khong_nha_khoa() {
        let mut khoa = match zalo_core::lock::vao("phep-thu-thieu-tep") {
            zalo_core::lock::KetQuaKhoa::DiTiep(k) => Some(k),
            zalo_core::lock::KetQuaKhoa::BanKhacDangMo(_) => return,
        };
        assert!(!DuongLui::CoNhungThieuTep.ban_giao(&mut khoa));
        assert!(khoa.is_some(), "không bàn giao được cho ai mà đã nhả khóa");
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

    /// `do_hien_tai` không bao giờ hoảng, và **không bao giờ trả về "không có"**.
    #[test]
    fn do_hien_tai_khong_hoang_va_luon_hien() {
        let d = DuongLui::do_hien_tai();
        assert!(matches!(d, DuongLui::Co(_) | DuongLui::CoNhungThieuTep));
        assert!(d.nen_hien());
    }
}
