//! Duyệt cây thư mục.
//!
//! **KHÔNG đi xuyên reparse point** (`R-09`). Đi xuyên là mở đường cho một lệnh
//! xóa lan sang thư mục ở đầu bên kia của junction.
//!
//! # Vì sao tự viết thay vì dùng `walkdir`
//!
//! Kế hoạch đánh dấu `walkdir` là ứng viên **chưa chốt**, chờ một phép đo:
//! junction NTFS không phải symlink, nên "mặc định không theo symlink" chưa
//! chắc đã chặn junction.
//!
//! Đã đo trên junction thật: `walkdir` **không** đi xuyên, nhưng bản tự duyệt
//! cũng vậy — và bản tự duyệt **không tốn thêm crate nào**. Nên loại `walkdir`.
//!
//! Phép đo còn cho thấy Rust báo junction là `is_dir() == false`, tức riêng
//! điều đó đã đủ chặn. Chốt tường minh theo cờ `FILE_ATTRIBUTE_REPARSE_POINT`
//! vẫn được giữ: dựa vào một hành vi phụ của thư viện để canh một cửa an toàn
//! là đặt cược vào thứ có thể đổi mà không báo trước.
//!
//! Duyệt bằng **ngăn xếp chứ không đệ quy**: cây sâu bất thường thì đệ quy tràn
//! ngăn xếp, mà tràn giữa một lượt quét thì người dùng nhận một danh sách cụt.
//!
//! Đếm được lỗi truy cập chứ không nuốt. Mốc **M2**.

use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

/// `FILE_ATTRIBUTE_REPARSE_POINT`.
#[cfg(windows)]
const REPARSE_POINT: u32 = 0x400;

/// Một tệp tìm được.
#[derive(Clone, Debug)]
pub struct Tep {
    pub duong_dan: PathBuf,
    pub co: u64,
}

/// Kết quả một lượt duyệt.
#[derive(Debug, Default)]
pub struct KetQuaDuyet {
    pub tep: Vec<Tep>,
    /// Số lỗi truy cập. **Đếm chứ không nuốt.**
    pub loi: usize,
}

