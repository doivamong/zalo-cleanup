//! Ảnh xem trước — **ma sát mạnh nhất của cả giao diện**.
//!
//! Người ta bấm bừa qua một hộp cảnh báo dễ hơn nhiều so với bấm bừa qua ảnh
//! của chính mình. Đó là toàn bộ lý do mô-đun này tồn tại.
//!
//! # Ba luật, và cả ba đều đến từ số đo
//!
//! **① Nhận dạng bằng magic byte, không bằng phần mở rộng.** 43,7% tệp Zalo
//! không có phần mở rộng, mà 88,5% trong số đó là JPEG.
//!
//! **② Phải có bộ giải mã JPEG XL.** `.jxl` chiếm **46,4%** dữ liệu thật. Thiếu
//! nó là ma sát hỏng với gần một nửa số tệp.
//!
//! **③ Không đọc được thì hiện ô `?`, KHÔNG BAO GIỜ giấu tệp khỏi danh sách.**
//! Giấu đi là người dùng xóa một thứ họ chưa từng nhìn thấy, mà lại tưởng mình
//! đã xem hết.
//!
//! # Vì sao chỉ mười hai ảnh
//!
//! Hội đồng chốt mười hai, lấy ngẫu nhiên, kèm một dòng nói thẳng tỷ lệ mẫu
//! (`RB-43`). Mười hai ảnh không nói được gì về mười hai nghìn tệp còn lại —
//! nhưng chúng đủ để người dùng nhận ra "đây là ảnh cưới của tôi" và dừng tay.
//!
//! Giải mã chạy **ngoài luồng vẽ** (`RB-129`), đọc tối đa 8 MB đầu mỗi tệp, và
//! thu ảnh về cạnh dài 128 px trước khi đưa sang giao diện.

use crate::xem_truoc::{ngui, LoaiTep};
use std::path::{Path, PathBuf};

/// Số ảnh lấy mẫu. Hội đồng chốt.
pub const SO_ANH: usize = 12;
/// Cạnh dài của ảnh thu nhỏ, tính bằng điểm ảnh.
pub const CANH: u32 = 128;
/// Đọc tối đa từng này byte đầu mỗi tệp.
pub const DOC_TOI_DA: usize = 8 * 1024 * 1024;

/// Một ô trong lưới xem trước.
pub struct O {
    pub duong_dan: PathBuf,
    pub loai: LoaiTep,
    /// `None` nghĩa là **không giải mã được** — ô hiện dấu hỏi. Tệp vẫn nằm
    /// trong danh sách.
    pub anh: Option<AnhNho>,
}

/// Ảnh đã thu nhỏ, dạng RGBA thẳng để đẩy vào egui.
pub struct AnhNho {
    pub rong: usize,
    pub cao: usize,
    pub diem: Vec<u8>,
}

/// Bộ sinh số giả ngẫu nhiên nhỏ, **chỉ để lấy mẫu**.
///
/// Lấy mười hai tệp đầu danh sách thì mẫu luôn rơi vào cùng một thư mục, và
/// người dùng nhìn thấy đúng một loại ảnh dù lượt quét chạm nhiều nơi.
struct Xorshift(u64);

