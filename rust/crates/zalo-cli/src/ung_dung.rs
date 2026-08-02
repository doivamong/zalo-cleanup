//! Trạng thái và các màn hình của vỏ dòng lệnh.
//!
//! # Quy tắc đọc tệp này
//!
//! Mọi chuỗi in ra đây là **hành vi quan sát được** của công cụ, và bộ test
//! đầu-cuối so thẳng vào chúng. Sửa một chữ là sửa hợp đồng — chạy lại cổng M3
//! trước khi tin rằng mình chỉ đang "làm câu chữ đẹp hơn".
//!
//! # Phần chưa có ở mốc này
//!
//! Xóa, sao lưu và khôi phục thuộc mốc **M4**. Ở M3 chúng đi hết đường hỏi và
//! xác nhận — vì chính đường ấy là thứ giữ an toàn — rồi **dừng lại và nói
//! thẳng ra là chưa làm**, chứ không im lặng coi như xong. Một chốt an toàn giả
//! vờ đã chạy còn tệ hơn một chốt chưa có.

use crate::hien;
use crate::nhap::Nhap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use zalo_core::protect::{Luat, Muc, VungBaoVe};
use zalo_core::scan::{duoi_kieu_dotnet, duong_dan_tuong_doi, ten_tep};
use zalo_core::store::{
    doc_bo_sao_luu, doc_cai_dat, tim_ban_sao_luu, BoSaoLuu, CaiDat, TEN_BAN_KE,
};
use zalo_core::thoigian::{hom_nay, ngay_dia_phuong, Ngay};
use zalo_core::{sysinfo, walk};

/// Thư mục Zalo lưu bản độc lập — bản luôn được giữ lại khi khử trùng lặp.
const THU_MUC_DOC_LAP: [&str; 4] = ["video", "picture", "voice", "file"];

/// Các thư mục cache của ứng dụng Zalo, tính từ `ZaloData`.
const CACHE_UNG_DUNG: [&str; 8] = [
    "Cache",
    "Code Cache",
    "GPUCache",
    "DawnGraphiteCache",
    "DawnWebGPUCache",
    "ShaderCache",
    r"media\update",
    r"media\temp",
];

/// Hai thư mục bị chặn cứng ở tầng mã. Không bộ lọc nào chạm được.
const TEN_BAO_VE: [&str; 2] = ["Database", "Partitions"];

/// Mức xác nhận, phải **tương xứng với rủi ro**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MucXacNhan {
    /// Bắt gõ đúng cụm `XÓA`. Dành cho dữ liệu thật: mất là mất vĩnh viễn.
    GoCumTu,
    /// Chỉ hỏi `c/k`. Dành cho những gì lấy lại được.
    CoKhong,
}

/// Các loại quét chỉ cần xác nhận nhẹ, vì mất cũng lấy lại được: bản trùng lặp
/// đã xác minh SHA-256 là còn một bản giống hệt, còn cache thì ứng dụng tự dựng.
const QUET_NHE: [&str; 3] = ["BẢN TRÙNG LẶP", "CACHE ZALO", "CACHE HỆ THỐNG"];

/// Kết luận của một lượt chọn thư mục con.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChonThuMuc {
    /// Bấm Enter — không đổi gì.
    GiuNguyen,
    /// Gõ `*` — bỏ hết ràng buộc thư mục, quét tất cả. **Có ý**, không phải lỡ tay.
    TatCa,
    /// Có giá trị không hiểu được. Kèm danh sách chúng để nói lại cho người dùng.
    NhapSai(Vec<String>),
    /// Gõ toàn dấu phẩy và khoảng trắng.
    KhongChonDuocGi,
    /// Chọn được danh sách cụ thể.
    Chon(Vec<String>),
}

/// Phân tích chuỗi người dùng gõ khi chọn thư mục con.
///
/// # Nguyên tắc bất biến số 3 nằm trọn trong hàm này
///
/// Nhập sai thì **giữ nguyên**, không bao giờ tự mở rộng phạm vi. Mở rộng phạm
/// vi sau một lần gõ nhầm nghĩa là lượt xóa kế tiếp quét rộng hơn người dùng
/// tưởng — và họ không có cách nào biết, vì bộ lọc chỉ hiện dạng rút gọn.
///
/// Tách thành hàm thuần vì cổng M3 **không phủ được** ngã này: phép thử
/// đầu-cuối tương ứng chỉ kiểm CÂU CHỮ in ra, không kiểm trạng thái bộ lọc. Đo
/// bằng đột biến — cho nhánh nhập sai xóa sạch danh sách thư mục mà cổng M3 vẫn
/// xanh, vì công cụ vẫn in đúng câu "giữ nguyên" trong khi đã đổi.
pub fn phan_tich_chon_thu_muc(tho: &str, tat_ca: &[String]) -> ChonThuMuc {
    if tho.is_empty() {
        return ChonThuMuc::GiuNguyen;
    }
    if tho == "*" {
        return ChonThuMuc::TatCa;
    }
    let mut chon: Vec<String> = Vec::new();
    let mut hong: Vec<String> = Vec::new();
    for phan in tho.split(',') {
        let p = phan.trim();
        if p.is_empty() {
            continue;
        }
        match p.parse::<usize>() {
            Ok(n) if n >= 1 && n <= tat_ca.len() => chon.push(tat_ca[n - 1].clone()),
            _ => hong.push(p.to_string()),
        }
    }
    // Một giá trị hỏng là hỏng cả lượt. Nhận phần đúng và bỏ phần sai nghĩa là
    // người dùng gõ "1,9" trên danh sách bốn thư mục sẽ nhận về đúng thư mục 1
    // mà tưởng mình đã chọn hai.
    if !hong.is_empty() {
        return ChonThuMuc::NhapSai(hong);
    }
    if chon.is_empty() {
        return ChonThuMuc::KhongChonDuocGi;
    }
    chon.dedup();
    ChonThuMuc::Chon(chon)
}

/// Chọn mức xác nhận theo loại quét.
///
/// # Vì sao liệt kê loại NHẸ chứ không liệt kê loại NẶNG
///
/// Bản PowerShell viết `$isRealData = ($ScanKind -eq 'DỮ LIỆU ZALO')`, tức một
/// loại quét lạ sẽ rơi vào nhánh **nhẹ**. Trên tập loại quét hiện có, hai cách
/// viết cho kết quả giống hệt nhau ở mọi đầu vào — nên đây không phải một khác
/// biệt hành vi mà là một khác biệt về hướng ngã khi có người thêm loại mới.
///
/// Liệt kê chiều này thì loại quét mới **mặc định là nặng**, và người thêm nó
/// phải cố ý hạ mức xuống. Chiều kia thì quên một dòng là dữ liệu thật đi qua
/// cửa nhẹ mà không ai hay.
///
/// Hàm này tách rời khỏi màn hình vì cổng M3 **không phủ được nó**: phép thử
/// đầu-cuối duy nhất nói về mức xác nhận lại treo vào một lượt có xóa tệp, tức
/// thuộc mốc M4. Đã đo bằng đột biến — đổi mức xác nhận của bản trùng lặp thành
/// nặng mà cổng M3 vẫn xanh. Nên chốt này phải có phép thử của riêng nó.
pub fn muc_xac_nhan(loai_quet: &str) -> MucXacNhan {
    if QUET_NHE.contains(&loai_quet) {
        MucXacNhan::CoKhong
    } else {
        MucXacNhan::GoCumTu
    }
}

pub use zalo_core::act::TepQuet;

/// Trạng thái bản sao lưu vừa tạo, để bước xóa xét trước khi mở khóa.
#[derive(Clone)]
pub struct SaoLuuGanNhat {
    pub dau_quet: String,
    pub thu_muc: String,
    pub tong: usize,
    pub da_chep: usize,
    pub chep_hong: usize,
    pub xac_minh_hong: usize,
    pub het_cho: bool,
}

pub struct UngDung {
    pub goc: String,
    pub goc_du_lieu: String,
    pub co_zalo: bool,
    pub vbv: VungBaoVe,
    pub luat: Vec<Luat>,
    pub cai_dat: CaiDat,

    // ---- bộ lọc
    pub tu_ngay: Option<Ngay>,
    pub den_ngay: Option<Ngay>,
    pub thu_muc: Vec<String>,
    pub duoi: Vec<String>,
    pub loai_thu_muc: Vec<String>,
    pub loai_duoi: Vec<String>,
    pub co_toi_thieu_kb: u64,
    pub giu_rescache: bool,

    // ---- kết quả quét
    pub quet: Option<Vec<TepQuet>>,
    pub loai_quet: String,
    pub goc_quet: String,
    pub dau_quet: String,
    /// Các gốc để dọn thư mục rỗng sau khi xóa.
    pub goc_don_dep: Vec<String>,
    pub sao_luu_gan_nhat: Option<SaoLuuGanNhat>,

    // ---- bộ đệm đo cây
    pub cay: Option<(usize, u64)>,
    pub loi_quet_lan_cuoi: usize,

    pub nhap: Nhap,
}

impl UngDung {
    pub fn moi(goc: String, goc_du_lieu: String) -> Self {
        let thu_muc_cong_cu = sysinfo::thu_muc_cong_cu();
        let cai_dat = doc_cai_dat(&thu_muc_cong_cu.join("settings.json"));
        let goc = canon(&goc);
        let goc_du_lieu = canon(&goc_du_lieu);
        let luat = dung_luat_bao_ve(&thu_muc_cong_cu);
        let vbv = VungBaoVe::dung(&luat, &goc_du_lieu, &TEN_BAO_VE);
        let co_zalo = !goc.is_empty() && Path::new(&goc).is_dir();
        UngDung {
            goc,
            goc_du_lieu,
            co_zalo,
            vbv,
            luat,
            cai_dat,
            tu_ngay: None,
            den_ngay: None,
            thu_muc: Vec::new(),
            duoi: Vec::new(),
            loai_thu_muc: Vec::new(),
            loai_duoi: Vec::new(),
            co_toi_thieu_kb: 0,
            giu_rescache: true,
            quet: None,
            loai_quet: String::new(),
            goc_quet: String::new(),
            dau_quet: String::new(),
            goc_don_dep: Vec::new(),
            sao_luu_gan_nhat: None,
            cay: None,
            loi_quet_lan_cuoi: 0,
            nhap: Nhap::moi(),
        }
    }

    /// Nguyên tắc bất biến số 2: đổi bộ lọc là kết quả quét cũ **bị hủy**.
    ///
    /// Phải hủy cả bản sao lưu gần nhất. Giữ nó lại nghĩa là một bản sao lưu của
    /// **kết quả quét cũ** vẫn mở khóa được bước xóa cho **kết quả quét mới** —
    /// tức xóa những tệp chưa từng được sao lưu.
    fn huy_ket_qua_quet(&mut self) {
        self.quet = None;
        self.loai_quet.clear();
        self.dau_quet.clear();
        self.goc_don_dep.clear();
        self.sao_luu_gan_nhat = None;
    }

    /// Bản sao lưu hiện có đúng là của kết quả quét đang giữ, và **sạch**.
    ///
    /// Tương ứng `Test-BackupClean`. **Không tự viết lại phép kiểm ở đây**: gọi
    /// thẳng [`zalo_core::gate::sao_luu_sach`], nơi chốt ấy sống cùng phép thử
    /// của riêng nó. Chép logic an toàn ra chỗ thứ hai là mời hai chỗ trôi khỏi
    /// nhau, và chỗ trôi sẽ đúng là chỗ không có phép thử nào canh.
    fn sao_luu_sach(&self) -> bool {
        let bk = self
            .sao_luu_gan_nhat
            .as_ref()
            .map(|b| zalo_core::gate::KetQuaSaoLuu {
                dau_quet: b.dau_quet.clone(),
                tong: b.tong as u64,
                xong: b.da_chep as u64,
                loi_chep: b.chep_hong as u64,
                loi_xac_minh: b.xac_minh_hong as u64,
                het_cho: b.het_cho,
            });
        zalo_core::gate::sao_luu_sach(bk.as_ref(), &self.dau_quet)
    }

