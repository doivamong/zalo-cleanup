//! Chốt xem trước, và nhận dạng loại tệp bằng **magic byte**.
//!
//! # Chốt xem trước là gì
//!
//! Không mở danh sách tệp sắp mất thì **nút xóa không bật**. Đây là ma sát mạnh
//! nhất của cả giao diện: người ta bấm bừa qua một hộp cảnh báo dễ hơn nhiều so
//! với bấm bừa qua một danh sách ảnh của chính mình.
//!
//! Nó là một luật trạng thái, nên nó nằm ở đây chứ không nằm trong mã vẽ — mã
//! vẽ thì không có phép thử nào canh được.
//!
//! # Vì sao ngửi magic byte chứ không tin phần mở rộng
//!
//! Đo trên 57.035 tệp dữ liệu Zalo thật:
//!
//! | Loại | Tỷ lệ |
//! |---|---|
//! | `.jxl` | 46,4% |
//! | **không có phần mở rộng** | 43,7% |
//! | `.jpg` + `.png` | **2,5%** |
//!
//! Ngửi 400 mẫu trong nhóm không có phần mở rộng: **88,5% là JPEG**. Tin phần
//! mở rộng thì xem trước được 2,5% số tệp; ngửi magic byte thì được 41%.
//!
//! Xem `docs/quyet-dinh.md` §Q10 — đây là chỗ số đo lật ngược kết luận ban đầu
//! của hội đồng.

/// Loại tệp nhận ra được từ mấy byte đầu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaiTep {
    Jpeg,
    Png,
    /// JPEG XL, cả hai dạng vỏ: codestream trần và hộp ISOBMFF.
    JpegXl,
    Mp4,
    /// Không nhận ra. Ô xem trước hiện dấu hỏi — nhưng tệp **vẫn nằm trong danh
    /// sách**, không bao giờ bị giấu đi vì không xem trước được.
    KhongRo,
}

impl LoaiTep {
    /// Bản này có vẽ được ảnh xem trước không.
    pub fn xem_truoc_duoc(self) -> bool {
        matches!(self, LoaiTep::Jpeg | LoaiTep::Png)
    }
}

/// Nhận dạng bằng mấy byte đầu tệp.
pub fn ngui(dau: &[u8]) -> LoaiTep {
    if dau.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return LoaiTep::Jpeg;
    }
    if dau.starts_with(b"\x89PNG\r\n\x1a\n") {
        return LoaiTep::Png;
    }
    // JPEG XL: codestream trần bắt đầu bằng FF 0A; dạng hộp bắt đầu bằng
    // 00 00 00 0C 4A 58 4C 20 0D 0A 87 0A.
    if dau.starts_with(&[0xFF, 0x0A]) {
        return LoaiTep::JpegXl;
    }
    if dau.len() >= 12
        && dau[..12]
            == [
                0, 0, 0, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
            ]
    {
        return LoaiTep::JpegXl;
    }
    // MP4/MOV: `ftyp` ở byte thứ 4.
    if dau.len() >= 8 && &dau[4..8] == b"ftyp" {
        return LoaiTep::Mp4;
    }
    LoaiTep::KhongRo
}

/// Đọc mấy byte đầu rồi ngửi. Đọc **8 KB** là đủ cho mọi chữ ký ở trên.
pub fn ngui_tep(duong_dan: &std::path::Path) -> LoaiTep {
    use std::io::Read;
    let mut f = match std::fs::File::open(duong_dan) {
        Ok(f) => f,
        Err(_) => return LoaiTep::KhongRo,
    };
    let mut d = [0u8; 32];
    let n = f.read(&mut d).unwrap_or(0);
    ngui(&d[..n])
}

/// Chốt xem trước của một kết quả quét.
///
/// Nút xóa **chỉ bật** khi người dùng đã thật sự mở danh sách. Đóng danh sách
/// rồi thì vẫn coi là đã xem — cái cần là họ **đã nhìn thấy**, không phải là
/// danh sách đang mở.
#[derive(Debug, Default, Clone)]
pub struct ChotXemTruoc {
    da_xem: bool,
}

