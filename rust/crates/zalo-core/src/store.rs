//! catalog.json · settings · profiles · nhật ký · CSV.
//!
//! `catalog.json` **dùng chung** với bản PowerShell, nên hai bản phải chấp nhận
//! cùng một tập lỗi định dạng và nêu tên mục sai **giống nhau** (`R-06`).
//!
//! `logs\` cũng dùng chung — lịch sử dọn dẹp phải là một dòng duy nhất — nên
//! mỗi nhật ký thêm một dòng đầu ghi rõ bản nào đã ghi nó. Mốc **M2**.
//!
//! # Đọc JSON kiểu nào cũng phải nuốt lỗi cho đúng chỗ
//!
//! Bản PowerShell bọc cả phép đọc `settings.json` trong `try { } catch { }`
//! rỗng: tệp hỏng thì **giữ nguyên giá trị mặc định** và chạy tiếp, chứ không
//! dừng công cụ. Bản này giữ đúng nết đó — một tệp cấu hình hỏng không được
//! phép chặn người dùng khỏi việc khôi phục dữ liệu của họ. Mốc **M3**.

use serde_json::Value;
use std::path::{Path, PathBuf};

/// Tên tệp bản kê nằm trong mỗi thư mục sao lưu. Hợp đồng giữa hai bản.
pub const TEN_BAN_KE: &str = "_zalocleanup_backup.json";

/// Chính sách sao lưu. Sao lưu **không** bắt buộc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChinhSach {
    /// Mỗi lần xóa dữ liệu thật sẽ hỏi. Mặc định.
    Hoi,
    /// Không hỏi.
    Khong,
    /// Phải có bản sao lưu sạch mới cho xóa.
    BatBuoc,
}

impl ChinhSach {
    pub fn tu_chuoi(s: &str) -> Option<Self> {
        match s {
            "HOI" => Some(ChinhSach::Hoi),
            "KHONG" => Some(ChinhSach::Khong),
            "BATBUOC" => Some(ChinhSach::BatBuoc),
            _ => None,
        }
    }
    pub fn ra_chuoi(self) -> &'static str {
        match self {
            ChinhSach::Hoi => "HOI",
            ChinhSach::Khong => "KHONG",
            ChinhSach::BatBuoc => "BATBUOC",
        }
    }
    /// Nhãn hiển thị. Tương ứng `Show-PolicyLabel`.
    pub fn nhan(self) -> &'static str {
        match self {
            ChinhSach::Hoi => "Hỏi mỗi lần xóa dữ liệu thật",
            ChinhSach::Khong => "Không hỏi",
            ChinhSach::BatBuoc => "Bắt buộc sao lưu sạch mới cho xóa",
        }
    }
}

/// Nội dung `settings.json`.
#[derive(Debug, Clone)]
pub struct CaiDat {
    pub chinh_sach: ChinhSach,
    /// Các thư mục đã từng dùng để sao lưu, nhớ để lần sau khỏi phải nhớ đường dẫn.
    pub goc_sao_luu: Vec<String>,
}

impl Default for CaiDat {
    fn default() -> Self {
        CaiDat {
            chinh_sach: ChinhSach::Hoi,
            goc_sao_luu: Vec::new(),
        }
    }
}

/// Đọc `settings.json`. Thiếu tệp hoặc tệp hỏng đều trả về mặc định, **không lỗi**.
///
/// Giữ đúng nết bản PowerShell tới từng chi tiết: một giá trị `BackupPolicy` lạ
/// **không** ghi đè mặc định, còn `BackupRoots` thì nhận cả dạng chuỗi đơn lẫn
/// dạng mảng vì `ConvertTo-Json` của PowerShell rút mảng một phần tử thành chuỗi.
pub fn doc_cai_dat(tep: &Path) -> CaiDat {
    let mut cd = CaiDat::default();
    let noi_dung = match std::fs::read_to_string(tep) {
        Ok(s) => s,
        Err(_) => return cd,
    };
    let v: Value = match serde_json::from_str(bo_bom(&noi_dung)) {
        Ok(v) => v,
        Err(_) => return cd,
    };
    if let Some(s) = v.get("BackupPolicy").and_then(Value::as_str) {
        if let Some(c) = ChinhSach::tu_chuoi(s) {
            cd.chinh_sach = c;
        }
    }
    match v.get("BackupRoots") {
        Some(Value::Array(a)) => {
            cd.goc_sao_luu = a
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
        }
        Some(Value::String(s)) => cd.goc_sao_luu = vec![s.clone()],
        _ => {}
    }
    cd
}