    fn co_sao_luu_cua_lan_quet_nay(&self) -> bool {
        matches!(&self.sao_luu_gan_nhat, Some(b) if b.dau_quet == self.dau_quet)
    }

    fn thu_muc_nhat_ky(&self) -> PathBuf {
        sysinfo::thu_muc_cong_cu().join("logs")
    }

    /// Lượt dọn này có chạm vào thư mục dữ liệu Zalo thật không.
    ///
    /// Tương ứng `Test-TouchesZalo`. Sandbox của bộ test nằm trong `%TEMP%` nên
    /// luôn trả `false` — đó là lý do các phép thử không bao giờ đụng tới nhánh
    /// hỏi đóng Zalo.
    fn cham_toi_zalo(&self) -> bool {
        let nen = match std::env::var("APPDATA") {
            Ok(a) if !a.trim().is_empty() => Path::new(&a).join("ZaloData"),
            _ => return false,
        };
        let nen = nen.to_string_lossy().to_lowercase();
        let mut cac: Vec<String> = self.goc_don_dep.clone();
        if cac.is_empty() {
            cac.push(self.goc_quet.clone());
        }
        cac.iter()
            .any(|p| !p.trim().is_empty() && p.to_lowercase().starts_with(&nen))
    }

    fn canh_bao_loi_quet(&self) {
        if self.loi_quet_lan_cuoi > 0 {
            println!();
            println!(
                "  Cảnh báo: {} mục không đọc được và đã bị bỏ qua.",
                hien::so(self.loi_quet_lan_cuoi as i64)
            );
            println!("  Kết quả có thể chưa đầy đủ. Thử đóng Zalo rồi quét lại.");
        }
    }

    /// Liệt kê tệp dưới một thư mục, đếm lỗi chứ không nuốt.
    fn duyet(&mut self, goc: &str) -> Vec<walk::Tep> {
        let r = walk::duyet(Path::new(goc));
        self.loi_quet_lan_cuoi += r.loi;
        r.tep
    }

    // ============================================================ màn hình chính