impl ChotXemTruoc {
    pub fn moi() -> Self {
        ChotXemTruoc { da_xem: false }
    }
    pub fn danh_dau_da_xem(&mut self) {
        self.da_xem = true;
    }
    pub fn da_xem(&self) -> bool {
        self.da_xem
    }
    /// Được phép mở trang xác nhận xóa chưa.
    pub fn cho_sang_trang_xac_nhan(&self) -> bool {
        self.da_xem
    }
    /// Lý do nút đang tắt, cho trình đọc màn hình (`ĐM-06`).
    pub fn ly_do_tat(&self) -> Option<&'static str> {
        if self.da_xem {
            None
        } else {
            Some("cần xem danh sách tệp sắp mất trước")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chốt xem trước phải chặn từ đầu, và chỉ mở sau khi người dùng đã nhìn.
    #[test]
    fn chua_xem_thi_khong_sang_duoc_trang_xac_nhan() {
        let mut c = ChotXemTruoc::moi();
        assert!(!c.cho_sang_trang_xac_nhan());
        assert_eq!(c.ly_do_tat(), Some("cần xem danh sách tệp sắp mất trước"));
        c.danh_dau_da_xem();
        assert!(c.cho_sang_trang_xac_nhan());
        assert_eq!(c.ly_do_tat(), None);
    }

    /// Kết quả quét mới thì chốt phải đóng lại. Giữ trạng thái "đã xem" của lượt
    /// quét TRƯỚC là cho phép xóa một danh sách chưa ai nhìn qua.
    #[test]
    fn ket_qua_quet_moi_thi_chot_dong_lai() {
        let mut c = ChotXemTruoc::moi();
        c.danh_dau_da_xem();
        assert!(c.cho_sang_trang_xac_nhan());
        c = ChotXemTruoc::moi();
        assert!(
            !c.cho_sang_trang_xac_nhan(),
            "chốt vẫn mở sau khi quét lại — sẽ xóa được danh sách chưa ai nhìn"
        );
    }

    #[test]
    fn ngui_dung_cac_chu_ky_that() {
        assert_eq!(ngui(&[0xFF, 0xD8, 0xFF, 0xE0]), LoaiTep::Jpeg);
        assert_eq!(ngui(b"\x89PNG\r\n\x1a\nrac"), LoaiTep::Png);
        assert_eq!(ngui(&[0xFF, 0x0A, 0x00]), LoaiTep::JpegXl);
        assert_eq!(
            ngui(&[0, 0, 0, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A]),
            LoaiTep::JpegXl
        );
        assert_eq!(ngui(b"\0\0\0\x18ftypmp42"), LoaiTep::Mp4);
        assert_eq!(ngui(b"khong phai anh"), LoaiTep::KhongRo);
        assert_eq!(ngui(&[]), LoaiTep::KhongRo, "tệp rỗng không được làm nổ");
        assert_eq!(ngui(&[0xFF]), LoaiTep::KhongRo, "một byte cũng không nổ");
    }

    /// Tệp **không có phần mở rộng** vẫn phải nhận ra được — 43,7% dữ liệu Zalo
    /// thật nằm ở nhóm này, và 88,5% trong đó là JPEG.
    #[test]
    fn nhan_ra_duoc_tep_khong_co_phan_mo_rong() {
        let d = std::env::temp_dir().join(format!("zngui_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("7594809871497");
        let mut b = vec![0xFF, 0xD8, 0xFF, 0xE0];
        b.extend_from_slice(&[0u8; 100]);
        std::fs::write(&p, &b).unwrap();
        assert_eq!(ngui_tep(&p), LoaiTep::Jpeg);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Tệp không đọc được ra `KhongRo`, **không** làm nổ chương trình.
    #[test]
    fn tep_khong_doc_duoc_ra_khong_ro_chu_khong_no() {
        assert_eq!(
            ngui_tep(std::path::Path::new(r"C:\khong_he_co_tep_nay_8172")),
            LoaiTep::KhongRo
        );
    }

    /// Bản này chưa có bộ giải mã JPEG XL. Phép thử ghim sự thật ấy lại thay vì
    /// để nó lặng lẽ trôi: `.jxl` chiếm **46,4%** dữ liệu Zalo thật, nên nửa số
    /// tệp hiện ô `?` chứ không có ảnh xem trước. Ngày nào thêm bộ giải mã thì
    /// phép thử này đỏ và người sửa buộc phải đọc lại con số ấy.
    #[test]
    fn jpeg_xl_nhan_ra_duoc_nhung_chua_ve_duoc() {
        assert_eq!(ngui(&[0xFF, 0x0A]), LoaiTep::JpegXl);
        assert!(
            !LoaiTep::JpegXl.xem_truoc_duoc(),
            "đã vẽ được JXL — cập nhật tài liệu M5, ma sát xem trước vừa mạnh lên đáng kể"
        );
        assert!(LoaiTep::Jpeg.xem_truoc_duoc());
        assert!(LoaiTep::Png.xem_truoc_duoc());
        assert!(!LoaiTep::Mp4.xem_truoc_duoc());
        assert!(!LoaiTep::KhongRo.xem_truoc_duoc());
    }
}