/// Ghi `settings.json` theo **đúng dạng** `ConvertTo-Json` của PowerShell sinh ra.
pub fn ghi_cai_dat(tep: &Path, cd: &CaiDat) -> std::io::Result<()> {
    let v = serde_json::json!({
        "BackupPolicy": cd.chinh_sach.ra_chuoi(),
        "BackupRoots": cd.goc_sao_luu,
    });
    std::fs::write(tep, serde_json::to_string_pretty(&v).unwrap_or_default())
}

/// Bỏ BOM UTF-8 nếu có.
///
/// `Set-Content -Encoding UTF8` của PowerShell 5.1 **luôn** ghi BOM, nên mọi tệp
/// cấu hình do bản PowerShell tạo ra đều mở đầu bằng ba byte đó. `serde_json`
/// coi chúng là rác và bỏ cuộc ngay ký tự đầu tiên.
fn bo_bom(s: &str) -> &str {
    s.strip_prefix('\u{feff}').unwrap_or(s)
}

/// Bản kê của một thư mục sao lưu — nội dung `_zalocleanup_backup.json`.
#[derive(Debug, Clone, Default)]
pub struct BanKe {
    pub tao_luc: String,
    pub goc_nguon: String,
    pub loai_quet: String,
    pub so_tep: i64,
    pub so_byte: i64,
    pub chep_hong: i64,
    pub xac_minh_hong: i64,
}

/// Một bản sao lưu tìm được: thư mục chứa nó, và bản kê bên trong.
#[derive(Debug, Clone)]
pub struct BoSaoLuu {
    pub thu_muc: PathBuf,
    pub ban_ke: BanKe,
}

fn so(v: &Value, ten: &str) -> i64 {
    v.get(ten).and_then(Value::as_i64).unwrap_or(0)
}
fn chu(v: &Value, ten: &str) -> String {
    v.get(ten).and_then(Value::as_str).unwrap_or("").to_string()
}

/// Đọc một thư mục xem có phải bản sao lưu không. Tương ứng `Read-BackupSet`.
pub fn doc_bo_sao_luu(thu_muc: &Path) -> Option<BoSaoLuu> {
    let idx = thu_muc.join(TEN_BAN_KE);
    let noi_dung = std::fs::read_to_string(&idx).ok()?;
    let v: Value = serde_json::from_str(bo_bom(&noi_dung)).ok()?;
    Some(BoSaoLuu {
        thu_muc: thu_muc.to_path_buf(),
        ban_ke: BanKe {
            tao_luc: chu(&v, "Created"),
            goc_nguon: chu(&v, "SourceRoot"),
            loai_quet: chu(&v, "ScanKind"),
            so_tep: so(&v, "Count"),
            so_byte: so(&v, "Bytes"),
            chep_hong: so(&v, "CopyFail"),
            xac_minh_hong: so(&v, "VerifyFail"),
        },
    })
}

