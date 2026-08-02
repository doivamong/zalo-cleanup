//! Bốn chế độ quét và trạng thái kết quả quét.
//!
//! Trạng thái này là trung tâm của nguyên tắc bất biến 1 và 2: không quét thì
//! không xóa được, và đổi bộ lọc là kết quả cũ bị hủy.
//!
//! Khử trùng lặp kết luận **chỉ bằng SHA-256 toàn tệp**, không bao giờ bằng tên
//! tệp — hai nơi lưu đặt tên theo hai quy ước khác nhau. Mốc **M2**.

/// Phần mở rộng của một tên tệp, **theo đúng luật .NET** `Path.GetExtension`.
///
/// # Đây là một bẫy thật, không phải chuyện câu chữ
///
/// `Path::extension()` của Rust và `Path.GetExtension` của .NET **hiểu khác
/// nhau** ở tệp bắt đầu bằng dấu chấm:
///
/// | Tên tệp | .NET | Rust `Path::extension` |
/// |---|---|---|
/// | `.rescache` | `".rescache"` | `None` |
/// | `video.jxl` | `".jxl"` | `Some("jxl")` |
/// | `a.` | `""` | `Some("")` |
///
/// Dữ liệu Zalo thật có **4.226 tệp `.rescache`**, và công cụ dùng đúng phần mở
/// rộng đó để loại chúng khỏi lượt quét. Dùng thẳng `Path::extension()` là
/// 4.226 tệp bị phân loại sai — không phải một khác biệt lý thuyết.
///
/// Luật .NET: tìm dấu chấm cuối cùng trong TÊN TỆP; không có thì trả rỗng; nó
/// là ký tự cuối thì cũng trả rỗng; còn lại trả từ dấu chấm đó tới hết.
pub fn duoi_kieu_dotnet(ten_tep: &str) -> &str {
    match ten_tep.rfind('.') {
        None => "",
        Some(i) if i + 1 == ten_tep.len() => "",
        Some(i) => &ten_tep[i..],
    }
}

/// Tên tệp cuối đường dẫn. Cắt bằng `rfind` để không hoảng với đường dẫn dị dạng.
pub fn ten_tep(duong_dan: &str) -> &str {
    match duong_dan.rfind('\\') {
        Some(i) => &duong_dan[i + 1..],
        None => duong_dan,
    }
}

/// Đường dẫn tương đối so với gốc — tương ứng `Get-RelPath`.
///
/// Không nằm dưới gốc thì trả về **nguyên đường dẫn tuyệt đối**, y như bản
/// PowerShell. Đó là hành vi đã từng suýt gây họa ở `Invoke-Backup`, nên người
/// gọi phải tự lo, chứ hàm này không được tự ý đổi cho "an toàn hơn".
pub fn duong_dan_tuong_doi<'a>(day_du: &'a str, goc: &str) -> &'a str {
    if goc.trim().is_empty() {
        return day_du;
    }
    let mut b = goc.to_string();
    if !b.ends_with('\\') {
        b.push('\\');
    }
    if day_du.len() >= b.len() && day_du[..b.len()].eq_ignore_ascii_case(&b) {
        &day_du[b.len()..]
    } else {
        day_du
    }
}

/// Bộ lọc của một lượt quét — tương ứng `Test-PassFilterUnguarded`.
///
/// **KHÔNG kiểm vùng bảo vệ.** Người gọi có bổn phận tự chạy phép kiểm ấy
/// TRƯỚC, đúng như bên bản PowerShell — nơi tên hàm mang hẳn chữ `Unguarded`
/// để chỗ gọi tự nói ra điều đó.
#[derive(Debug, Default, Clone)]
pub struct BoLoc {
    /// Giữ lại `.rescache`, tức LOẠI chúng khỏi kết quả quét.
    pub giu_rescache: bool,
    /// Chỉ nhận các đuôi này. Rỗng nghĩa là nhận tất cả. Chữ thường.
    pub duoi_nhan: Vec<String>,
    /// Loại các đuôi này. Chữ thường.
    pub duoi_loai: Vec<String>,
    /// Loại các thư mục cấp một này, so theo tên.
    pub thu_muc_loai: Vec<String>,
    /// Gốc quét, để tính thư mục cấp một.
    pub goc: String,
}

/// Nhãn dành cho tệp không có phần mở rộng. Phải khớp từng ký tự với bản
/// PowerShell vì người dùng chọn đuôi bằng đúng chuỗi này.
pub const KHONG_DUOI: &str = "(không đuôi)";

/// Nhãn dành cho tệp nằm ngay ở gốc quét, không thuộc thư mục con nào.
pub const O_GOC: &str = "(gốc)";

fn chua_khong_phan_biet_hoa_thuong(ds: &[String], x: &str) -> bool {
    ds.iter().any(|c| c.eq_ignore_ascii_case(x))
}