    pub fn chay(&mut self) -> Option<()> {
        loop {
            self.man_hinh_chinh();
            match self.nhap.dong("   Chọn")?.as_str() {
                "1" => self.tro_giup_lay_lai_dung_luong()?,
                "2" => {
                    if self.co_zalo {
                        self.xem_co_cay();
                        self.nhap.dong("  Enter để tiếp tục")?;
                    }
                }
                "3" => {
                    self.khoi_phuc()?;
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                "9" => self.menu_nang_cao()?,
                "0" => {
                    println!();
                    println!("   Thoát. Không có tác vụ nền nào được đặt lại.");
                    return None;
                }
                _ => {}
            }
        }
    }

    fn man_hinh_chinh(&mut self) {
        println!();
        println!("  ╔══════════════════════════════════════════════════════════╗");
        println!("  ║   DỌN DẸP ZALO — chỉ chạy khi bạn mở công cụ             ║");
        println!("  ╚══════════════════════════════════════════════════════════╝");
        println!();
        let goc_he_thong = sysinfo::goc_he_thong();
        let trong = sysinfo::byte_trong(&goc_he_thong);
        if trong >= 0 {
            println!(
                "   Ổ {} còn trống    {}",
                sysinfo::nhan_o_dia(&goc_he_thong),
                hien::co(trong)
            );
        }
        if self.co_zalo {
            if self.cay.is_none() {
                println!("   Đang đo thư mục Zalo...");
                self.do_co_cay();
            }
            if let Some((_, byte)) = self.cay {
                println!("   Thư mục Zalo      {}", hien::co(byte as i64));
            } else {
                println!("   Thư mục Zalo      chưa đo — bấm 2 để xem");
            }
        } else {
            println!("   Máy này chưa cài Zalo — vẫn dọn được cache hệ thống.");
        }
        println!();
        println!("   Bạn muốn làm gì?");
        println!();
        println!("    1   Lấy lại dung lượng ổ đĩa");
        if self.co_zalo {
            println!("    2   Xem máy đang chiếm bao nhiêu");
        }
        println!("    3   Khôi phục dữ liệu đã sao lưu");
        println!();
        println!("    9   Tùy chọn nâng cao");
        println!("    0   Thoát");
        println!();
    }

    fn do_co_cay(&mut self) {
        self.loi_quet_lan_cuoi = 0;
        let goc = self.goc.clone();
        let tep = self.duyet(&goc);
        let byte: u64 = tep.iter().map(|t| t.co).sum();
        self.cay = Some((tep.len(), byte));
    }

    fn xem_co_cay(&mut self) {
        hien::tieu_de("Dung lượng thư mục Zalo");
        self.do_co_cay();
        if let Some((n, byte)) = self.cay {
            println!();
            println!("  Tổng      : {} tệp", hien::so(n as i64));
            println!("  Dung lượng: {}", hien::co(byte as i64));
        }
        self.canh_bao_loi_quet();
    }

    // ============================================================ trình dẫn

    fn tro_giup_lay_lai_dung_luong(&mut self) -> Option<()> {
        let mut nhac = String::new();
        loop {
            hien::tieu_de("Lấy lại dung lượng ổ đĩa");
            if self.co_zalo {
                println!("   1  Dữ liệu Zalo cũ theo thời gian");
                println!("   2  Bản trùng lặp trong Zalo        — an toàn nhất, không mất ảnh");
                println!("   3  Cache của ứng dụng Zalo");
            } else {
                println!("   Máy này chưa cài Zalo nên ba mục về Zalo đã được ẩn.");
            }
            println!("   4  Cache hệ thống ngoài Zalo");
            println!("   0  Quay lại");
            if !nhac.is_empty() {
                println!();
                println!("   {nhac}");
                nhac.clear();
            }
            println!();
            let c = self.nhap.dong("  Chọn")?;
            if c.is_empty() || c == "0" {
                return Some(());
            }
            if matches!(c.as_str(), "1" | "2" | "3") && !self.co_zalo {
                nhac = "Mục đó cần Zalo, mà máy này chưa cài.".into();
                continue;
            }
            let co_ket_qua = match c.as_str() {
                "1" => self.tro_giup_zalo_cu()?,
                "2" => {
                    self.quet_trung_lap()?;
                    if self.co_ket_qua_quet() {
                        println!();
                        if self.nhap.co_khong("  Xem chi tiết vài cặp trùng? (c/k)")? {
                            self.xem_chi_tiet_quet();
                        }
                        self.xoa()?;
                    }
                    true
                }
                "3" => {
                    self.quet_cache_ung_dung();
                    if self.co_ket_qua_quet() {
                        self.xoa()?;
                    }
                    true
                }
                "4" => {
                    println!();
                    println!("  Cache hệ thống ngoài Zalo chưa có ở bản này — xem mốc M4.");
                    true
                }
                _ => {
                    nhac = format!("Không có mục \"{c}\". Chọn 1 đến 4, hoặc 0 để quay lại.");
                    continue;
                }
            };
            if co_ket_qua {
                self.nhap.dong("  Enter để quay lại menu")?;
            }
        }
    }

    fn co_ket_qua_quet(&self) -> bool {
        self.quet.as_ref().map(|v| !v.is_empty()).unwrap_or(false)
    }

    // ============================================================ mốc tuổi

    fn tro_giup_zalo_cu(&mut self) -> Option<bool> {
        hien::tieu_de("Dọn dữ liệu Zalo cũ");
        println!("  Đang đo dung lượng theo từng mốc thời gian...");
        let moc = self.do_moc_tuoi();

        hien::tieu_de("Dọn dữ liệu Zalo cũ");
        println!();
        // Chỉ hiện mốc THẬT SỰ có dữ liệu. Một mốc 0 byte là lựa chọn vô nghĩa,
        // và bộ test có phép thử riêng bắt đúng chuyện hiện nó ra.
        let mut lua_chon: Vec<(&str, &str, u64, usize)> = Vec::new();
        if moc.n12 > 0 {
            lua_chon.push(("1", "Cũ hơn 12 tháng", moc.b12, moc.n12));
        }
        if moc.n6 > 0 {
            lua_chon.push(("2", "Cũ hơn 6 tháng", moc.b6, moc.n6));
        }
        if moc.n_truoc > 0 {
            lua_chon.push(("3", "Trước năm 2026", moc.b_truoc, moc.n_truoc));
        }

        if lua_chon.is_empty() {
            println!("  Không còn dữ liệu cũ ở các mốc thường dùng.");
            println!(
                "  Toàn bộ thư mục Zalo hiện là {}.",
                hien::co(moc.b_tong as i64)
            );
        } else {
            println!("  Dọn dữ liệu cũ tới mốc nào?");
            println!();
            for (k, t, b, n) in &lua_chon {
                println!(
                    "   {}  {} →  {}   ({} tệp)",
                    k,
                    hien::trai(t, 16),
                    hien::phai(&hien::co(*b as i64), 10),
                    hien::so(*n as i64)
                );
            }
            // Số thứ tự giữ nguyên để quen tay, nên phải nói rõ vì sao có chỗ trống.
            if lua_chon.len() < 3 {
                println!();
                println!("   Các mốc còn lại không hiện vì không còn dữ liệu nào.");
            }
        }
        println!();
        println!("   4  Tôi tự nhập ngày");
        println!("   0  Quay lại");
        println!();
        self.canh_bao_loi_quet();

        let c = self.nhap.dong("  Chọn")?;
        if c.is_empty() || c == "0" {
            return Some(false);
        }
        if matches!(c.as_str(), "1" | "2" | "3") && !lua_chon.iter().any(|(k, ..)| *k == c) {
            println!("  Mốc đó không có dữ liệu nào.");
            return Some(true);
        }
        let nay = hom_nay();
        match c.as_str() {
            "1" => {
                self.tu_ngay = None;
                self.den_ngay = Some(nay.lui_thang(12));
            }
            "2" => {
                self.tu_ngay = None;
                self.den_ngay = Some(nay.lui_thang(6));
            }
            "3" => {
                self.tu_ngay = None;
                self.den_ngay = Some(Ngay {
                    nam: 2025,
                    thang: 12,
                    ngay: 31,
                });
            }
            "4" => {
                println!("  Tự nhập ngày chưa có ở bản này — xem mốc M4.");
                return Some(true);
            }
            _ => {
                println!("  Không hiểu lựa chọn.");
                return Some(true);
            }
        }
        self.huy_ket_qua_quet();
        self.quet_theo_bo_loc(false);
        if !self.co_ket_qua_quet() {
            println!();
            println!("  Không có tệp nào trong khoảng này.");
            return Some(true);
        }
        println!();
        if self
            .nhap
            .co_khong("  Xem chi tiết trước khi quyết định? (c/k)")?
        {
            self.xem_chi_tiet_quet();
        }
        self.xoa()?;
        Some(true)
    }

    fn do_moc_tuoi(&mut self) -> MocTuoi {
        let nay = hom_nay();
        let m12 = nay.lui_thang(12).nua_dem();
        let m6 = nay.lui_thang(6).nua_dem();
        let truoc_2026 = Ngay {
            nam: 2026,
            thang: 1,
            ngay: 1,
        }
        .nua_dem();

        self.loi_quet_lan_cuoi = 0;
        let goc = self.goc.clone();
        let mut m = MocTuoi::default();
        for t in self.duyet(&goc) {
            let s = t.duong_dan.to_string_lossy().to_string();
            if self.vbv.chan(&s) {
                continue;
            }
            if self.giu_rescache && duoi_kieu_dotnet(ten_tep(&s)).eq_ignore_ascii_case(".rescache")
            {
                continue;
            }
            m.b_tong += t.co;
            m.n_tong += 1;
            if t.sua_luc < m12 {
                m.b12 += t.co;
                m.n12 += 1;
            }
            if t.sua_luc < m6 {
                m.b6 += t.co;
                m.n6 += 1;
            }
            if t.sua_luc < truoc_2026 {
                m.b_truoc += t.co;
                m.n_truoc += 1;
            }
        }
        m
    }

    // ============================================================ quét theo bộ lọc

    fn quet_theo_bo_loc(&mut self, im_lang: bool) {
        if !Path::new(&self.goc).is_dir() {
            println!("Thư mục gốc không hợp lệ.");
            return;
        }
        if !im_lang {
            hien::tieu_de("Đang quét theo bộ lọc");
            println!("  Từ ngày  : {}", nhan_ngay(self.tu_ngay));
            println!("  Đến ngày : {}", nhan_ngay(self.den_ngay));
            println!("  Bước này chỉ đọc, không xóa gì.");
        }

        // Cận trên là HẾT ngày đó, không phải nửa đêm đầu ngày — chọn "đến
        // 31/12" mà mất sạch tệp của chính ngày 31/12 là một cái bẫy im lặng.
        let lo = self.tu_ngay.map(|n| n.nua_dem());
        let hi = self.den_ngay.map(|n| ngay_ke_tiep(n).nua_dem());
        let byte_toi_thieu = self.co_toi_thieu_kb * 1024;

        let mut cac_goc: Vec<String> = Vec::new();
        if self.thu_muc.is_empty() {
            cac_goc.push(self.goc.clone());
        } else {
            for f in &self.thu_muc {
                let p = Path::new(&self.goc).join(f);
                if p.is_dir() {
                    cac_goc.push(p.to_string_lossy().to_string());
                }
            }
        }

        let mut ket: Vec<TepQuet> = Vec::new();
        let mut bi_chan = 0usize;
        self.loi_quet_lan_cuoi = 0;
        for g in cac_goc {
            for t in self.duyet(&g) {
                let s = t.duong_dan.to_string_lossy().to_string();
                match self.xet_tep(&s, t.co, t.sua_luc, lo, hi, byte_toi_thieu) {
                    XetTep::Chan => bi_chan += 1,
                    XetTep::LoaiBoiBoLoc => {}
                    XetTep::Nhan => ket.push(TepQuet::moi(s, t.co)),
                }
            }
        }

        let tong: u64 = ket.iter().map(|t| t.co).sum();
        let n = ket.len();
        self.quet = Some(ket);
        self.loai_quet = "DỮ LIỆU ZALO".into();
        self.goc_quet = self.goc.clone();
        self.dau_quet = zalo_core::thoigian::luc_nay().dinh_dang();
        self.goc_don_dep = vec![self.goc.clone()];
        self.sao_luu_gan_nhat = None;

        println!();
        println!("  Tìm thấy   : {} tệp", hien::so(n as i64));
        println!("  Dung lượng : {}", hien::co(tong as i64));
        if bi_chan > 0 {
            println!(
                "  Đã chặn {} tệp thuộc vùng bảo vệ.",
                hien::so(bi_chan as i64)
            );
        }
        self.canh_bao_loi_quet();
    }

    /// Số phận của một tệp trong lượt quét theo bộ lọc.
    ///
    /// # Vì sao là một hàm riêng
    ///
    /// Vùng bảo vệ phải được hỏi **TRƯỚC MỌI THỨ KHÁC**, và phải hỏi ở đúng một
    /// chỗ. Trước đây phép kiểm ấy nằm rải trong vòng lặp, và đột biến cho thấy
    /// **gỡ hẳn nó ra thì cổng M3 vẫn xanh** — vì phép thử đầu-cuối nào chạm tới
    /// vùng bảo vệ cũng đều là phép thử có xóa tệp, tức thuộc mốc M4.
    ///
    /// Đây đúng loại lỗ đã từng cắn dự án này ở `Test-KeeperAlive`: chốt có mặt
    /// trong mã, nhưng không có gì chứng minh nó được GỌI. Gom vào một hàm thì
    /// phép thử đơn vị hỏi thẳng được.
    fn xet_tep(
        &self,
        duong_dan: &str,
        co: u64,
        sua_luc: std::time::SystemTime,
        lo: Option<std::time::SystemTime>,
        hi: Option<std::time::SystemTime>,
        byte_toi_thieu: u64,
    ) -> XetTep {
        if self.vbv.chan(duong_dan) {
            return XetTep::Chan;
        }
        if let Some(l) = lo {
            if sua_luc < l {
                return XetTep::LoaiBoiBoLoc;
            }
        }
        if let Some(h) = hi {
            if sua_luc >= h {
                return XetTep::LoaiBoiBoLoc;
            }
        }
        if co < byte_toi_thieu {
            return XetTep::LoaiBoiBoLoc;
        }
        if !self.qua_bo_loc(duong_dan) {
            return XetTep::LoaiBoiBoLoc;
        }
        XetTep::Nhan
    }

    /// Tương ứng `Test-PassFilterUnguarded`. **Không** kiểm vùng bảo vệ — người
    /// gọi có bổn phận kiểm trước, y như bên bản PowerShell.
    fn qua_bo_loc(&self, duong_dan: &str) -> bool {
        let duoi_goc = duoi_kieu_dotnet(ten_tep(duong_dan));
        if self.giu_rescache && duoi_goc.eq_ignore_ascii_case(".rescache") {
            return false;
        }
        let d = if duoi_goc.is_empty() {
            zalo_core::scan::KHONG_DUOI.to_string()
        } else {
            duoi_goc.to_lowercase()
        };
        if !self.duoi.is_empty() && !self.duoi.iter().any(|x| x.eq_ignore_ascii_case(&d)) {
            return false;
        }
        if !self.loai_duoi.is_empty() && self.loai_duoi.iter().any(|x| x.eq_ignore_ascii_case(&d)) {
            return false;
        }
        if !self.loai_thu_muc.is_empty() {
            let dau = thu_muc_cap_mot(duong_dan, &self.goc);
            if self
                .loai_thu_muc
                .iter()
                .any(|x| x.eq_ignore_ascii_case(dau))
            {
                return false;
            }
        }
        true
    }

    fn xem_chi_tiet_quet(&mut self) {
        hien::tieu_de("Chi tiết kết quả quét");
        let ds = match &self.quet {
            Some(v) if !v.is_empty() => v.clone(),
            _ => {
                println!("  Chưa có kết quả quét.");
                return;
            }
        };
        let mut sap = ds.clone();
        sap.sort_by(|a, b| b.co.cmp(&a.co));
        println!();
        println!("  Mười tệp lớn nhất:");
        for t in sap.iter().take(10) {
            println!(
                "   {}  [{}]",
                duong_dan_tuong_doi(&t.duong_dan, &self.goc_quet),
                hien::co(t.co as i64)
            );
        }
    }

    // ============================================================ cache ứng dụng

    fn quet_cache_ung_dung(&mut self) {
        if self.goc_du_lieu.trim().is_empty() || !Path::new(&self.goc_du_lieu).is_dir() {
            println!("Không xác định được thư mục ZaloData.");
            return;
        }
        hien::tieu_de("Cache của ứng dụng Zalo");
        println!("  Các thư mục cache nằm ngoài ZaloDownloads. Zalo tự tạo lại được.");
        println!("  Không chứa tin nhắn hay ảnh video đã nhận.");
        println!();

        let mut ket: Vec<TepQuet> = Vec::new();
        self.loi_quet_lan_cuoi = 0;
        for rel in CACHE_UNG_DUNG {
            let p = Path::new(&self.goc_du_lieu).join(rel);
            if !p.is_dir() {
                continue;
            }
            let ps = p.to_string_lossy().to_string();
            if self.vbv.chan_thu_muc_goc(&ps) {
                continue;
            }
            let mut n = 0usize;
            let mut b = 0u64;
            for t in self.duyet(&ps) {
                let s = t.duong_dan.to_string_lossy().to_string();
                if self.vbv.chan(&s) {
                    continue;
                }
                if duoi_kieu_dotnet(ten_tep(&s)).eq_ignore_ascii_case(".rescache") {
                    continue;
                }
                ket.push(TepQuet::moi(s, t.co));
                n += 1;
                b += t.co;
            }
            if n > 0 {
                println!(
                    "   {} {} tệp · {}",
                    hien::trai(rel, 22),
                    hien::phai(&hien::so(n as i64), 7),
                    hien::co(b as i64)
                );
            }
        }
        if ket.is_empty() {
            println!();
            println!("  Không có gì để dọn.");
            return;
        }
        let tong: u64 = ket.iter().map(|t| t.co).sum();
        let n = ket.len();
        self.quet = Some(ket);
        self.loai_quet = "CACHE ZALO".into();
        self.goc_quet = self.goc_du_lieu.clone();
        self.dau_quet = zalo_core::thoigian::luc_nay().dinh_dang();
        // Chỉ dọn thư mục rỗng TRONG các thư mục cache, không dọn cả ZaloData.
        self.goc_don_dep = CACHE_UNG_DUNG
            .iter()
            .map(|r| Path::new(&self.goc_du_lieu).join(r))
            .filter(|p| p.is_dir())
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        self.sao_luu_gan_nhat = None;
        println!();
        println!(
            "  Tổng: {} tệp · {}",
            hien::so(n as i64),
            hien::co(tong as i64)
        );
        self.canh_bao_loi_quet();
    }

    // ============================================================ khử trùng lặp

    fn quet_trung_lap(&mut self) -> Option<()> {
        if !Path::new(&self.goc).is_dir() {
            println!("Thư mục gốc không hợp lệ.");
            return Some(());
        }
        let goc_res = Path::new(&self.goc).join("resource");
        if !goc_res.is_dir() {
            println!("Không có thư mục resource để đối chiếu.");
            return Some(());
        }

        hien::tieu_de("Tìm bản trùng lặp");
        println!("  Zalo lưu mỗi tấm ảnh và mỗi video hai bản: một bản độc lập, một bản");
        println!("  trong thư mục theo hội thoại. Công cụ tìm bản thừa và luôn giữ bản gốc.");
        println!();
        println!("  Kết luận chỉ dựa trên đối chiếu nội dung bằng SHA256 toàn tệp.");
        println!();

        self.loi_quet_lan_cuoi = 0;
        println!("  Bước 1/4 · lập chỉ mục bản giữ lại...");
        let mut giu_theo_co: HashMap<u64, Vec<String>> = HashMap::new();
        let mut so_giu = 0usize;
        for d in THU_MUC_DOC_LAP {
            let p = Path::new(&self.goc).join(d);
            if !p.is_dir() {
                continue;
            }
            let ps = p.to_string_lossy().to_string();
            for t in self.duyet(&ps) {
                let s = t.duong_dan.to_string_lossy().to_string();
                if t.co == 0 || duoi_kieu_dotnet(ten_tep(&s)).eq_ignore_ascii_case(".rescache") {
                    continue;
                }
                if self.vbv.chan(&s) {
                    continue;
                }
                giu_theo_co.entry(t.co).or_default().push(s);
                so_giu += 1;
            }
        }
        println!(
            "          {} tệp độc lập, {} nhóm kích thước",
            hien::so(so_giu as i64),
            hien::so(giu_theo_co.len() as i64)
        );

        if so_giu == 0 {
            println!();
            println!("  Không còn bản độc lập nào để đối chiếu.");
            println!("  Mọi tệp trong resource lúc này đều là bản duy nhất — không xóa gì cả.");
            return Some(());
        }

        println!("  Bước 2/4 · lọc ứng viên theo kích thước...");
        let mut ung_vien: Vec<TepQuet> = Vec::new();
        let mut byte_ung_vien = 0u64;
        let res_s = goc_res.to_string_lossy().to_string();
        for t in self.duyet(&res_s) {
            let s = t.duong_dan.to_string_lossy().to_string();
            if t.co == 0 || duoi_kieu_dotnet(ten_tep(&s)).eq_ignore_ascii_case(".rescache") {
                continue;
            }
            // Cache nằm trong resource nhưng không phải bản trùng của gì cả.
            if s.contains("\\Cache\\") {
                continue;
            }
            if self.vbv.chan(&s) {
                continue;
            }
            if giu_theo_co.contains_key(&t.co) {
                byte_ung_vien += t.co;
                ung_vien.push(TepQuet::moi(s, t.co));
            }
        }
        println!(
            "          {} ứng viên · {}",
            hien::so(ung_vien.len() as i64),
            hien::co(byte_ung_vien as i64)
        );

        if ung_vien.is_empty() {
            println!();
            println!("  Không tìm thấy bản trùng nào.");
            return Some(());
        }

        println!();
        println!("  Bước tiếp theo đọc đĩa để băm nội dung, có thể mất vài phút.");
        if !self.nhap.co_khong("  Tiếp tục? (c/k)")? {
            println!("  Đã hủy.");
            return Some(());
        }

        println!();
        println!("  Bước 3/4 · lọc nhanh bằng chữ ký đầu và cuối tệp...");
        let mut chu_ky: HashMap<String, String> = HashMap::new();
        let mut loi = 0usize;
        let mut can_bam: Vec<String> = ung_vien.iter().map(|c| c.duong_dan.clone()).collect();
        for c in &ung_vien {
            if let Some(ds) = giu_theo_co.get(&c.co) {
                for kp in ds {
                    if Path::new(kp).is_file() {
                        can_bam.push(kp.clone());
                    }
                }
            }
        }
        can_bam.sort();
        can_bam.dedup();
        for p in &can_bam {
            // Tệp đọc không được thì bỏ qua ở đây và ĐẾM sau, lúc dò cặp: một
            // tệp không băm được là một tệp ta không biết gì về nội dung, nên
            // nó không bao giờ được coi là bản trùng.
            if let Ok(h) = zalo_core::hash::chu_ky_nhanh(Path::new(p)) {
                chu_ky.insert(p.clone(), h);
            }
        }

        let mut cap: Vec<(TepQuet, String, String)> = Vec::new();
        let mut loai_nhanh = 0usize;
        for c in &ung_vien {
            let cq = match chu_ky.get(&c.duong_dan) {
                Some(v) => v.clone(),
                None => {
                    loi += 1;
                    continue;
                }
            };
            if let Some(ds) = giu_theo_co.get(&c.co) {
                for kp in ds {
                    let kq = match chu_ky.get(kp) {
                        Some(v) => v,
                        None => continue,
                    };
                    if *kq == cq {
                        cap.push((c.clone(), kp.clone(), cq.clone()));
                        break;
                    } else {
                        loai_nhanh += 1;
                    }
                }
            }
        }
        println!(
            "          {} cặp qua vòng lọc nhanh",
            hien::so(cap.len() as i64)
        );

        println!("  Bước 4/4 · xác minh SHA256 toàn tệp...");
        // Tệp từ 128 KB trở xuống đã bị đọc TRỌN ở bước 3, nên chữ ký `FULL:`
        // của nó CHÍNH LÀ SHA-256 toàn tệp — đọc lại lần nữa là đọc thừa nguyên
        // một lượt toàn bộ dữ liệu. Kết luận không hề nới lỏng.
        let mut can_toan: Vec<String> = Vec::new();
        for (c, k, sig) in &cap {
            if sig.starts_with("FULL:") {
                continue;
            }
            can_toan.push(c.duong_dan.clone());
            can_toan.push(k.clone());
        }
        can_toan.sort();
        can_toan.dedup();
        let mut toan: HashMap<String, String> = HashMap::new();
        for p in &can_toan {
            if let Ok(h) = zalo_core::hash::sha256_toan_tep(Path::new(p)) {
                toan.insert(p.clone(), h);
            }
        }

        let mut trung: Vec<TepQuet> = Vec::new();
        let mut loai_toan = 0usize;
        for (c, k, sig) in &cap {
            if sig.starts_with("FULL:") {
                trung.push(gan_ban_giu_lai(c, k));
                continue;
            }
            match (toan.get(&c.duong_dan), toan.get(k)) {
                (Some(a), Some(b)) => {
                    if a == b {
                        trung.push(gan_ban_giu_lai(c, k));
                    } else {
                        loai_toan += 1;
                    }
                }
                _ => loi += 1,
            }
        }

        let byte_trung: u64 = trung.iter().map(|t| t.co).sum();
        println!();
        println!(
            "  Bản trùng xác nhận : {} tệp",
            hien::so(trung.len() as i64)
        );
        println!("  Có thể thu hồi     : {}", hien::co(byte_trung as i64));
        println!("  Loại ở vòng lọc nhanh : {}", hien::so(loai_nhanh as i64));
        println!("  Loại ở vòng toàn tệp  : {}", hien::so(loai_toan as i64));
        if loi > 0 {
            println!("  Lỗi đọc tệp        : {}", hien::so(loi as i64));
        }
        self.canh_bao_loi_quet();

        if trung.is_empty() {
            return Some(());
        }
        self.goc_quet = self.goc.clone();
        self.quet = Some(trung);
        self.loai_quet = "BẢN TRÙNG LẶP".into();
        self.dau_quet = zalo_core::thoigian::luc_nay().dinh_dang();
        self.goc_don_dep = vec![self.goc.clone()];
        self.sao_luu_gan_nhat = None;
        Some(())
    }

    // ============================================================ menu nâng cao

    fn menu_nang_cao(&mut self) -> Option<()> {
        loop {
            hien::tieu_de("Tùy chọn nâng cao");
            println!(
                "  Bộ lọc: {} · thư mục {} · đuôi {} · từ {} KB",
                nhan_khoang(self.tu_ngay, self.den_ngay),
                if self.thu_muc.is_empty() {
                    "tất cả".to_string()
                } else {
                    self.thu_muc.join(",")
                },
                if self.duoi.is_empty() {
                    "tất cả".to_string()
                } else {
                    self.duoi.join(",")
                },
                self.co_toi_thieu_kb
            );
            if let Some(ds) = &self.quet {
                let tong: u64 = ds.iter().map(|t| t.co).sum();
                println!(
                    "  Kết quả quét đang giữ: {} · {} tệp · {}",
                    self.loai_quet,
                    hien::so(ds.len() as i64),
                    hien::co(tong as i64)
                );
            }
            println!();
            println!("   1  Khoảng thời gian        2  Thư mục con");
            println!("   3  Đuôi tệp                4  Kích thước tối thiểu");
            println!("   5  Loại trừ                6  Hồ sơ bộ lọc");
            println!();
            println!("   7  Quét theo bộ lọc        8  Chi tiết + xuất CSV");
            // Nhãn phải nói rõ là xóa TỆP TRÊN ĐĨA. Nhãn cũ "Xóa kết quả quét
            // đang giữ" đọc tự nhiên trong tiếng Việt là "bỏ kết quả quét đi" —
            // một việc vô hại — trong khi phím này xóa vĩnh viễn.
            println!("   9  Sao lưu và xác minh     X  Xóa hẳn tệp trong kết quả quét");
            println!();
            println!("   K  Khôi phục từ bản sao lưu");
            println!("   V  Shadow Copy             B  Vùng bảo vệ");
            println!("   L  Lịch sử dọn dẹp         C  Chính sách sao lưu");
            println!("   T  Đổi tài khoản Zalo");
            println!("   0  Quay lại");
            println!();
            let c = self.nhap.dong("  Chọn")?.to_uppercase();
            if c.is_empty() || c == "0" {
                return Some(());
            }
            match c.as_str() {
                "2" => {
                    self.chon_thu_muc()?;
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                "7" => {
                    self.quet_theo_bo_loc(false);
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                "8" => {
                    self.xem_chi_tiet_quet();
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                "X" => {
                    self.xoa()?;
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                "K" => {
                    self.khoi_phuc()?;
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                "B" => {
                    self.bao_cao_vung_bao_ve();
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                "9" => {
                    self.sao_luu()?;
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                "1" | "3" | "4" | "5" | "6" | "V" | "L" | "C" | "T" => {
                    println!();
                    println!("  Mục này chưa có ở bản Rust — xem docs/ke-hoach-port.md.");
                    self.nhap.dong("  Enter để tiếp tục")?;
                }
                _ => {}
            }
        }
    }

    /// Tương ứng `Select-FolderList`. Nguyên tắc bất biến số 3 nằm ở đây: nhập
    /// sai thì **giữ nguyên**, không bao giờ tự mở rộng phạm vi.
    fn chon_thu_muc(&mut self) -> Option<()> {
        let tat_ca = thu_muc_con(&self.goc);
        if tat_ca.is_empty() {
            println!("Không có thư mục con.");
            return Some(());
        }
        hien::tieu_de("Thư mục con");
        for (i, t) in tat_ca.iter().enumerate() {
            println!("   {}  {}", hien::phai(&(i + 1).to_string(), 2), t);
        }
        println!();
        println!("   Nhập số cách nhau bởi dấu phẩy, ví dụ: 1,3,5");
        println!("   Gõ  *  để chọn tất cả");
        println!("   Enter để giữ nguyên");
        println!();
        let tho = self.nhap.dong("  Chọn")?;

        match phan_tich_chon_thu_muc(&tho, &tat_ca) {
            ChonThuMuc::GiuNguyen => println!("  Giữ nguyên."),
            ChonThuMuc::TatCa => {
                self.huy_ket_qua_quet();
                self.thu_muc.clear();
                println!("  Đã đặt: tất cả thư mục.");
            }
            ChonThuMuc::NhapSai(hong) => {
                println!();
                println!("  Giá trị không hợp lệ: {}", hong.join(", "));
                println!("  Chỉ nhận số từ 1 đến {}.", tat_ca.len());
                println!("  Bộ lọc giữ nguyên, không đổi gì để tránh xóa nhầm.");
            }
            ChonThuMuc::KhongChonDuocGi => println!("  Không chọn được gì. Giữ nguyên."),
            ChonThuMuc::Chon(chon) => {
                self.huy_ket_qua_quet();
                println!("  Đã đặt: {}", chon.join(", "));
                self.thu_muc = chon;
            }
        }
        Some(())
    }

    // ============================================================ vùng bảo vệ

    fn bao_cao_vung_bao_ve(&mut self) {
        hien::tieu_de("Vùng bảo vệ — chỉ báo cáo, không bao giờ xóa");
        if self.goc_du_lieu.trim().is_empty() || !Path::new(&self.goc_du_lieu).is_dir() {
            println!("  Không xác định được thư mục ZaloData.");
            return;
        }
        println!();
        for n in TEN_BAO_VE {
            let p = Path::new(&self.goc_du_lieu).join(n);
            if !p.is_dir() {
                continue;
            }
            let r = walk::duyet(&p);
            let b: u64 = r.tep.iter().map(|t| t.co).sum();
            println!(
                "   {} {} tệp · {}",
                hien::trai(n, 12),
                hien::phai(&hien::so(r.tep.len() as i64), 8),
                hien::co(b as i64)
            );
        }
        println!();
        println!("  Database   — cơ sở dữ liệu tin nhắn. Xóa là mất lịch sử chat vĩnh viễn.");
        println!("  Partitions — dữ liệu phiên đăng nhập. Xóa sẽ phải đăng nhập lại.");
        println!();
        println!("  Công cụ chặn cứng hai thư mục trên ở tầng code. Không bộ lọc nào,");
        println!("  kể cả bộ lọc do bạn tự đặt, chạm được vào chúng.");
        println!();
        println!("  Muốn thu gọn vùng này, hãy dùng chức năng quản lý dữ liệu trong chính");
        println!("  ứng dụng Zalo — nơi hiểu cấu trúc cơ sở dữ liệu của nó.");
        println!();
        println!("  ── Vùng bảo vệ ngoài Zalo");
        println!("  Mức  tất cả  chặn chính nó và mọi thứ bên dưới.");
        println!("  Mức  gốc     chỉ chặn khi nhắm thẳng vào chính thư mục; con vẫn dọn được.");
        println!();
        let mut sap = self.luat.clone();
        sap.sort_by_key(|r| matches!(r.muc, Muc::Goc));
        for r in &sap {
            let lv = match r.muc {
                Muc::TatCa => "tất cả",
                Muc::Goc => "gốc   ",
            };
            println!("   [{}]  {}", lv, r.duong_dan);
        }
        println!();
        println!("  Các mục mức  gốc  là lưới chắn cho catalog.json: một mục ghi nhầm");
        println!(
            "  \"%LOCALAPPDATA%\" sẽ bị loại, còn \"%LOCALAPPDATA%\\npm-cache\" vẫn dọn được."
        );
    }

    // ============================================================ khôi phục

    fn khoi_phuc(&mut self) -> Option<()> {
        hien::tieu_de("Khôi phục dữ liệu đã sao lưu");
        println!("  Đang tìm các bản sao lưu trên máy...");
        let mut bo = tim_ban_sao_luu(&self.cai_dat.goc_sao_luu, &sysinfo::cac_o_dia());

        if bo.is_empty() {
            println!();
            println!("  Không tìm thấy bản sao lưu nào do công cụ này tạo ra.");
            println!("  Công cụ đã tìm trong các thư mục từng dùng và các ổ đĩa.");
            println!();
            let tay = self
                .nhap
                .dong("  Nhập đường dẫn thủ công nếu bạn biết (Enter để quay lại)")?;
            if tay.is_empty() || !Path::new(&tay).exists() {
                return Some(());
            }
            let p = PathBuf::from(&tay);
            if let Some(s) = doc_bo_sao_luu(&p) {
                bo.push(s);
            }
            if let Ok(doc) = std::fs::read_dir(&p) {
                for e in doc.flatten() {
                    if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        if let Some(s) = doc_bo_sao_luu(&e.path()) {
                            bo.push(s);
                        }
                    }
                }
            }
            if bo.is_empty() {
                println!("  Vẫn không thấy bản sao lưu nào ở đó.");
                return Some(());
            }
        }

        println!();
        println!("  Tìm thấy {} bản sao lưu:", bo.len());
        println!();
        for (i, s) in bo.iter().enumerate() {
            let m = &s.ban_ke;
            println!(
                "   {}  {}   {} tệp · {}",
                i + 1,
                m.tao_luc,
                hien::so(m.so_tep),
                hien::co(m.so_byte)
            );
            println!("      Nội dung : {}", m.loai_quet);
            if let Some(tt) = tom_tat_sao_luu(s) {
                println!("      Gồm      : {}", tt.dau.join(" · "));
                println!("      Loại tệp : {}", tt.duoi.join(" · "));
                println!(
                    "      Tệp từ   : {} đến {}",
                    tt.cu_nhat.dinh_dang(),
                    tt.moi_nhat.dinh_dang()
                );
            }
            println!("      Nằm ở    : {}", s.thu_muc.display());
            println!("      Trả về   : {}", m.goc_nguon);
            if m.chep_hong > 0 || m.xac_minh_hong > 0 {
                println!(
                    "      Cảnh báo : bản này từng lỗi (chép {}, xác minh {})",
                    hien::so(m.chep_hong),
                    hien::so(m.xac_minh_hong)
                );
            }
            println!();
        }
        println!("   Gõ số để khôi phục, hoặc  x <số>  để xem danh sách tệp bên trong");
        println!("   Enter để quay lại");
        let tra_loi = self.nhap.dong("  Chọn")?;
        if tra_loi.is_empty() {
            println!("  Đã hủy.");
            return Some(());
        }

        let thap = tra_loi.to_lowercase();
        if let Some(phan) = thap.strip_prefix('x') {
            let k: usize = match phan.trim().parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("  Số không hợp lệ.");
                    return Some(());
                }
            };
            if k < 1 || k > bo.len() {
                println!("  Số không hợp lệ.");
                return Some(());
            }
            println!();
            println!("  Ba tệp lớn nhất trong bản {k}:");
            if let Some(tt) = tom_tat_sao_luu(&bo[k - 1]) {
                for m in &tt.mau {
                    println!("   {m}");
                }
            }
            println!();
            println!("  Toàn bộ nằm tại: {}", bo[k - 1].thu_muc.display());
            println!("  Mở thư mục đó bằng File Explorer để xem đầy đủ.");
            return Some(());
        }

        let n = match tra_loi.parse::<usize>() {
            Ok(n) if n >= 1 && n <= bo.len() => n,
            _ => {
                println!("  Số không hợp lệ. Đã hủy.");
                return Some(());
            }
        };
        let bo_chon = bo[n - 1].clone();
        let dich = bo_chon.ban_ke.goc_nguon.clone();

        println!();
        println!("  Nếu tệp đích đã tồn tại:");
        println!("   1  Bỏ qua, giữ tệp hiện có   (mặc định, an toàn nhất)");
        println!("   2  Ghi đè bằng bản sao lưu");
        let ghi_de = self.nhap.dong("  Chọn (Enter = 1)")? == "2";

        println!();
        println!("  Sẽ khôi phục về: {dich}");
        if !Path::new(&dich).is_dir() {
            println!("  Thư mục đích không còn tồn tại. Sẽ được tạo lại.");
        }

        // Đo chỗ trống TRƯỚC khi ghi byte đầu tiên. Khôi phục nửa chừng rồi hết
        // chỗ để lại một trạng thái dở dang mà người dùng không biết còn thiếu
        // gì — tệ hơn hẳn so với dừng hẳn và nói rõ thiếu bao nhiêu.
        println!("  Đang tính dung lượng cần thiết...");
        let (se_ghi, se_bo_qua, can) = do_cho_khoi_phuc(&bo_chon.thu_muc, &dich, ghi_de);
        println!();
        println!("  Sẽ ghi  : {} tệp", hien::so(se_ghi as i64));
        if se_bo_qua > 0 {
            println!("  Bỏ qua  : {} tệp đã tồn tại", hien::so(se_bo_qua as i64));
        }
        println!("  Cần chỗ : {}", hien::co(can));
        let trong = sysinfo::byte_trong(&dich);
        println!(
            "  Ổ {} trống: {}",
            sysinfo::nhan_o_dia(&dich),
            hien::co(trong)
        );
        if se_ghi == 0 {
            println!();
            println!("  Không có gì để khôi phục.");
            return Some(());
        }
        if trong >= 0 && trong < can {
            println!();
            println!(
                "  Không đủ chỗ trên ổ {}. Thiếu khoảng {}.",
                sysinfo::nhan_o_dia(&dich),
                hien::co(can - trong)
            );
            println!("  Chưa ghi tệp nào. Khôi phục nửa chừng rồi hết chỗ sẽ để lại trạng thái");
            println!("  dở dang, nên công cụ dừng hẳn thay vì làm liều.");
            return Some(());
        }
        println!("  Đủ chỗ.");

        if !self.chan_khi_zalo_dang_chay(&dich) {
            println!("  Dừng lại, chưa khôi phục gì.");
            return Some(());
        }

        let k = match zalo_core::act::khoi_phuc(
            &bo_chon.thu_muc,
            &dich,
            ghi_de,
            &self.thu_muc_nhat_ky(),
        ) {
            Ok(k) => k,
            Err(e) => {
                println!("  Không khôi phục được: {e}");
                return Some(());
            }
        };
        self.cay = None;
        println!();
        if k.het_cho {
            println!("  Dừng sớm: ổ đích hết chỗ giữa chừng.");
            println!(
                "  Đã khôi phục {} tệp trước khi hết chỗ.",
                hien::so(k.da_khoi_phuc as i64)
            );
            println!("  Bản sao lưu vẫn nguyên vẹn. Giải phóng thêm chỗ rồi chạy lại là an toàn.");
        } else {
            println!("  Đã khôi phục: {} tệp", hien::so(k.da_khoi_phuc as i64));
        }
        if k.bo_qua > 0 {
            println!("  Bỏ qua (đã có): {} tệp", hien::so(k.bo_qua as i64));
        }
        if k.that_bai > 0 {
            println!("  Thất bại      : {} tệp", hien::so(k.that_bai as i64));
        }
        println!("  Nhật ký: {}", k.tep_nhat_ky.display());
        Some(())
    }

    /// Zalo đang chạy mà lượt này chạm vào dữ liệu của nó thì **dừng lại**.
    ///
    /// Trả `true` nghĩa là đi tiếp được. Xem chú thích ở
    /// [`zalo_core::sysinfo::tien_trinh_dang_chay`] về chỗ bản này cố ý khác bản
    /// PowerShell: nó **không tự đóng Zalo**, chỉ báo rồi dừng.
    fn chan_khi_zalo_dang_chay(&self, duong_dan: &str) -> bool {
        let nen = match std::env::var("APPDATA") {
            Ok(a) if !a.trim().is_empty() => Path::new(&a).join("ZaloData"),
            _ => return true,
        };
        if !duong_dan
            .to_lowercase()
            .starts_with(&nen.to_string_lossy().to_lowercase())
        {
            return true;
        }
        let p = sysinfo::tien_trinh_dang_chay("Zalo");
        if p.is_empty() {
            return true;
        }
        println!();
        println!(
            "  Zalo đang chạy ({} tiến trình). Cần đóng trước khi thao tác.",
            p.len()
        );
        println!("  Hãy tự đóng Zalo rồi chạy lại — bản này không tự tắt ứng dụng của bạn.");
        false
    }

    // ============================================================ xóa

    /// Đi hết đường hỏi và xác nhận rồi **dừng lại**. Xem chú thích đầu tệp.
    fn xoa(&mut self) -> Option<()> {
        let ds = match &self.quet {
            Some(v) if !v.is_empty() => v.clone(),
            _ => {
                println!("Chưa có kết quả quét.");
                return Some(());
            }
        };
        let tong: u64 = ds.iter().map(|t| t.co).sum();
        let du_lieu_that = muc_xac_nhan(&self.loai_quet) == MucXacNhan::GoCumTu;

        hien::tieu_de(&format!("Xóa · {}", self.loai_quet));
        println!("  Số tệp     : {}", hien::so(ds.len() as i64));
        println!("  Dung lượng : {}", hien::co(tong as i64));

        if self.loai_quet == "BẢN TRÙNG LẶP" {
            println!();
            println!("  Mỗi tệp đã được xác minh bằng SHA256 là giống hệt một bản khác đang");
            println!("  được giữ lại. Bạn sẽ không mất tấm ảnh hay đoạn video nào.");
        } else if self.loai_quet == "CACHE ZALO" {
            println!();
            println!("  Cache của ứng dụng. Zalo tự tạo lại khi cần.");
        } else {
            println!(
                "  Khoảng thời gian : {}",
                nhan_khoang(self.tu_ngay, self.den_ngay)
            );
            println!();
            println!("  Đây là dữ liệu thật. Tệp sẽ bị xóa hẳn, không qua Thùng rác,");
            println!("  không khôi phục được. Ảnh và video quá hạn lưu trên máy chủ Zalo");
            println!("  sẽ mất vĩnh viễn.");
        }

        // Chính sách sao lưu chỉ áp dụng cho dữ liệu thật.
        if du_lieu_that
            && self.cai_dat.chinh_sach == zalo_core::store::ChinhSach::BatBuoc
            && !self.sao_luu_sach()
        {
            println!();
            println!("  Đã chặn: chính sách hiện tại là bắt buộc sao lưu.");
            println!("  Hãy sao lưu trong Tùy chọn nâng cao rồi quay lại xóa.");
            return Some(());
        }
        if du_lieu_that
            && self.cai_dat.chinh_sach == zalo_core::store::ChinhSach::Hoi
            && !self.co_sao_luu_cua_lan_quet_nay()
        {
            println!();
            println!("  Kết quả này chưa được sao lưu. Sao lưu là cách duy nhất để còn đường lui.");
            println!();
            println!("   1  Sao lưu trước rồi xóa");
            println!("   2  Xóa luôn, không sao lưu");
            println!("   Enter để hủy");
            let bc = self.nhap.dong("  Chọn")?;
            if bc == "1" {
                self.sao_luu()?;
                if !self.co_sao_luu_cua_lan_quet_nay() {
                    println!();
                    println!("  Chưa sao lưu được. Dừng lại, chưa xóa gì.");
                    return Some(());
                }
            } else if bc == "2" {
                println!("  Đã chọn xóa mà không sao lưu.");
            } else {
                println!("  Đã hủy. Không tệp nào bị đụng đến.");
                return Some(());
            }
        }

        // Sao lưu có mà KHÔNG sạch thì phải xác nhận nặng hơn hẳn. Xóa lúc này
        // là mất vĩnh viễn đúng những tệp chưa được sao lưu tốt.
        if let Some(b) = self.sao_luu_gan_nhat.clone() {
            if b.dau_quet == self.dau_quet && (b.chep_hong > 0 || b.xac_minh_hong > 0) {
                println!();
                println!(
                    "  Đã chặn: sao lưu chưa sạch (chép lỗi {}, xác minh lỗi {}).",
                    hien::so(b.chep_hong as i64),
                    hien::so(b.xac_minh_hong as i64)
                );
                println!("  Xóa bây giờ sẽ mất vĩnh viễn đúng những tệp chưa được sao lưu tốt.");
                println!();
                let tl = self
                    .nhap
                    .dong("  Vẫn muốn xóa? Gõ chính xác:  TÔI CHẤP NHẬN MẤT")?;
                if !zalo_core::confirm::khop_cum_xac_nhan(
                    &tl,
                    "TÔI CHẤP NHẬN MẤT",
                    "TOI CHAP NHAN MAT",
                ) {
                    println!("  Đã hủy. Không tệp nào bị đụng đến.");
                    return Some(());
                }
            }
        }

        println!();
        if du_lieu_that {
            let tra_loi = self.nhap.dong("  Gõ đúng chữ  XÓA  để xác nhận")?;
            if !zalo_core::confirm::khop_cum_xac_nhan(&tra_loi, "XÓA", "XOA") {
                println!("  Đã hủy. Không tệp nào bị đụng đến.");
                return Some(());
            }
        } else if !self.nhap.co_khong("  Xóa luôn? (c/k)")? {
            println!("  Đã hủy. Không tệp nào bị đụng đến.");
            return Some(());
        }

        if self.cham_toi_zalo() && !self.chan_khi_zalo_dang_chay(&self.goc_quet.clone()) {
            println!("  Dừng lại, chưa xóa gì.");
            return Some(());
        }

        let trong_truoc = sysinfo::byte_trong(&self.goc_quet);
        println!();
        println!("  Đang xóa. Có thể bấm Ctrl+C để dừng an toàn bất cứ lúc nào.");

        let dich_sl = self
            .sao_luu_gan_nhat
            .as_ref()
            .filter(|b| b.dau_quet == self.dau_quet)
            .map(|b| b.thu_muc.clone());
        let r = match zalo_core::act::xoa(
            &ds,
            &self.vbv,
            &self.loai_quet,
            &self.dau_quet,
            dich_sl.as_deref(),
            !du_lieu_that && self.loai_quet != "BẢN TRÙNG LẶP",
            &self.thu_muc_nhat_ky(),
        ) {
            Ok(r) => r,
            Err(e) => {
                println!("  Không mở được nhật ký nên đã dừng, chưa xóa gì: {e}");
                return Some(());
            }
        };

        let mut goc_don = self.goc_don_dep.clone();
        if goc_don.is_empty() {
            goc_don.push(self.goc_quet.clone());
        }
        let so_thu_muc = zalo_core::act::don_thu_muc_rong(
            &goc_don,
            self.loai_quet != "CACHE HỆ THỐNG",
            &self.vbv,
        );

        self.cay = None;
        println!();
        println!("  Đã xóa       : {} tệp", hien::so(r.da_xoa as i64));
        println!("  Giải phóng   : {}", hien::co(r.byte_thu_hoi as i64));
        println!("  Thư mục rỗng : {}", hien::so(so_thu_muc as i64));
        if r.bien_mat > 0 {
            println!(
                "  Biến mất trước khi xóa: {} tệp (tiến trình khác đã xóa)",
                hien::so(r.bien_mat as i64)
            );
        }
        if r.vung_bao_ve > 0 {
            println!(
                "  Chặn bởi vùng bảo vệ  : {} tệp",
                hien::so(r.vung_bao_ve as i64)
            );
        }
        if r.mat_ban_goc > 0 {
            println!(
                "  Giữ lại vì mất bản gốc: {} tệp",
                hien::so(r.mat_ban_goc as i64)
            );
            println!("  Bản gốc của những tệp đó đã biến mất hoặc đã đổi kể từ lúc quét,");
            println!("  nên chúng không còn là bản thừa nữa. Quét lại để có kết quả đúng.");
        }
        if r.cat_cut > 0 {
            println!(
                "  Cắt cụt      : {} tệp đang bị khóa",
                hien::so(r.cat_cut as i64)
            );
            println!("                 Tên tệp còn đó nhưng nội dung đã rỗng, dung lượng đã được thu về.");
        }
        if r.that_bai > 0 {
            println!(
                "  Thất bại     : {} tệp (đang bị khóa)",
                hien::so(r.that_bai as i64)
            );
            for e in r.loi.iter().take(5) {
                println!("    {e}");
            }
        }
        println!("  Nhật ký      : {}", r.tep_nhat_ky.display());

        let trong_sau = sysinfo::byte_trong(&self.goc_quet);
        if trong_truoc >= 0 && trong_sau >= 0 {
            let thu = trong_sau - trong_truoc;
            let txt = if thu >= 0 {
                format!("+{}", hien::co(thu))
            } else {
                format!("−{}", hien::co(-thu))
            };
            println!();
            println!("  Ổ đĩa trước  : {}", hien::co(trong_truoc));
            println!("  Ổ đĩa sau    : {}", hien::co(trong_sau));
            println!("  Thực tế thu được: {txt}");

            // R-12: dung lượng phải đo bằng chỗ trống THẬT của ổ đĩa. Xóa nhiều
            // mà ổ không rộng thêm gần như chắc chắn là Volume Shadow Copy đang
            // giữ lại khối cũ — dung lượng đổi chủ chứ không về với người dùng.
            if r.byte_thu_hoi > 500 * 1024 * 1024 && thu < (r.byte_thu_hoi as i64) / 2 {
                println!();
                println!("  ─── Cảnh báo: chưa thu được dung lượng thật ───");
                println!(
                    "  Đã xóa {} nhưng ổ đĩa chỉ rộng thêm {txt}.",
                    hien::co(r.byte_thu_hoi as i64)
                );
                println!("  Nguyên nhân gần như chắc chắn là Shadow Copy của System Restore.");
            }
        }

        // Nguyên tắc bất biến số 1: xóa xong thì kết quả quét hết hiệu lực.
        self.huy_ket_qua_quet();
        Some(())
    }

    // ============================================================ sao lưu

    fn sao_luu(&mut self) -> Option<()> {
        let ds = match &self.quet {
            Some(v) if !v.is_empty() => v.clone(),
            _ => {
                println!("Chưa có kết quả quét.");
                return Some(());
            }
        };
        let tong: u64 = ds.iter().map(|t| t.co).sum();
        let goc = if self.goc_quet.trim().is_empty() {
            self.goc.clone()
        } else {
            self.goc_quet.clone()
        };

        hien::tieu_de("Sao lưu và xác minh");
        for o in sysinfo::cac_o_dia() {
            println!(
                "   Ổ {}  trống {}",
                sysinfo::nhan_o_dia(&o),
                hien::co(sysinfo::byte_trong(&o))
            );
        }
        println!("  Cần ít nhất: {}", hien::co(tong as i64));

        let dich = self
            .nhap
            .dong("  Nhập thư mục đích (ví dụ D:\\SaoLuuZalo)")?;
        if dich.is_empty() {
            println!("  Đã hủy.");
            return Some(());
        }
        let trong = sysinfo::byte_trong(&dich);
        if trong < 0 {
            println!("  Đường dẫn đích không hợp lệ: không đọc được ổ đĩa.");
            return Some(());
        }
        // Chừa 2% cộng 100 MB: hệ tệp còn siêu dữ liệu, và một bản sao lưu vừa
        // khít ổ đĩa là một bản sao lưu sắp hỏng.
        let can = (tong as f64 * 1.02) as i64 + 100 * 1024 * 1024;
        println!();
        println!(
            "  Ổ {} trống {}, cần {}",
            sysinfo::nhan_o_dia(&dich),
            hien::co(trong),
            hien::co(can)
        );
        if trong < can {
            println!();
            println!("  Không đủ chỗ. Đã dừng, chưa chép tệp nào.");
            println!(
                "  Thiếu khoảng {}. Chọn ổ khác hoặc thu hẹp phạm vi.",
                hien::co(can - trong)
            );
            return Some(());
        }
        println!("  Đủ chỗ.");

        println!();
        println!("  Mức xác minh sau khi chép:");
        println!("   1  Kích thước toàn bộ, cộng SHA256 mẫu 50 tệp  (nhanh, mặc định)");
        println!(
            "   2  SHA256 toàn bộ                              (chậm nhưng chắc chắn tuyệt đối)"
        );
        let toan_bo = self.nhap.dong("  Chọn (Enter = 1)")? == "2";

        let dau = zalo_core::thoigian::luc_nay().dau_thoi_gian();
        let thu_muc = Path::new(&dich).join(&dau);
        let r = match zalo_core::act::sao_luu(&ds, &goc, &thu_muc, toan_bo) {
            Ok(r) => r,
            Err(e) => {
                println!("  Không sao lưu được: {e}");
                return Some(());
            }
        };
        let _ = zalo_core::act::ghi_loai_quet(&thu_muc, &self.loai_quet);
        self.nho_goc_sao_luu(&dich);

        if r.het_cho {
            println!();
            println!("  Ổ đích hết chỗ giữa chừng. Đã dừng ngay tại tệp đó.");
            println!(
                "  Chép được {}/{} tệp trước khi hết chỗ.",
                hien::so(r.da_chep as i64),
                hien::so(r.tong as i64)
            );
            println!("  Bản sao lưu này KHÔNG trọn vẹn nên không mở đường xóa.");
        }

        self.sao_luu_gan_nhat = Some(SaoLuuGanNhat {
            dau_quet: self.dau_quet.clone(),
            thu_muc: thu_muc.to_string_lossy().to_string(),
            tong: r.tong,
            da_chep: r.da_chep,
            chep_hong: r.chep_hong,
            xac_minh_hong: r.xac_minh_hong,
            het_cho: r.het_cho,
        });

        println!();
        println!("  Đã chép   : {} tệp", hien::so(r.da_chep as i64));
        // Nói đúng ĐỘ PHỦ, đừng gộp thành một chữ "đã xác minh": kích thước được
        // đối chiếu cho 100% số tệp đã chép, còn SHA-256 thì chỉ cho phần đã băm.
        println!(
            "  Xác minh  : kích thước {}/{} tệp · SHA256 {}/{} tệp",
            hien::so(r.da_chep as i64),
            hien::so(r.da_chep as i64),
            hien::so(r.da_xac_minh as i64),
            hien::so(r.da_chep as i64)
        );
        println!("  Vị trí    : {}", thu_muc.display());

        if r.chep_hong > 0 || r.xac_minh_hong > 0 || r.het_cho || r.da_chep != r.tong {
            let lf = self.thu_muc_nhat_ky().join(format!("saoluu_loi_{dau}.txt"));
            let _ = std::fs::create_dir_all(self.thu_muc_nhat_ky());
            let _ = std::fs::write(&lf, r.nhat_ky_loi.join("\r\n"));
            println!();
            if r.chep_hong > 0 {
                println!("  Chép lỗi     : {} tệp", hien::so(r.chep_hong as i64));
            }
            if r.xac_minh_hong > 0 {
                println!("  Xác minh lỗi : {} tệp", hien::so(r.xac_minh_hong as i64));
            }
            if r.da_chep != r.tong {
                println!(
                    "  Thiếu        : {}/{} tệp chưa được chép",
                    hien::so((r.tong - r.da_chep) as i64),
                    hien::so(r.tong as i64)
                );
            }
            println!("  Bước xóa sẽ bị chặn cho đến khi sao lưu lại sạch.");
            println!("  Chi tiết: {}", lf.display());
        } else if toan_bo {
            println!("  Sao lưu sạch — đã đối chiếu SHA256 toàn bộ. Đã mở khóa bước xóa.");
        } else {
            println!(
                "  Sao lưu sạch ở mức đã kiểm: kích thước 100%, SHA256 {}/{} tệp.",
                hien::so(r.da_xac_minh as i64),
                hien::so(r.da_chep as i64)
            );
            println!("  Phần chưa băm có thể hỏng nội dung mà kích thước vẫn đúng.");
            println!("  Đã mở khóa bước xóa.");
        }
        Some(())
    }

    /// Nhớ thư mục sao lưu để lần sau khỏi phải nhớ đường dẫn.
    fn nho_goc_sao_luu(&mut self, dich: &str) {
        if dich.trim().is_empty() || self.cai_dat.goc_sao_luu.iter().any(|x| x == dich) {
            return;
        }
        self.cai_dat.goc_sao_luu.push(dich.to_string());
        let _ = zalo_core::store::ghi_cai_dat(
            &sysinfo::thu_muc_cong_cu().join("settings.json"),
            &self.cai_dat,
        );
    }
}

/// Đếm trước một lượt khôi phục sẽ ghi bao nhiêu tệp, bỏ qua bao nhiêu, và cần
/// bao nhiêu byte.
///
/// Tệp đã tồn tại mà có ghi đè thì chỉ cần thêm **phần chênh lệch**, không phải
/// cả cỡ tệp — đòi dư chỗ là từ chối một lượt khôi phục hoàn toàn làm được.
/// Chừa 2% cộng 50 MB cho siêu dữ liệu hệ tệp.
fn do_cho_khoi_phuc(thu_muc_sao_luu: &Path, dich: &str, ghi_de: bool) -> (usize, usize, i64) {
    let goc = thu_muc_sao_luu.to_string_lossy().to_string();
    let mut se_ghi = 0usize;
    let mut bo_qua = 0usize;
    let mut can: i64 = 0;
    for t in zalo_core::walk::duyet(thu_muc_sao_luu).tep {
        let s = t.duong_dan.to_string_lossy().to_string();
        if t.duong_dan
            .file_name()
            .map(|n| n == zalo_core::store::TEN_BAN_KE)
            == Some(true)
        {
            continue;
        }
        let dst = Path::new(dich).join(duong_dan_tuong_doi(&s, &goc));
        match std::fs::metadata(&dst) {
            Ok(md) => {
                if !ghi_de {
                    bo_qua += 1;
                    continue;
                }
                let them = t.co as i64 - md.len() as i64;
                if them > 0 {
                    can += them;
                }
                se_ghi += 1;
            }
            Err(_) => {
                can += t.co as i64;
                se_ghi += 1;
            }
        }
    }
    (
        (se_ghi),
        bo_qua,
        (can as f64 * 1.02) as i64 + 50 * 1024 * 1024,
    )
}

/// Gắn bản giữ lại vào một ứng viên đã xác minh là trùng.
///
/// Bản giữ lại đi theo tệp tới tận lúc xóa, vì đó là thứ
/// [`zalo_core::gate::ban_giu_lai_con_song`] kiểm lại ngay trước khi hạ tay.
fn gan_ban_giu_lai(c: &TepQuet, giu: &str) -> TepQuet {
    TepQuet {
        duong_dan: c.duong_dan.clone(),
        co: c.co,
        giu_lai: giu.to_string(),
    }
}

/// Số phận của một tệp trong lượt quét. Ba ngã, và **chỉ** ba.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XetTep {
    /// Thuộc vùng bảo vệ. Bị chặn cứng, và được ĐẾM để báo cho người dùng biết.
    Chan,
    /// Không qua bộ lọc. Bình thường, không đếm.
    LoaiBoiBoLoc,
    /// Vào kết quả quét.
    Nhan,
}

#[derive(Default)]
struct MocTuoi {
    b12: u64,
    n12: usize,
    b6: u64,
    n6: usize,
    b_truoc: u64,
    n_truoc: usize,
    b_tong: u64,
    n_tong: usize,
}

struct TomTat {
    dau: Vec<String>,
    duoi: Vec<String>,
    cu_nhat: Ngay,
    moi_nhat: Ngay,
    mau: Vec<String>,
}

/// Mô tả bên trong bản sao lưu có gì. Tương ứng `Get-BackupSummary`.
fn tom_tat_sao_luu(bo: &BoSaoLuu) -> Option<TomTat> {
    let r = walk::duyet(&bo.thu_muc);
    let tep: Vec<&walk::Tep> = r
        .tep
        .iter()
        .filter(|t| {
            t.duong_dan
                .file_name()
                .map(|n| n != TEN_BAN_KE)
                .unwrap_or(true)
        })
        .collect();
    if tep.is_empty() {
        return None;
    }
    let goc = bo.thu_muc.to_string_lossy().to_string();

    let mut theo_dau: HashMap<String, (usize, u64)> = HashMap::new();
    let mut theo_duoi: HashMap<String, (usize, u64)> = HashMap::new();
    for t in &tep {
        let s = t.duong_dan.to_string_lossy().to_string();
        let e = theo_dau
            .entry(thu_muc_cap_mot(&s, &goc).to_string())
            .or_default();
        e.0 += 1;
        e.1 += t.co;
        let d = duoi_kieu_dotnet(ten_tep(&s)).to_lowercase();
        let d = if d.is_empty() {
            "không đuôi".to_string()
        } else {
            d
        };
        let e = theo_duoi.entry(d).or_default();
        e.0 += 1;
        e.1 += t.co;
    }
    let sap = |m: HashMap<String, (usize, u64)>| -> Vec<(String, usize, u64)> {
        let mut v: Vec<(String, usize, u64)> = m.into_iter().map(|(k, (n, b))| (k, n, b)).collect();
        v.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
        v
    };

    let mut mau: Vec<&&walk::Tep> = tep.iter().collect();
    mau.sort_by(|a, b| b.co.cmp(&a.co));

    Some(TomTat {
        dau: sap(theo_dau)
            .into_iter()
            .take(3)
            .map(|(k, _, b)| format!("{} {}", k, hien::co(b as i64)))
            .collect(),
        duoi: sap(theo_duoi)
            .into_iter()
            .take(3)
            .map(|(k, n, _)| format!("{} ({})", k, hien::so(n as i64)))
            .collect(),
        cu_nhat: ngay_dia_phuong(tep.iter().map(|t| t.sua_luc).min().unwrap()),
        moi_nhat: ngay_dia_phuong(tep.iter().map(|t| t.sua_luc).max().unwrap()),
        mau: mau
            .into_iter()
            .take(3)
            .map(|t| {
                let s = t.duong_dan.to_string_lossy().to_string();
                format!(
                    "{}  [{}]",
                    duong_dan_tuong_doi(&s, &goc),
                    hien::co(t.co as i64)
                )
            })
            .collect(),
    })
}

/// Thư mục cấp một của một đường dẫn so với gốc; ở ngay gốc thì trả `(gốc)`.
fn thu_muc_cap_mot<'a>(duong_dan: &'a str, goc: &str) -> &'a str {
    let rel = duong_dan_tuong_doi(duong_dan, goc);
    match rel.find('\\') {
        Some(i) => &rel[..i],
        None => zalo_core::scan::O_GOC,
    }
}

fn thu_muc_con(goc: &str) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    if let Ok(doc) = std::fs::read_dir(goc) {
        for e in doc.flatten() {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                v.push(e.file_name().to_string_lossy().to_string());
            }
        }
    }
    v.sort();
    v
}