/// Đi tìm bản sao lưu thay vì bắt người dùng nhớ đường dẫn.
///
/// Ưu tiên các thư mục đã từng dùng, sau đó **quét nông** các ổ đĩa — gốc ổ và
/// một cấp con. Quét sâu là đi lục cả máy của người ta để tìm một thư mục.
///
/// Sắp giảm dần theo `Created` để bản mới nhất nằm trên cùng, y như bản kia.
pub fn tim_ban_sao_luu(goc_da_dung: &[String], cac_o: &[String]) -> Vec<BoSaoLuu> {
    let mut ung_vien: Vec<PathBuf> = Vec::new();
    for r in goc_da_dung {
        let p = PathBuf::from(r);
        if p.is_dir() {
            ung_vien.push(p);
        }
    }
    for o in cac_o {
        let p = PathBuf::from(o);
        ung_vien.push(p.clone());
        if let Ok(doc) = std::fs::read_dir(&p) {
            for e in doc.flatten() {
                if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    ung_vien.push(e.path());
                }
            }
        }
    }

    let mut ra: Vec<BoSaoLuu> = Vec::new();
    let mut da_thay: Vec<String> = Vec::new();
    let them = |bo: BoSaoLuu, ra: &mut Vec<BoSaoLuu>, da_thay: &mut Vec<String>| {
        let k = bo.thu_muc.to_string_lossy().to_lowercase();
        if !da_thay.contains(&k) {
            da_thay.push(k);
            ra.push(bo);
        }
    };

    for c in ung_vien {
        // Bản thân thư mục có thể là một bản sao lưu.
        if let Some(bo) = doc_bo_sao_luu(&c) {
            them(bo, &mut ra, &mut da_thay);
            continue;
        }
        if let Ok(doc) = std::fs::read_dir(&c) {
            for e in doc.flatten() {
                if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                if let Some(bo) = doc_bo_sao_luu(&e.path()) {
                    them(bo, &mut ra, &mut da_thay);
                }
            }
        }
    }
    ra.sort_by(|a, b| b.ban_ke.tao_luc.cmp(&a.ban_ke.tao_luc));
    ra
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Hop(PathBuf);
    impl Hop {
        fn moi(ten: &str) -> Self {
            let p = std::env::temp_dir().join(format!("zstore_{}_{ten}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Hop(p)
        }
    }
    impl Drop for Hop {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn thieu_tep_thi_dung_mac_dinh() {
        let cd = doc_cai_dat(Path::new(r"C:\khong_he_co_tep_nay_9182.json"));
        assert_eq!(cd.chinh_sach, ChinhSach::Hoi);
        assert!(cd.goc_sao_luu.is_empty());
    }

    /// Tệp hỏng **không được** làm công cụ dừng. Xem chú thích đầu mô-đun.
    #[test]
    fn tep_hong_thi_van_dung_mac_dinh_chu_khong_no() {
        let h = Hop::moi("hong");
        let f = h.0.join("settings.json");
        std::fs::write(&f, "{ khong phai json hop le").unwrap();
        let cd = doc_cai_dat(&f);
        assert_eq!(cd.chinh_sach, ChinhSach::Hoi);
    }

    /// `Set-Content -Encoding UTF8` của PowerShell 5.1 luôn ghi BOM.
    #[test]
    fn doc_duoc_tep_co_bom_utf8() {
        let h = Hop::moi("bom");
        let f = h.0.join("settings.json");
        let mut b = vec![0xEF, 0xBB, 0xBF];
        b.extend_from_slice(br#"{"BackupPolicy":"BATBUOC","BackupRoots":["D:\\bk"]}"#);
        std::fs::write(&f, b).unwrap();
        let cd = doc_cai_dat(&f);
        assert_eq!(cd.chinh_sach, ChinhSach::BatBuoc);
        assert_eq!(cd.goc_sao_luu, vec![r"D:\bk".to_string()]);
    }

    /// `ConvertTo-Json` rút mảng một phần tử thành chuỗi trần. Phải nhận cả hai.
    #[test]
    fn nhan_ca_mang_lan_chuoi_don_cho_backuproots() {
        let h = Hop::moi("mang");
        let f = h.0.join("s.json");
        std::fs::write(&f, r#"{"BackupRoots":"D:\\mot"}"#).unwrap();
        assert_eq!(doc_cai_dat(&f).goc_sao_luu, vec![r"D:\mot".to_string()]);
        std::fs::write(&f, r#"{"BackupRoots":["D:\\a","D:\\b"]}"#).unwrap();
        assert_eq!(doc_cai_dat(&f).goc_sao_luu.len(), 2);
    }

    /// Giá trị chính sách lạ **không** được ghi đè mặc định.
    #[test]
    fn chinh_sach_la_thi_giu_mac_dinh() {
        let h = Hop::moi("la");
        let f = h.0.join("s.json");
        std::fs::write(&f, r#"{"BackupPolicy":"XOA_HET_DI"}"#).unwrap();
        assert_eq!(doc_cai_dat(&f).chinh_sach, ChinhSach::Hoi);
    }

    #[test]
    fn ghi_roi_doc_lai_ra_dung_cai_da_ghi() {
        let h = Hop::moi("vongtron");
        let f = h.0.join("s.json");
        let cd = CaiDat {
            chinh_sach: ChinhSach::Khong,
            goc_sao_luu: vec![r"E:\x".into(), r"F:\y".into()],
        };
        ghi_cai_dat(&f, &cd).unwrap();
        let lai = doc_cai_dat(&f);
        assert_eq!(lai.chinh_sach, ChinhSach::Khong);
        assert_eq!(lai.goc_sao_luu, cd.goc_sao_luu);
    }

    #[test]
    fn doc_duoc_ban_ke_sao_luu() {
        let h = Hop::moi("banke");
        let d = h.0.join("20260801_090000");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(
            d.join(TEN_BAN_KE),
            r#"{"Tool":"ZaloCleanup","Version":4,"Created":"01/08/2026 09:00:00",
                "SourceRoot":"C:\\x","ScanKind":"DỮ LIỆU ZALO","Count":3,"Bytes":450000,
                "FullVerify":false,"Verified":3,"VerifyFail":0,"CopyFail":0}"#,
        )
        .unwrap();
        let bo = doc_bo_sao_luu(&d).expect("phải đọc được bản kê");
        assert_eq!(bo.ban_ke.so_tep, 3);
        assert_eq!(bo.ban_ke.so_byte, 450000);
        assert_eq!(bo.ban_ke.loai_quet, "DỮ LIỆU ZALO");
        assert_eq!(bo.ban_ke.tao_luc, "01/08/2026 09:00:00");
    }

    #[test]
    fn thu_muc_khong_co_ban_ke_thi_khong_phai_ban_sao_luu() {
        let h = Hop::moi("khongke");
        assert!(doc_bo_sao_luu(&h.0).is_none());
    }

    #[test]
    fn tim_thay_ban_sao_luu_o_cap_con_va_khong_trung_lap() {
        let h = Hop::moi("tim");
        let d = h.0.join("20260801_090000");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join(TEN_BAN_KE), r#"{"Created":"01/08/2026 09:00:00"}"#).unwrap();
        let goc = h.0.to_string_lossy().to_string();
        // Đưa CÙNG một gốc vào hai lần để bắt lỗi đếm trùng.
        let ds = tim_ban_sao_luu(&[goc.clone(), goc], &[]);
        assert_eq!(ds.len(), 1, "cùng một bản sao lưu bị đếm hai lần");
        assert_eq!(ds[0].thu_muc, d);
    }

    #[test]
    fn ban_moi_nhat_nam_tren_cung() {
        let h = Hop::moi("sapxep");
        for t in ["20260101_000000", "20260801_090000", "20260401_120000"] {
            let d = h.0.join(t);
            std::fs::create_dir_all(&d).unwrap();
            // Chuỗi ngày dd/MM/yyyy không sắp được theo thứ tự chữ, nên bản kê
            // dùng đúng chuỗi mà bản PowerShell ghi ra và ta so y như nó.
            std::fs::write(
                d.join(TEN_BAN_KE),
                format!(r#"{{"Created":"{t}"}}"#).as_bytes(),
            )
            .unwrap();
        }
        let ds = tim_ban_sao_luu(&[h.0.to_string_lossy().to_string()], &[]);
        assert_eq!(ds.len(), 3);
        assert_eq!(ds[0].ban_ke.tao_luc, "20260801_090000");
    }
}