impl BoLoc {
    /// Tệp này có qua bộ lọc không.
    pub fn qua(&self, duong_dan: &str) -> bool {
        let duoi_goc = duoi_kieu_dotnet(ten_tep(duong_dan));

        if self.giu_rescache && duoi_goc.eq_ignore_ascii_case(".rescache") {
            return false;
        }

        let duoi = if duoi_goc.is_empty() {
            KHONG_DUOI.to_string()
        } else {
            duoi_goc.to_lowercase()
        };

        if !self.duoi_nhan.is_empty() && !chua_khong_phan_biet_hoa_thuong(&self.duoi_nhan, &duoi) {
            return false;
        }
        if !self.duoi_loai.is_empty() && chua_khong_phan_biet_hoa_thuong(&self.duoi_loai, &duoi) {
            return false;
        }
        if !self.thu_muc_loai.is_empty() {
            let rel = duong_dan_tuong_doi(duong_dan, &self.goc);
            let dau = match rel.find('\\') {
                Some(i) => &rel[..i],
                None => O_GOC,
            };
            if chua_khong_phan_biet_hoa_thuong(&self.thu_muc_loai, dau) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duoi_theo_luat_dotnet_chu_khong_theo_luat_rust() {
        // Bảng này đã đối chiếu tận nơi với [IO.Path]::GetExtension của .NET.
        assert_eq!(duoi_kieu_dotnet(".rescache"), ".rescache");
        assert_eq!(duoi_kieu_dotnet(".gitignore"), ".gitignore");
        assert_eq!(duoi_kieu_dotnet("video.jxl"), ".jxl");
        assert_eq!(duoi_kieu_dotnet("a.b.c"), ".c");
        assert_eq!(duoi_kieu_dotnet("a."), "");
        assert_eq!(duoi_kieu_dotnet("a"), "");
        assert_eq!(duoi_kieu_dotnet("7594809871497"), "");
        assert_eq!(duoi_kieu_dotnet(""), "");
    }

    #[test]
    fn rescache_bi_loai_dung_ca_4226_tep_kieu_do() {
        let b = BoLoc {
            giu_rescache: true,
            ..Default::default()
        };
        assert!(!b.qua(r"C:\z\resource\.rescache"));
        assert!(!b.qua(r"C:\z\resource\.RESCACHE"));
        assert!(b.qua(r"C:\z\video\7594809871497"));
    }

    #[test]
    fn khong_giu_rescache_thi_no_qua_binh_thuong() {
        let b = BoLoc::default();
        assert!(b.qua(r"C:\z\resource\.rescache"));
    }

    #[test]
    fn loc_theo_duoi_nhan_va_duoi_loai() {
        let b = BoLoc {
            duoi_nhan: vec![".jxl".into()],
            ..Default::default()
        };
        assert!(b.qua(r"C:\z\a.jxl"));
        assert!(b.qua(r"C:\z\a.JXL"), "so đuôi không phân biệt hoa thường");
        assert!(!b.qua(r"C:\z\a.jpg"));

        let b = BoLoc {
            duoi_loai: vec![".jpg".into()],
            ..Default::default()
        };
        assert!(b.qua(r"C:\z\a.jxl"));
        assert!(!b.qua(r"C:\z\a.jpg"));
    }

    #[test]
    fn tep_khong_duoi_mang_nhan_rieng() {
        let b = BoLoc {
            duoi_nhan: vec![KHONG_DUOI.into()],
            ..Default::default()
        };
        assert!(b.qua(r"C:\z\video\7594809871497"));
        assert!(!b.qua(r"C:\z\video\a.jxl"));
    }

    #[test]
    fn loai_thu_muc_cap_mot() {
        let b = BoLoc {
            thu_muc_loai: vec!["resource".into()],
            goc: r"C:\z".into(),
            ..Default::default()
        };
        assert!(!b.qua(r"C:\z\resource\c1\a.jxl"));
        assert!(b.qua(r"C:\z\video\a.jxl"));
    }

    #[test]
    fn tep_ngay_o_goc_mang_nhan_goc() {
        let b = BoLoc {
            thu_muc_loai: vec![O_GOC.into()],
            goc: r"C:\z".into(),
            ..Default::default()
        };
        assert!(!b.qua(r"C:\z\ngay_o_goc.bin"));
        assert!(b.qua(r"C:\z\video\a.bin"));
    }

    #[test]
    fn duong_dan_tuong_doi_tra_nguyen_khi_khong_nam_duoi_goc() {
        assert_eq!(duong_dan_tuong_doi(r"C:\z\a\b", r"C:\z"), r"a\b");
        assert_eq!(duong_dan_tuong_doi(r"C:\z\a\b", r"C:\z\"), r"a\b");
        assert_eq!(duong_dan_tuong_doi(r"C:\Z\A\b", r"c:\z"), r"A\b");
        assert_eq!(duong_dan_tuong_doi(r"D:\khac\b", r"C:\z"), r"D:\khac\b");
        assert_eq!(duong_dan_tuong_doi(r"C:\z\a", ""), r"C:\z\a");
    }

    #[test]
    fn ten_tep_khong_hoang_voi_duong_dan_di_dang() {
        assert_eq!(ten_tep(r"C:\a\b.txt"), "b.txt");
        assert_eq!(ten_tep("khong_co_gach_cheo"), "khong_co_gach_cheo");
        assert_eq!(ten_tep(""), "");
        assert_eq!(ten_tep("\\"), "");
    }
}