fn ngay_ke_tiep(n: Ngay) -> Ngay {
    zalo_core::thoigian::lich_tu_ngay(zalo_core::thoigian::ngay_tu_lich(n.nam, n.thang, n.ngay) + 1)
}

fn nhan_ngay(n: Option<Ngay>) -> String {
    match n {
        None => "không giới hạn".into(),
        Some(d) => d.dinh_dang(),
    }
}

/// Tương ứng `Show-DateRangeLabel`.
fn nhan_khoang(tu: Option<Ngay>, den: Option<Ngay>) -> String {
    match (tu, den) {
        (None, None) => "mọi thời điểm".into(),
        (None, Some(d)) => format!("đến {}", d.dinh_dang()),
        (Some(f), None) => format!("từ {}", f.dinh_dang()),
        (Some(f), Some(d)) => format!("{} → {}", f.dinh_dang(), d.dinh_dang()),
    }
}

/// Đưa đường dẫn về dạng chuẩn. Tương ứng `Get-CanonPath`.
///
/// Vùng bảo vệ so bằng **chuỗi**, nên một đường dẫn dạng ngắn 8.3 ở đây là vùng
/// bảo vệ biến mất không một lời cảnh báo. Lỗi này do máy chủ CI tìm ra ở bản
/// PowerShell — `%TEMP%` trên đó là dạng ngắn, và bốn phép thử vùng bảo vệ đỏ
/// ngay lần chạy đầu trong khi trên máy phát triển chúng vẫn xanh.
fn canon(p: &str) -> String {
    if p.trim().is_empty() {
        return p.to_string();
    }
    let day_du = std::fs::canonicalize(p)
        .map(|c| {
            let s = c.to_string_lossy().to_string();
            // `canonicalize` trả về dạng `\\?\C:\...`; bản PowerShell không có
            // tiền tố đó và vùng bảo vệ so chuỗi, nên phải bỏ đi.
            s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
        })
        .unwrap_or_else(|_| p.to_string());
    if day_du.len() == 3 && day_du.as_bytes()[1] == b':' {
        return day_du;
    }
    day_du.trim_end_matches('\\').to_string()
}