impl Xorshift {
    fn moi(hat: u64) -> Self {
        Xorshift(if hat == 0 { 0x9E3779B97F4A7C15 } else { hat })
    }
    fn tiep(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// Chọn tối đa [`SO_ANH`] chỉ số ngẫu nhiên trong `0..n`.
///
/// `hat` do người gọi đưa vào để phép thử tái lập được — không tự lấy đồng hồ
/// bên trong.
pub fn chon_mau(n: usize, hat: u64) -> Vec<usize> {
    let mut v: Vec<usize> = (0..n).collect();
    if n <= SO_ANH {
        return v;
    }
    let mut r = Xorshift::moi(hat);
    // Xáo Fisher–Yates một phần: chỉ cần SO_ANH phần tử đầu.
    for i in 0..SO_ANH {
        let j = i + (r.tiep() % (n - i) as u64) as usize;
        v.swap(i, j);
    }
    v.truncate(SO_ANH);
    v.sort_unstable();
    v
}

/// Đọc tối đa [`DOC_TOI_DA`] byte đầu tệp.
fn doc_dau(p: &Path) -> Option<Vec<u8>> {
    use std::io::Read;
    let f = std::fs::File::open(p).ok()?;
    let mut b = Vec::new();
    f.take(DOC_TOI_DA as u64).read_to_end(&mut b).ok()?;
    Some(b)
}

/// Thu một ảnh RGBA về cạnh dài [`CANH`] bằng phép lấy mẫu gần nhất.
///
/// Cố ý không dùng bộ lọc mượt: đây là ảnh 128 px để người ta **nhận ra** tấm
/// ảnh của mình, không phải để ngắm. Lấy mẫu gần nhất nhanh hơn nhiều lần, mà
/// mười hai ảnh phải xong trước khi người dùng kịp sốt ruột.
fn thu_nho(rong: usize, cao: usize, diem: &[u8]) -> AnhNho {
    if rong == 0 || cao == 0 {
        return AnhNho {
            rong: 0,
            cao: 0,
            diem: Vec::new(),
        };
    }
    let ty = (CANH as f32 / rong.max(cao) as f32).min(1.0);
    let (rn, cn) = (
        ((rong as f32 * ty) as usize).max(1),
        ((cao as f32 * ty) as usize).max(1),
    );
    let mut ra = vec![0u8; rn * cn * 4];
    for y in 0..cn {
        let sy = y * cao / cn;
        for x in 0..rn {
            let sx = x * rong / rn;
            let s = (sy * rong + sx) * 4;
            let d = (y * rn + x) * 4;
            if s + 3 < diem.len() {
                ra[d..d + 4].copy_from_slice(&diem[s..s + 4]);
            }
        }
    }
    AnhNho {
        rong: rn,
        cao: cn,
        diem: ra,
    }
}

/// Giải mã một tệp thành ảnh thu nhỏ. `None` nghĩa là không đọc được.
pub fn giai_ma(p: &Path) -> (LoaiTep, Option<AnhNho>) {
    let b = match doc_dau(p) {
        Some(b) => b,
        None => return (LoaiTep::KhongRo, None),
    };
    let loai = ngui(&b);
    let anh = match loai {
        LoaiTep::Jpeg | LoaiTep::Png => giai_ma_thuong(&b),
        LoaiTep::JpegXl => giai_ma_jxl(&b),
        _ => None,
    };
    (loai, anh)
}

fn giai_ma_thuong(b: &[u8]) -> Option<AnhNho> {
    let img = image::load_from_memory(b).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    Some(thu_nho(w, h, rgba.as_raw()))
}

/// Giải mã JPEG XL.
///
/// `jxl-oxide` trả về kênh dạng `f32` trong khoảng 0..1, và số kênh thay đổi
/// theo ảnh — xám một kênh, xám kèm alpha hai, màu ba, màu kèm alpha bốn. Phải
/// xét cả bốn ca: đoán bừa là ba kênh thì ảnh xám ra một mớ nhiễu, mà ảnh xám
/// thì Zalo có thật.
fn giai_ma_jxl(b: &[u8]) -> Option<AnhNho> {
    let img = jxl_oxide::JxlImage::builder()
        .read(std::io::Cursor::new(b))
        .ok()?;
    let khung = img.render_frame(0).ok()?;
    let fb = khung.image_all_channels();
    let (w, h, kenh) = (fb.width(), fb.height(), fb.channels());
    let buf = fb.buf();
    let mut rgba = vec![0u8; w * h * 4];
    let u8_tu = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    for i in 0..w * h {
        let s = i * kenh;
        if s + kenh > buf.len() {
            break;
        }
        let (r, g, bl, a) = match kenh {
            1 => (buf[s], buf[s], buf[s], 1.0),
            2 => (buf[s], buf[s], buf[s], buf[s + 1]),
            3 => (buf[s], buf[s + 1], buf[s + 2], 1.0),
            _ => (buf[s], buf[s + 1], buf[s + 2], buf[s + 3]),
        };
        let d = i * 4;
        rgba[d] = u8_tu(r);
        rgba[d + 1] = u8_tu(g);
        rgba[d + 2] = u8_tu(bl);
        rgba[d + 3] = u8_tu(a);
    }
    Some(thu_nho(w, h, &rgba))
}

/// Dòng nói thẳng tỷ lệ mẫu — `RB-43`.
///
/// Không được nhỏ hơn chữ thường và không được xám. Mười hai ảnh trông như một
/// bằng chứng đầy đủ nếu không có câu này.
pub fn dong_ty_le_mau(so_anh: usize, tong: usize) -> String {
    if tong == 0 {
        return String::new();
    }
    // Mẫu phủ hết thì câu cảnh báo đổi hẳn nghĩa. Bản đầu in ra "không nói được
    // gì về 0 tệp còn lại" — một câu vô nghĩa, và một câu vô nghĩa ở chỗ cảnh
    // báo dạy người đọc bỏ qua cả những câu có nghĩa.
    if so_anh >= tong {
        return format!("Đang xem cả {tong} tệp của lượt quét này.");
    }
    let pt = so_anh as f64 * 100.0 / tong as f64;
    format!(
        "{so_anh} ảnh lấy ngẫu nhiên trong {tong} tệp ({pt:.1}%). \
         Chúng không nói được gì về {} tệp còn lại.",
        tong - so_anh
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chon_mau_khong_qua_muoi_hai_va_khong_trung() {
        for n in [0usize, 1, 5, 12, 13, 100, 57_351] {
            let v = chon_mau(n, 12345);
            assert!(v.len() <= SO_ANH.min(n).max(0), "n={n} lấy {} mẫu", v.len());
            assert!(v.iter().all(|&i| i < n.max(1) || n == 0));
            let mut s = v.clone();
            s.dedup();
            assert_eq!(s.len(), v.len(), "n={n} có chỉ số trùng");
        }
    }

    /// Ít hơn ngưỡng thì lấy hết — không được bỏ sót tệp nào khi chỉ có vài tệp.
    #[test]
    fn it_hon_nguong_thi_lay_het() {
        assert_eq!(chon_mau(5, 1), vec![0, 1, 2, 3, 4]);
        assert_eq!(chon_mau(0, 1), Vec::<usize>::new());
    }

    /// Cùng hạt thì cùng mẫu — phép thử phải tái lập được.
    #[test]
    fn cung_hat_thi_cung_mau() {
        assert_eq!(chon_mau(1000, 7), chon_mau(1000, 7));
        assert_ne!(
            chon_mau(1000, 7),
            chon_mau(1000, 8),
            "hai hạt khác nhau lại cho cùng một mẫu"
        );
    }

    /// Mẫu **không được** dồn vào đầu danh sách: lấy mười hai tệp đầu thì người
    /// dùng chỉ nhìn thấy đúng một thư mục.
    #[test]
    fn mau_trai_ra_ca_danh_sach_chu_khong_don_dau() {
        let v = chon_mau(10_000, 42);
        assert_eq!(v.len(), SO_ANH);
        assert!(
            *v.last().unwrap() > 1_000,
            "mười hai mẫu đều nằm trong 1.000 phần tử đầu: {v:?}"
        );
    }

    #[test]
    fn thu_nho_giu_ty_le_va_khong_vuot_canh() {
        let d = vec![0u8; 400 * 200 * 4];
        let a = thu_nho(400, 200, &d);
        assert_eq!(a.rong, CANH as usize);
        assert_eq!(a.cao, CANH as usize / 2);
        assert_eq!(a.diem.len(), a.rong * a.cao * 4);
    }

    /// Ảnh nhỏ hơn ngưỡng thì **không phóng to** — phóng lên chỉ ra một khối mờ.
    #[test]
    fn anh_nho_hon_nguong_thi_giu_nguyen() {
        let d = vec![0u8; 40 * 20 * 4];
        let a = thu_nho(40, 20, &d);
        assert_eq!((a.rong, a.cao), (40, 20));
    }

    #[test]
    fn thu_nho_khong_no_voi_dau_vao_di_dang() {
        let a = thu_nho(0, 0, &[]);
        assert_eq!((a.rong, a.cao), (0, 0));
        // Đệm ngắn hơn kích thước khai báo: phải chịu được, không hoảng.
        let a = thu_nho(100, 100, &[0u8; 16]);
        assert!(a.rong > 0);
    }

    /// Giải mã tệp rác phải ra `None`, **không** làm nổ chương trình.
    #[test]
    fn tep_rac_ra_none_chu_khong_no() {
        let d = std::env::temp_dir().join(format!("zanh_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("rac.bin");
        std::fs::write(&p, b"day khong phai anh gi ca").unwrap();
        let (loai, anh) = giai_ma(&p);
        assert_eq!(loai, LoaiTep::KhongRo);
        assert!(anh.is_none());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Tệp JPEG **cụt đầu** — magic byte đúng mà nội dung hỏng. Phải nhận đúng
    /// loại rồi trả `None` ảnh, chứ không được hoảng.
    #[test]
    fn jpeg_hong_nhan_dung_loai_nhung_khong_ra_anh() {
        let d = std::env::temp_dir().join(format!("zanh2_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("cut");
        std::fs::write(&p, [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]).unwrap();
        let (loai, anh) = giai_ma(&p);
        assert_eq!(loai, LoaiTep::Jpeg, "vẫn phải nhận ra là JPEG");
        assert!(anh.is_none());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Giải mã được một tệp PNG thật, dựng ngay trong phép thử.
    #[test]
    fn giai_ma_duoc_png_that() {
        let d = std::env::temp_dir().join(format!("zanh3_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("that.png");
        let img = image::RgbaImage::from_fn(64, 32, |x, y| {
            image::Rgba([(x * 4) as u8, (y * 8) as u8, 128, 255])
        });
        img.save(&p).unwrap();

        let (loai, anh) = giai_ma(&p);
        assert_eq!(loai, LoaiTep::Png);
        let a = anh.expect("phải giải mã được PNG thật");
        assert_eq!((a.rong, a.cao), (64, 32));
        assert_eq!(a.diem.len(), 64 * 32 * 4);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Giải mã **tệp `.jxl` THẬT** của Zalo, nếu máy này có dữ liệu.
    ///
    /// Phép thử dựng máy móc không chạm tới đường JPEG XL, mà `.jxl` chiếm
    /// **46,4%** dữ liệu Zalo thật — tức nhánh đông nhất lại là nhánh dễ để
    /// trống nhất. Máy không có dữ liệu thì tự bỏ qua và **in ra dòng `CHÚ Ý`**
    /// chứ không im lặng báo xanh, giống bộ đối chiếu song song.
    #[test]
    fn giai_ma_duoc_jxl_that_neu_may_nay_co_du_lieu() {
        let goc = std::env::var("ZALO_DOI_CHIEU_GOC").unwrap_or_else(|_| {
            r"C:\Users\ADMIN\AppData\Roaming\ZaloData\media\2068096368017928379\ZaloDownloads"
                .to_string()
        });
        if !Path::new(&goc).is_dir() {
            eprintln!("CHÚ Ý: không có dữ liệu Zalo thật nên bỏ qua phép thử JPEG XL.");
            return;
        }
        // Đo tận nơi: `.jxl` **không** nằm ở `picture\` cấp một mà nằm sâu trong
        // `resource\<hội thoại>\picture\`. Bản đầu của phép thử này chỉ ngó hai
        // thư mục cấp một rồi báo "không thấy tệp .jxl nào" — tức nó tự bỏ qua
        // chính nhánh đông nhất mà vẫn in ra một dòng nghe như bình thường.
        //
        // Duyệt nông có kiểm soát: tối đa vài chục thư mục hội thoại, dừng ngay
        // khi đủ mẫu. Đây là phép thử, không phải lượt quét.
        let mut mau: Vec<PathBuf> = Vec::new();
        let mut hang: Vec<PathBuf> = vec![Path::new(&goc).join("resource")];
        let mut da_mo = 0usize;
        while let Some(d) = hang.pop() {
            if mau.len() >= 3 || da_mo > 200 {
                break;
            }
            da_mo += 1;
            let doc = match std::fs::read_dir(&d) {
                Ok(x) => x,
                Err(_) => continue,
            };
            for e in doc.flatten() {
                let p = e.path();
                if p.is_dir() {
                    hang.push(p);
                } else if p.extension().map(|x| x == "jxl").unwrap_or(false) {
                    mau.push(p);
                    if mau.len() >= 3 {
                        break;
                    }
                }
            }
        }
        if mau.is_empty() {
            eprintln!("CHÚ Ý: không thấy tệp .jxl nào để thử.");
            return;
        }
        let mut xong = 0;
        for p in &mau {
            let (loai, a) = giai_ma(p);
            assert_eq!(loai, LoaiTep::JpegXl, "{p:?} không nhận ra là JPEG XL");
            let a = a.unwrap_or_else(|| panic!("không giải mã được {p:?}"));
            assert!(a.rong > 0 && a.cao > 0);
            assert!(a.rong <= CANH as usize && a.cao <= CANH as usize);
            assert_eq!(a.diem.len(), a.rong * a.cao * 4);
            // Ảnh không được toàn trong suốt — đó là dấu hiệu ghép kênh sai.
            assert!(
                a.diem.chunks(4).any(|px| px[3] > 0),
                "{p:?} giải ra ảnh trong suốt hoàn toàn — ghép kênh sai"
            );
            xong += 1;
        }
        eprintln!("đã giải mã {xong} tệp .jxl thật của Zalo");
    }

    /// **RB-43.** Dòng tỷ lệ mẫu phải nói thẳng phần *không* biết gì.
    #[test]
    fn dong_ty_le_mau_noi_thang_phan_khong_biet() {
        let s = dong_ty_le_mau(12, 12_418);
        assert!(s.contains("12 ảnh"));
        assert!(s.contains("12.406") || s.contains("12406"));
        assert!(s.contains("không nói được gì"));
        assert_eq!(dong_ty_le_mau(0, 0), "");
    }

    /// Mẫu phủ hết thì phải đổi câu. Đã nhìn thấy trên màn hình thật: bản đầu
    /// in ra "không nói được gì về 0 tệp còn lại" — một câu vô nghĩa nằm đúng
    /// chỗ cảnh báo, tức dạy người đọc bỏ qua cả những câu có nghĩa.
    #[test]
    fn mau_phu_het_thi_doi_cau() {
        let s = dong_ty_le_mau(11, 11);
        assert!(!s.contains("0 tệp còn lại"), "câu vô nghĩa: {s}");
        assert!(s.contains("cả 11 tệp"));
        // Mẫu lớn hơn tổng thì cũng ngã về nhánh ấy chứ không tính ra số âm.
        assert!(!dong_ty_le_mau(12, 5).contains("còn lại"));
    }
}