#[cfg(windows)]
fn la_reparse_point(md: &std::fs::Metadata) -> bool {
    md.file_attributes() & REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn la_reparse_point(md: &std::fs::Metadata) -> bool {
    md.file_type().is_symlink()
}

/// Duyệt toàn bộ tệp dưới `goc`, không đi xuyên reparse point.
pub fn duyet(goc: &Path) -> KetQuaDuyet {
    let mut ra = KetQuaDuyet::default();
    let mut hang: Vec<PathBuf> = vec![goc.to_path_buf()];

    while let Some(d) = hang.pop() {
        let doc = match std::fs::read_dir(&d) {
            Ok(x) => x,
            Err(_) => {
                // Bản PowerShell duyệt bằng HAI phép liệt kê riêng — một cho
                // tệp, một cho thư mục con — và mỗi phép hỏng thì đếm một lỗi.
                // Thư mục không đọc được thì cả hai cùng hỏng, tức HAI lỗi.
                // Đếm hai ở đây để con số khớp bản kia; gộp thành một lượt
                // read_dir chỉ là chuyện hiệu năng chứ không phải ngữ nghĩa.
                ra.loi += 2;
                continue;
            }
        };

        for e in doc {
            let e = match e {
                Ok(x) => x,
                Err(_) => {
                    ra.loi += 1;
                    continue;
                }
            };
            // `DirEntry::metadata` trên Windows dùng dữ liệu đã có sẵn từ lượt
            // liệt kê, và KHÔNG đi theo reparse point — đúng thứ cần ở đây.
            let md = match e.metadata() {
                Ok(m) => m,
                Err(_) => {
                    ra.loi += 1;
                    continue;
                }
            };
            if la_reparse_point(&md) {
                continue;
            }
            if md.is_dir() {
                hang.push(e.path());
            } else if md.is_file() {
                ra.tep.push(Tep {
                    duong_dan: e.path(),
                    co: md.len(),
                });
            }
        }
    }
    ra
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Sandbox(PathBuf);
    impl Sandbox {
        fn moi(ten: &str) -> Self {
            let p = std::env::temp_dir().join(format!("zwalk_{}_{ten}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Sandbox(p)
        }
        fn tep(&self, rel: &str, n: usize) {
            let p = self.0.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, vec![0u8; n]).unwrap();
        }
    }
    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn tim_du_tep_o_moi_do_sau() {
        let s = Sandbox::moi("sau");
        s.tep("a.bin", 1);
        s.tep("x/b.bin", 2);
        s.tep("x/y/z/c.bin", 3);
        let r = duyet(&s.0);
        assert_eq!(r.tep.len(), 3);
        assert_eq!(r.loi, 0);
        let mut co: Vec<u64> = r.tep.iter().map(|t| t.co).collect();
        co.sort();
        assert_eq!(co, vec![1, 2, 3]);
    }

    #[test]
    fn goc_khong_ton_tai_thi_dem_loi_chu_khong_hoang() {
        let r = duyet(Path::new(r"C:\khong_he_ton_tai_zzz_0192"));
        assert!(r.tep.is_empty());
        assert!(r.loi > 0, "phải đếm lỗi chứ không im lặng trả về rỗng");
    }

    #[test]
    fn thu_muc_rong_khong_sinh_loi() {
        let s = Sandbox::moi("rong");
        std::fs::create_dir_all(s.0.join("trong/loi")).unwrap();
        let r = duyet(&s.0);
        assert_eq!(r.tep.len(), 0);
        assert_eq!(r.loi, 0);
    }

    /// Phép thử quan trọng nhất của mô-đun này — cổng ② của mốc M2.
    ///
    /// Junction NTFS không phải symlink, nên "thư viện không theo symlink" chưa
    /// chắc đã chặn nó. Dựng junction THẬT rồi đo, đúng như kế hoạch đòi.
    #[cfg(windows)]
    #[test]
    fn khong_di_xuyen_junction() {
        let s = Sandbox::moi("junction");
        s.tep("goc/a.bin", 8);
        s.tep("that/t1.bin", 8);
        s.tep("that/t2.bin", 8);
        s.tep("that/t3.bin", 8);

        let lien_ket = s.0.join("goc").join("lienket");
        let dich = s.0.join("that");
        let ok = std::process::Command::new("cmd")
            .args([
                "/c",
                "mklink",
                "/J",
                lien_ket.to_str().unwrap(),
                dich.to_str().unwrap(),
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(
            ok,
            "không dựng được junction để thử — bỏ qua phép thử này là bỏ trống cổng ② của M2"
        );

        let r = duyet(&s.0.join("goc"));
        let so = r.tep.len();

        // Gỡ junction TRƯỚC khi Sandbox::drop xóa cây, đúng bài học của công cụ.
        let _ = std::fs::remove_dir(&lien_ket);

        assert_eq!(
            so, 1,
            "ĐÃ ĐI XUYÊN JUNCTION: tìm thấy {so} tệp thay vì 1. Đây là đường để một \
             lệnh xóa lan sang thư mục ở đầu bên kia."
        );
    }

    /// Chốt reparse point phải có phép thử RIÊNG, không chỉ dựa vào phép thử duyệt.
    ///
    /// Lý do: trên Windows, Rust báo junction là `is_dir() == false`, nên vòng
    /// duyệt đã không xuống đó dù có chốt hay không. Đã kiểm bằng đột biến — gỡ
    /// hẳn chốt thì phép thử `khong_di_xuyen_junction` vẫn xanh.
    ///
    /// Đó là phòng thủ hai lớp chứ không phải chốt thừa: lớp kia là một hành vi
    /// phụ của thư viện, có thể đổi mà không báo trước. Nhưng một lớp an toàn
    /// không có phép thử của riêng nó là một lớp chưa từng được chứng minh, nên
    /// kiểm thẳng vào hàm.
    #[cfg(windows)]
    #[test]
    fn nhan_dien_dung_reparse_point() {
        let s = Sandbox::moi("nhandien");
        s.tep("that/x.bin", 4);
        std::fs::create_dir_all(s.0.join("thuong")).unwrap();

        let lien_ket = s.0.join("lienket");
        let ok = std::process::Command::new("cmd")
            .args([
                "/c",
                "mklink",
                "/J",
                lien_ket.to_str().unwrap(),
                s.0.join("that").to_str().unwrap(),
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(ok, "không dựng được junction để thử");

        let md_lk = std::fs::symlink_metadata(&lien_ket).unwrap();
        let md_thuong = std::fs::symlink_metadata(s.0.join("thuong")).unwrap();
        let md_tep = std::fs::symlink_metadata(s.0.join("that").join("x.bin")).unwrap();

        let _ = std::fs::remove_dir(&lien_ket);

        assert!(
            la_reparse_point(&md_lk),
            "KHÔNG nhận ra junction là reparse point — chốt tường minh mất tác dụng"
        );
        assert!(!la_reparse_point(&md_thuong), "thư mục thường bị nhận nhầm");
        assert!(!la_reparse_point(&md_tep), "tệp thường bị nhận nhầm");
    }
}