/// Dựng bộ luật vùng bảo vệ. Phải khớp `Initialize-ProtectedAbs` từng mục.
fn dung_luat_bao_ve(thu_muc_cong_cu: &Path) -> Vec<Luat> {
    let bien = |t: &str| std::env::var(t).unwrap_or_default();
    let w = bien("WINDIR");
    let u = bien("USERPROFILE");
    let l = bien("LOCALAPPDATA");
    let a = bien("APPDATA");
    let pf = bien("ProgramFiles");
    let px = bien("ProgramFiles(x86)");
    let pd = bien("ProgramData");
    let goc_ht = sysinfo::goc_he_thong();

    let noi = |g: &str, p: &str| -> String {
        if g.is_empty() {
            String::new()
        } else {
            Path::new(g).join(p).to_string_lossy().to_string()
        }
    };

    let mut tat_ca: Vec<String> = vec![
        noi(&w, "WinSxS"),
        noi(&w, "Installer"),
        noi(&w, "System32"),
        noi(&w, "SysWOW64"),
        noi(&w, "servicing"),
        noi(&w, "assembly"),
        noi(&goc_ht, "hiberfil.sys"),
        noi(&goc_ht, "pagefile.sys"),
        noi(&goc_ht, "swapfile.sys"),
        noi(&a, r"Claude\vm_bundles"),
        noi(&u, r".cargo\bin"),
        noi(&u, ".rustup"),
        noi(&l, "Programs"),
        noi(&l, "Packages"),
        thu_muc_cong_cu.to_string_lossy().to_string(),
    ];
    tat_ca.retain(|s| !s.trim().is_empty());

    let mut luat: Vec<Luat> = tat_ca
        .iter()
        .map(|p| Luat {
            duong_dan: p.trim_end_matches('\\').to_string(),
            muc: Muc::TatCa,
        })
        .collect();

    for p in [&w, &u, &l, &a, &pf, &px, &pd, &goc_ht] {
        if p.trim().is_empty() {
            continue;
        }
        let mut t = p.trim_end_matches('\\').to_string();
        // Gốc ổ đĩa giữ dấu gạch chéo, nếu không `C:` sẽ chặn nhầm `C:foo`.
        if t.len() == 2 && t.as_bytes()[1] == b':' {
            t.push('\\');
        }
        if luat.iter().any(|r| r.duong_dan == t) {
            continue;
        }
        luat.push(Luat {
            duong_dan: t,
            muc: Muc::Goc,
        });
    }
    luat
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nguyên tắc bất biến số 3. Cổng M3 **không** phủ được — nó chỉ kiểm câu
    /// chữ in ra, không kiểm bộ lọc có thật sự giữ nguyên không.
    #[test]
    fn nhap_sai_thi_giu_nguyen_chu_khong_bao_gio_tu_mo_rong() {
        let tm: Vec<String> = ["video", "picture", "resource"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        // Số ngoài khoảng.
        assert_eq!(
            phan_tich_chon_thu_muc("99", &tm),
            ChonThuMuc::NhapSai(vec!["99".into()])
        );
        // Không phải số.
        assert_eq!(
            phan_tich_chon_thu_muc("abc", &tm),
            ChonThuMuc::NhapSai(vec!["abc".into()])
        );
        // Số 0 không hợp lệ: danh sách đánh từ 1.
        assert_eq!(
            phan_tich_chon_thu_muc("0", &tm),
            ChonThuMuc::NhapSai(vec!["0".into()])
        );
        // MỘT giá trị hỏng làm hỏng CẢ lượt — không nhận phần đúng rồi bỏ phần sai.
        assert_eq!(
            phan_tich_chon_thu_muc("1,99", &tm),
            ChonThuMuc::NhapSai(vec!["99".into()])
        );
    }

    #[test]
    fn chon_thu_muc_cac_nga_con_lai() {
        let tm: Vec<String> = ["video", "picture", "resource"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(phan_tich_chon_thu_muc("", &tm), ChonThuMuc::GiuNguyen);
        assert_eq!(phan_tich_chon_thu_muc("*", &tm), ChonThuMuc::TatCa);
        assert_eq!(
            phan_tich_chon_thu_muc(" , ,", &tm),
            ChonThuMuc::KhongChonDuocGi
        );
        assert_eq!(
            phan_tich_chon_thu_muc("1,3", &tm),
            ChonThuMuc::Chon(vec!["video".into(), "resource".into()])
        );
        assert_eq!(
            phan_tich_chon_thu_muc(" 2 ", &tm),
            ChonThuMuc::Chon(vec!["picture".into()]),
            "khoảng trắng quanh số phải được bỏ qua"
        );
    }

    /// Chốt này cổng M3 **không** phủ được — xem chú thích ở [`muc_xac_nhan`].
    #[test]
    fn du_lieu_that_doi_go_cum_tu_con_cai_lay_lai_duoc_thi_khong() {
        assert_eq!(muc_xac_nhan("DỮ LIỆU ZALO"), MucXacNhan::GoCumTu);
        assert_eq!(muc_xac_nhan("BẢN TRÙNG LẶP"), MucXacNhan::CoKhong);
        assert_eq!(muc_xac_nhan("CACHE ZALO"), MucXacNhan::CoKhong);
        assert_eq!(muc_xac_nhan("CACHE HỆ THỐNG"), MucXacNhan::CoKhong);
    }

    /// Loại quét lạ phải ngã về phía **nặng**. Đây là toàn bộ lý do hàm liệt kê
    /// danh sách nhẹ chứ không liệt kê danh sách nặng.
    #[test]
    fn loai_quet_la_nga_ve_phia_nang() {
        assert_eq!(muc_xac_nhan(""), MucXacNhan::GoCumTu);
        assert_eq!(muc_xac_nhan("MỘT LOẠI AI ĐÓ THÊM SAU"), MucXacNhan::GoCumTu);
        // Sai hoa thường cũng là "lạ" — không được đoán ý.
        assert_eq!(muc_xac_nhan("cache zalo"), MucXacNhan::GoCumTu);
    }

    /// Vùng bảo vệ phải bị chặn NGAY TRONG vòng quét, và phải được ĐẾM.
    ///
    /// Cổng M3 không phủ được ngã này — mọi phép thử đầu-cuối chạm tới vùng bảo
    /// vệ đều là phép thử có xóa tệp, tức thuộc M4. Đã đo bằng đột biến: gỡ hẳn
    /// chốt ra khỏi vòng quét mà cổng vẫn xanh.
    #[test]
    fn vung_bao_ve_bi_chan_ngay_trong_vong_quet_va_duoc_dem() {
        let tam = std::env::temp_dir().join(format!("zm3_{}", std::process::id()));
        let du_lieu = tam.join("ZaloData");
        std::fs::create_dir_all(du_lieu.join("Database")).unwrap();
        std::fs::create_dir_all(du_lieu.join("media/acc/ZaloDownloads/video")).unwrap();
        let goc = du_lieu.join("media/acc/ZaloDownloads");

        let mut app = UngDung::moi(
            goc.to_string_lossy().to_string(),
            du_lieu.to_string_lossy().to_string(),
        );
        app.giu_rescache = false;

        let bay_gio = std::time::SystemTime::now();
        // Dựng đường dẫn từ GỐC ĐÃ CHUẨN HÓA của chính ứng dụng, không từ biến
        // cục bộ ở trên.
        //
        // Máy chủ CI có %TEMP% dạng ngắn 8.3 (`C:\Users\RUNNE~1\...`), nên biến
        // `tam` ở trên là dạng ngắn còn `app.goc_du_lieu` là dạng dài — và phép
        // thử này đỏ trên CI trong khi vẫn xanh trên máy phát triển. Vòng quét
        // thật không bao giờ gặp cảnh đó vì mọi đường dẫn nó xét đều do bộ duyệt
        // sinh ra từ chính cái gốc đã chuẩn hóa. Đây là phép thử sai, không phải
        // công cụ sai — nhưng nó sai theo đúng cách mà một lỗi thật cũng sẽ sai,
        // nên phải dựng lại cho đúng chứ không nới điều kiện cho qua.
        //
        // Việc canon có thật sự mở được tên 8.3 hay không thì có phép thử riêng
        // ngay bên dưới.
        let trong_vung = Path::new(&app.goc_du_lieu).join("Database").join("chat.db");
        let ngoai_vung = Path::new(&app.goc).join("video").join("v1");

        assert_eq!(
            app.xet_tep(&trong_vung.to_string_lossy(), 100, bay_gio, None, None, 0),
            XetTep::Chan,
            "tệp trong Database KHÔNG bị chặn — vùng bảo vệ đã mất tác dụng"
        );
        assert_eq!(
            app.xet_tep(&ngoai_vung.to_string_lossy(), 100, bay_gio, None, None, 0),
            XetTep::Nhan,
            "tệp thường lại bị chặn — vùng bảo vệ đang quá tay"
        );

        // Bị chặn phải THẮNG mọi bộ lọc khác, kể cả bộ lọc lẽ ra đã loại nó.
        assert_eq!(
            app.xet_tep(
                &trong_vung.to_string_lossy(),
                1,
                bay_gio,
                None,
                None,
                999_999_999
            ),
            XetTep::Chan,
            "vùng bảo vệ phải được hỏi TRƯỚC bộ lọc, nếu không con số bị chặn sẽ đếm hụt"
        );

        let _ = std::fs::remove_dir_all(&tam);
    }

    #[test]
    fn thu_muc_cap_mot_nhan_dung_ca_hai_ca() {
        assert_eq!(thu_muc_cap_mot(r"C:\z\video\a.bin", r"C:\z"), "video");
        assert_eq!(thu_muc_cap_mot(r"C:\z\a.bin", r"C:\z"), "(gốc)");
    }

    #[test]
    fn nhan_khoang_doc_duoc_o_ca_bon_ca() {
        let a = Ngay {
            nam: 2025,
            thang: 1,
            ngay: 2,
        };
        let b = Ngay {
            nam: 2026,
            thang: 3,
            ngay: 4,
        };
        assert_eq!(nhan_khoang(None, None), "mọi thời điểm");
        assert_eq!(nhan_khoang(None, Some(b)), "đến 04/03/2026");
        assert_eq!(nhan_khoang(Some(a), None), "từ 02/01/2025");
        assert_eq!(nhan_khoang(Some(a), Some(b)), "02/01/2025 → 04/03/2026");
    }

    /// Cận trên của bộ lọc là HẾT ngày đó. Chọn "đến 31/12" mà mất sạch tệp của
    /// chính ngày 31/12 là một cái bẫy im lặng.
    #[test]
    fn ngay_ke_tiep_qua_ranh_gioi_nam_va_thang() {
        assert_eq!(
            ngay_ke_tiep(Ngay {
                nam: 2025,
                thang: 12,
                ngay: 31
            }),
            Ngay {
                nam: 2026,
                thang: 1,
                ngay: 1
            }
        );
        assert_eq!(
            ngay_ke_tiep(Ngay {
                nam: 2024,
                thang: 2,
                ngay: 28
            }),
            Ngay {
                nam: 2024,
                thang: 2,
                ngay: 29
            },
            "2024 nhuận"
        );
    }

    /// Bộ luật phải có cả hai mức, và mức `gốc` không được nuốt mất mức `tất cả`.
    #[test]
    fn bo_luat_co_du_hai_muc() {
        let l = dung_luat_bao_ve(Path::new(r"D:\zalo-tool"));
        assert!(l.iter().any(|r| matches!(r.muc, Muc::TatCa)));
        assert!(l.iter().any(|r| matches!(r.muc, Muc::Goc)));
        assert!(
            l.iter()
                .any(|r| r.duong_dan == r"D:\zalo-tool" && matches!(r.muc, Muc::TatCa)),
            "thư mục công cụ phải được chặn ở mức tất cả"
        );
    }

    #[test]
    fn canon_bo_gach_cheo_thua_nhung_giu_goc_o_dia() {
        assert_eq!(canon(r"C:\"), r"C:\");
        assert_eq!(canon(""), "");
    }

    /// `canon` phải MỞ được tên ngắn 8.3 thành tên dài.
    ///
    /// # Vì sao đây là chuyện an toàn chứ không phải chuyện gọn gàng
    ///
    /// Vùng bảo vệ so bằng **chuỗi**. Đưa gốc dữ liệu vào ở dạng ngắn
    /// `C:\Users\RUNNE~1\...` trong khi bộ duyệt trả về dạng dài
    /// `C:\Users\runneradmin\...` thì hai bên không bao giờ khớp, và vùng bảo vệ
    /// **biến mất không một lời cảnh báo**.
    ///
    /// Bản PowerShell đã dính đúng lỗi này, và chính máy chủ CI tìm ra: `%TEMP%`
    /// trên đó là dạng ngắn nên `Database` với `Partitions` bị xóa sạch, mà màn
    /// hình không hề in dòng "Đã chặn ... tệp thuộc vùng bảo vệ".
    ///
    /// Ổ đĩa tắt 8.3 thì `ten_ngan` trả `None` và phép thử tự bỏ qua — nói rõ ra
    /// bằng `eprintln!` chứ không im lặng báo xanh.
    #[cfg(windows)]
    #[test]
    fn canon_mo_duoc_ten_ngan_8_3() {
        let dai = std::env::temp_dir().join(format!(
            "zalo_ten_that_dai_de_sinh_ten_ngan_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dai).unwrap();
        let dai_s = dai.to_string_lossy().to_string();

        match sysinfo::ten_ngan(&dai_s) {
            None => eprintln!(
                "CHÚ Ý: ổ đĩa này đã tắt tên ngắn 8.3 nên phép thử bỏ qua. \
                 Trên máy có bật 8.3 nó mới kiểm được gì."
            ),
            Some(ngan) => {
                assert!(ngan.contains('~'), "dạng ngắn phải có dấu ~, nhận {ngan}");
                assert_eq!(
                    canon(&ngan).to_lowercase(),
                    canon(&dai_s).to_lowercase(),
                    "canon KHÔNG mở được tên ngắn 8.3 — vùng bảo vệ sẽ trượt hết"
                );
            }
        }
        let _ = std::fs::remove_dir_all(&dai);
    }
}
