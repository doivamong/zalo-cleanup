//! Các màn hình.
//!
//! Mã ở đây **chỉ vẽ và chuyển sự kiện**. Mọi câu hỏi "có được xóa không" đều
//! hỏi [`crate::xac_nhan`], [`crate::xem_truoc`] hoặc lõi — không có `if` nào ở
//! đây tự trả lời câu ấy.
//!
//! Đường đi bắt buộc, và không có lối tắt nào:
//!
//! ```text
//! Trang chủ → chọn việc → QUÉT → XEM DANH SÁCH → gõ cụm từ → xóa
//!                                 ▲ chốt xem trước   ▲ chốt cụm từ + khóa mồi
//! ```

use crate::duong_lui::DuongLui;
use crate::muc_rui_ro::MucRuiRo;
use crate::nen::{Tin, ViecNen};
use crate::phong::{bieu_tuong, NguonPhong};
use crate::xac_nhan::{QuyetDinh, SuKien, TrangXacNhan};
use crate::xem_truoc::{ngui_tep, ChotXemTruoc, LoaiTep};
use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use zalo_core::act::TepQuet;
use zalo_core::protect::{Luat, Muc, VungBaoVe};
use zalo_core::{sysinfo, walk};

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
const TEN_BAO_VE: [&str; 2] = ["Database", "Partitions"];
const THU_MUC_DOC_LAP: [&str; 4] = ["video", "picture", "voice", "file"];

#[derive(PartialEq, Eq, Clone, Copy)]
enum ManHinh {
    TrangChu,
    LayLaiDungLuong,
    KetQuaQuet,
    XemDanhSach,
    XacNhanXoa,
    DangLam,
    KetQua,
    VungBaoVe,
    SaoLuu,
    KhoiPhuc,
}

/// Kết quả quét đang giữ, cùng chốt xem trước của riêng nó.
struct KetQuaQuet {
    tep: Vec<TepQuet>,
    loai: String,
    goc: String,
    goc_don_dep: Vec<String>,
    dau_quet: String,
    bi_chan: usize,
    chot: ChotXemTruoc,
    /// Loại tệp đã ngửi, tính một lần trong luồng nền cho tối đa từng ấy dòng.
    loai_tep: Vec<LoaiTep>,
}

impl KetQuaQuet {
    fn byte(&self) -> u64 {
        self.tep.iter().map(|t| t.co).sum()
    }
    fn muc(&self) -> MucRuiRo {
        MucRuiRo::tu_loai_quet(&self.loai)
    }
}

pub struct UngDung {
    man: ManHinh,
    goc: String,
    goc_du_lieu: String,
    co_zalo: bool,
    vbv: VungBaoVe,
    luat: Vec<Luat>,
    nguon_phong: NguonPhong,

    quet: Option<KetQuaQuet>,
    xn: Option<TrangXacNhan>,
    viec: Option<ViecNen>,
    ket_qua: Vec<String>,
    loi: Option<String>,
    /// Câu giải thích vì sao vừa bị chặn — `ĐM-04` đòi nó phải hiện ra chứ
    /// không chỉ im lặng không làm gì.
    chan_vi: Option<String>,
    truoc_do: std::time::Instant,
    /// `ĐM-08`. Dò một lần lúc mở, rồi dò lại theo nhịp — người dùng có thể bật
    /// trình đọc màn hình **giữa chừng**, và lúc đó họ càng cần đường lui.
    duong_lui: DuongLui,
    lan_do_duong_lui: std::time::Instant,

    /// Thư mục đích người dùng gõ vào ở màn sao lưu.
    dich_sao_luu: String,
    /// Xác minh SHA-256 **toàn bộ** thay vì mẫu 50 tệp.
    xac_minh_toan_bo: bool,
    /// Bản sao lưu vừa tạo cho đúng lượt quét đang giữ.
    sao_luu_gan_nhat: Option<zalo_core::gate::KetQuaSaoLuu>,
    /// Các bản sao lưu tìm được ở màn khôi phục.
    ds_sao_luu: Vec<zalo_core::store::BoSaoLuu>,
    /// Khôi phục có ghi đè tệp đã tồn tại không. Mặc định **không** — đè mất bản
    /// đang dùng là hỏng theo chiều ngược lại với chiều người dùng đang lo.
    ghi_de_khi_khoi_phuc: bool,

    /// Mười hai ô xem trước của lượt quét đang giữ.
    o_anh: Vec<crate::anh::O>,
    anh_dang_giai_ma: bool,
    /// Luồng giải mã ảnh, TÁCH RIÊNG khỏi `viec`.
    ///
    /// Dùng chung một chỗ thì mở danh sách xem trước sẽ đá văng việc nền đang
    /// chạy — và việc nền đang chạy có thể là một lượt xóa.
    viec_anh: Option<ViecNen>,
    /// Kết cấu đã nạp vào egui, giữ lại để khỏi nạp lại mỗi khung.
    ket_cau: Vec<Option<egui::TextureHandle>>,
}

impl UngDung {
    /// `goc_chi_dinh` là tham số `-Root`, giống hệt bản dòng lệnh.
    ///
    /// Không phải cờ dành cho lập trình viên: máy nào có nhiều tài khoản Zalo,
    /// hoặc có bố cục thư mục lạ, thì phép tự dò chọn nhầm — và một công cụ xóa
    /// dữ liệu chọn nhầm thư mục là chuyện không được để người dùng chịu.
    pub fn moi(nguon_phong: NguonPhong, goc_chi_dinh: Option<String>) -> Self {
        let goc = goc_chi_dinh
            .filter(|g| !g.trim().is_empty())
            .or_else(tim_goc_zalo)
            .unwrap_or_default();
        let goc_du_lieu = Path::new(&goc)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let luat = dung_luat_bao_ve();
        let vbv = VungBaoVe::dung(&luat, &goc_du_lieu, &TEN_BAO_VE);
        let co_zalo = !goc.is_empty() && Path::new(&goc).is_dir();
        UngDung {
            man: ManHinh::TrangChu,
            goc,
            goc_du_lieu,
            co_zalo,
            vbv,
            luat,
            nguon_phong,
            quet: None,
            xn: None,
            viec: None,
            ket_qua: Vec::new(),
            loi: None,
            chan_vi: None,
            truoc_do: std::time::Instant::now(),
            duong_lui: DuongLui::do_hien_tai(),
            lan_do_duong_lui: std::time::Instant::now(),
            dich_sao_luu: String::new(),
            xac_minh_toan_bo: false,
            sao_luu_gan_nhat: None,
            ds_sao_luu: Vec::new(),
            ghi_de_khi_khoi_phuc: false,
            o_anh: Vec::new(),
            anh_dang_giai_ma: false,
            viec_anh: None,
            ket_cau: Vec::new(),
        }
    }

    /// Bản sao lưu hiện có đúng là của kết quả quét đang giữ, và **sạch**.
    ///
    /// Gọi thẳng [`zalo_core::gate::sao_luu_sach`] — không viết lại phép kiểm ở
    /// đây. Chép một chốt an toàn ra chỗ thứ hai là mời hai chỗ trôi khỏi nhau,
    /// và chỗ trôi sẽ đúng là chỗ không có phép thử nào canh.
    fn sao_luu_sach(&self) -> bool {
        let dau = match &self.quet {
            Some(q) => q.dau_quet.clone(),
            None => return false,
        };
        zalo_core::gate::sao_luu_sach(self.sao_luu_gan_nhat.as_ref(), &dau)
    }

    /// Nguyên tắc bất biến số 2 và chốt xem trước, gộp một chỗ.
    ///
    /// Bỏ luôn bản sao lưu gần nhất: giữ nó lại nghĩa là một bản sao lưu của
    /// **kết quả quét cũ** vẫn mở khóa được bước xóa cho kết quả quét mới, tức
    /// xóa những tệp chưa từng được sao lưu.
    fn bo_ket_qua_quet(&mut self) {
        self.quet = None;
        self.xn = None;
        self.sao_luu_gan_nhat = None;
        self.o_anh.clear();
        self.ket_cau.clear();
        self.anh_dang_giai_ma = false;
    }
}

// ==================================================================== vòng vẽ

impl eframe::App for UngDung {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        self.thu_tin(ctx);
        self.esc_quay_lai(ctx);
        self.ve_dai_duong_lui(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.man {
                ManHinh::TrangChu => self.ve_trang_chu(ui),
                ManHinh::LayLaiDungLuong => self.ve_lay_lai_dung_luong(ui),
                ManHinh::KetQuaQuet => self.ve_ket_qua_quet(ui),
                ManHinh::XemDanhSach => self.ve_xem_danh_sach(ui),
                ManHinh::XacNhanXoa => self.ve_xac_nhan_xoa(ui, ctx),
                ManHinh::DangLam => self.ve_dang_lam(ui, ctx),
                ManHinh::KetQua => self.ve_ket_qua(ui),
                ManHinh::VungBaoVe => self.ve_vung_bao_ve(ui),
                ManHinh::SaoLuu => self.ve_sao_luu(ui),
                ManHinh::KhoiPhuc => self.ve_khoi_phuc(ui),
            });
        });
    }
}

impl UngDung {
    /// Nhận tin từ luồng nền. Chạy mỗi khung, không chặn.
    ///
    /// Gom hết tin ra một chỗ TRƯỚC rồi mới xử lý: xử lý ngay trong vòng đọc thì
    /// tay mượn của `self.viec` còn giữ, mà mỗi tin lại phải sửa `self.quet`,
    /// `self.man`… Gom trước cũng đúng hơn về nghĩa — một khung vẽ xử lý trọn
    /// mọi tin đã tới, không bỏ sót tin nào ở giữa.
    fn thu_tin(&mut self, ctx: &egui::Context) {
        let mut xong = false;
        let mut hop: Vec<Tin> = Vec::new();
        if let Some(v) = &self.viec {
            while let Ok(t) = v.nhan.try_recv() {
                hop.push(t);
            }
        }
        // Luồng ảnh chạy song song với luồng việc chính. Hút riêng, và dọn tay
        // cầm khi nó xong để khỏi giữ luồng đã chết.
        let mut anh_xong = false;
        if let Some(v) = &self.viec_anh {
            while let Ok(t) = v.nhan.try_recv() {
                if matches!(t, Tin::AnhXong(_)) {
                    anh_xong = true;
                }
                hop.push(t);
            }
        }
        if anh_xong {
            self.viec_anh = None;
        }
        if !hop.is_empty() || self.viec.is_some() || self.viec_anh.is_some() {
            for t in hop {
                match t {
                    Tin::DangLam(m) => {
                        if let Some(v) = &mut self.viec {
                            v.mo_ta = m;
                        }
                    }
                    Tin::QuetXong {
                        tep,
                        loai,
                        goc,
                        goc_don_dep,
                        bi_chan,
                        loi,
                    } => {
                        let n = tep.len();
                        self.quet = Some(KetQuaQuet {
                            tep,
                            loai,
                            goc,
                            goc_don_dep,
                            dau_quet: zalo_core::thoigian::luc_nay().dinh_dang(),
                            bi_chan,
                            // Chốt xem trước LUÔN đóng lại với mỗi kết quả quét
                            // mới. Giữ trạng thái "đã xem" của lượt trước là cho
                            // phép xóa một danh sách chưa ai nhìn qua.
                            chot: ChotXemTruoc::moi(),
                            loai_tep: Vec::new(),
                        });
                        if loi > 0 {
                            self.ket_qua
                                .push(format!("{} mục không đọc được, đã bỏ qua", loi));
                        }
                        self.man = if n == 0 {
                            ManHinh::KetQua
                        } else {
                            ManHinh::KetQuaQuet
                        };
                        if n == 0 {
                            self.ket_qua = vec!["Không có tệp nào khớp.".into()];
                        }
                        xong = true;
                    }
                    Tin::XoaXong(r, thu_muc_rong) => {
                        self.ket_qua = bao_cao_xoa(&r, thu_muc_rong);
                        self.bo_ket_qua_quet();
                        self.man = ManHinh::KetQua;
                        xong = true;
                    }
                    Tin::SaoLuuXong(r, dich) => {
                        // Ghi lại trạng thái để bước xóa XÉT, không phải để hiển
                        // thị. `het_cho` và phép so `da_chep` với `tong` là hai
                        // vế bắt buộc — xem `gate::sao_luu_sach`.
                        if let Some(q) = &self.quet {
                            self.sao_luu_gan_nhat = Some(zalo_core::gate::KetQuaSaoLuu {
                                dau_quet: q.dau_quet.clone(),
                                tong: r.tong as u64,
                                xong: r.da_chep as u64,
                                loi_chep: r.chep_hong as u64,
                                loi_xac_minh: r.xac_minh_hong as u64,
                                het_cho: r.het_cho,
                            });
                        }
                        let mut v = vec![
                            format!(
                                "Đã chép   : {} / {} tệp",
                                so(r.da_chep as i64),
                                so(r.tong as i64)
                            ),
                            format!(
                                "Xác minh  : kích thước {}/{} · SHA-256 {}/{}",
                                so(r.da_chep as i64),
                                so(r.da_chep as i64),
                                so(r.da_xac_minh as i64),
                                so(r.da_chep as i64)
                            ),
                            format!("Vị trí    : {dich}"),
                        ];
                        if self.sao_luu_sach() {
                            v.push(format!(
                                "{}  Sao lưu sạch. Đã mở khóa bước xóa.",
                                bieu_tuong::XONG
                            ));
                        } else {
                            v.push(format!(
                                "{}  Sao lưu CHƯA sạch. Bước xóa vẫn bị chặn.",
                                bieu_tuong::CANH_BAO
                            ));
                            if r.het_cho {
                                v.push(
                                    "Ổ đích hết chỗ giữa chừng — bản sao lưu không trọn vẹn."
                                        .into(),
                                );
                            }
                            for d in r.nhat_ky_loi.iter().take(5) {
                                v.push(d.clone());
                            }
                        }
                        self.ket_qua = v;
                        self.man = ManHinh::KetQua;
                        xong = true;
                    }
                    Tin::KhoiPhucXong(r) => {
                        self.ket_qua = vec![format!("Đã khôi phục {} tệp", r.da_khoi_phuc)];
                        self.man = ManHinh::KetQua;
                        xong = true;
                    }
                    Tin::AnhXong(o) => {
                        self.o_anh = o;
                        self.anh_dang_giai_ma = false;
                        // KHÔNG đổi màn hình: người dùng đang đứng ở danh sách,
                        // và ảnh chỉ hiện dần ra. Nhảy màn giữa chừng là cướp
                        // mất chỗ họ đang đọc.
                    }
                    Tin::Hong(m) => {
                        self.loi = Some(m);
                        self.man = ManHinh::KetQua;
                        xong = true;
                    }
                }
            }
            // Việc nền đang chạy thì phải vẽ lại đều để thanh tiến độ nhúc nhích.
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if xong {
            self.viec = None;
        }
    }

    /// **BP-06.** Esc luôn là "lui một bước", trên mọi màn hình **trừ hai chỗ**.
    ///
    /// Hai chỗ trừ ra, và lý do:
    ///
    /// - **Trang xác nhận xóa** tự xử Esc lấy, vì ở đó Esc nghĩa là **hủy hẳn**
    ///   lượt xóa chứ không phải lùi một màn.
    /// - **Màn đang làm** cũng tự xử, vì ở đó Esc nghĩa là **dừng thao tác đang
    ///   chạy** — thứ mà `BP-08` đòi.
    ///
    /// Đặt hai ngoại lệ ở đây, tường minh, thay vì để mỗi màn tự đoán: một phím
    /// mang ba nghĩa khác nhau tùy chỗ thì phải viết cả ba nghĩa ra một chỗ.
    fn esc_quay_lai(&mut self, ctx: &egui::Context) {
        if matches!(self.man, ManHinh::XacNhanXoa | ManHinh::DangLam) {
            return;
        }
        if !ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return;
        }
        self.man = match self.man {
            ManHinh::TrangChu => ManHinh::TrangChu,
            ManHinh::XemDanhSach => ManHinh::KetQuaQuet,
            ManHinh::SaoLuu => ManHinh::KetQuaQuet,
            _ => ManHinh::TrangChu,
        };
    }

    /// **ĐM-08.** Dải thông báo đường lui, hiện trên **mọi** màn hình.
    ///
    /// Đặt ở khung trên cùng chứ không nhét vào trang chủ: người dùng có thể
    /// bật trình đọc màn hình lúc đã đi sâu vào giữa luồng, và đó đúng là lúc
    /// họ cần đường lui nhất — bắt họ quay về trang chủ mới thấy nó thì coi như
    /// không có.
    ///
    /// Không có nút đóng dải này. `ĐM-05`: thông báo an toàn không tự biến mất,
    /// và cũng không được để người dùng lỡ tay tắt mất.
    fn ve_dai_duong_lui(&mut self, ctx: &egui::Context) {
        // Dò lại theo nhịp thưa: trình đọc màn hình có thể bật lên giữa chừng.
        if self.lan_do_duong_lui.elapsed().as_secs() >= 3 {
            self.duong_lui = DuongLui::do_hien_tai();
            self.lan_do_duong_lui = std::time::Instant::now();
        }
        if !self.duong_lui.nen_hien() {
            return;
        }
        let cau = self.duong_lui.cau();
        let mo_duoc = matches!(self.duong_lui, DuongLui::Co(_));
        egui::TopBottomPanel::top("duong_lui").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(format!("{}  {cau}", bieu_tuong::DAN_HUONG));
            if mo_duoc && ui.button("Mở bản dòng lệnh").clicked() {
                self.duong_lui.mo();
            }
            ui.add_space(4.0);
        });
    }

    fn tieu_de(&self, ui: &mut egui::Ui, chu: &str) {
        ui.add_space(6.0);
        ui.heading(chu);
        ui.separator();
        ui.add_space(4.0);
    }

    fn nut_quay_lai(&mut self, ui: &mut egui::Ui, ve: ManHinh) {
        if ui.button("← Quay lại").clicked() {
            self.chan_vi = None;
            self.man = ve;
        }
    }

    // ---------------------------------------------------------------- trang chủ

    fn ve_trang_chu(&mut self, ui: &mut egui::Ui) {
        self.tieu_de(ui, "Dọn dẹp Zalo");
        let goc_ht = sysinfo::goc_he_thong();
        let trong = sysinfo::byte_trong(&goc_ht);
        if trong >= 0 {
            ui.label(format!(
                "Ổ {} còn trống    {}",
                sysinfo::nhan_o_dia(&goc_ht),
                co(trong)
            ));
        }
        if !self.co_zalo {
            ui.label("Máy này chưa cài Zalo — vẫn dọn được cache hệ thống.");
        } else {
            ui.label(format!("Thư mục Zalo: {}", self.goc));
        }
        ui.add_space(10.0);
        ui.label("Bạn muốn làm gì?");
        ui.add_space(6.0);
        if nut_co_the_tat(ui, self.co_zalo, "Lấy lại dung lượng ổ đĩa").clicked() {
            self.man = ManHinh::LayLaiDungLuong;
        }
        if ui.button("Khôi phục dữ liệu đã sao lưu").clicked() {
            self.man = ManHinh::KhoiPhuc;
        }
        if ui.button("Xem vùng bảo vệ").clicked() {
            self.man = ManHinh::VungBaoVe;
        }
        ui.add_space(12.0);
        ui.separator();
        ui.label(match &self.nguon_phong {
            NguonPhong::HeThong(t) => format!("Phông: {t} (hệ thống)"),
            NguonPhong::Nhung => "Phông: bản nhúng sẵn".to_string(),
        });
        ui.label(
            "Công cụ này xóa vĩnh viễn, không qua Thùng rác. Không có tác vụ nền nào được đặt.",
        );
    }

    // ------------------------------------------------------- lấy lại dung lượng

    fn ve_lay_lai_dung_luong(&mut self, ui: &mut egui::Ui) {
        self.tieu_de(ui, "Lấy lại dung lượng ổ đĩa");
        let m = |mu: MucRuiRo| format!("{}  {}", mu.ky_hieu(), mu.chu());

        ui.label(format!(
            "{}   Bản trùng lặp trong Zalo",
            m(MucRuiRo::AnToan)
        ));
        if ui.button("Tìm bản trùng lặp").clicked() {
            self.chay_quet_trung_lap();
        }
        ui.add_space(8.0);
        ui.label(format!("{}   Cache của ứng dụng Zalo", m(MucRuiRo::AnToan)));
        if ui.button("Quét cache Zalo").clicked() {
            self.chay_quet_cache();
        }
        ui.add_space(8.0);
        ui.label(format!("{}   Dữ liệu Zalo cũ", m(MucRuiRo::NguyHiem)));
        if ui.button("Quét dữ liệu cũ hơn 12 tháng").clicked() {
            self.chay_quet_theo_tuoi(12);
        }
        ui.add_space(14.0);
        self.nut_quay_lai(ui, ManHinh::TrangChu);
    }

    // ---------------------------------------------------------- kết quả quét

    fn ve_ket_qua_quet(&mut self, ui: &mut egui::Ui) {
        let (loai, n, byte, bi_chan, muc, da_xem) = match &self.quet {
            Some(q) => (
                q.loai.clone(),
                q.tep.len(),
                q.byte(),
                q.bi_chan,
                q.muc(),
                q.chot.da_xem(),
            ),
            None => {
                self.man = ManHinh::TrangChu;
                return;
            }
        };
        self.tieu_de(ui, &format!("Kết quả quét · {loai}"));
        ui.label(format!("{}  {}", muc.ky_hieu(), muc.chu()));
        ui.add_space(6.0);
        ui.label(format!("Số tệp     : {}", so(n as i64)));
        ui.label(format!("Dung lượng : {}", co(byte as i64)));
        if bi_chan > 0 {
            ui.label(format!(
                "{}  Vùng bảo vệ đã chặn {} tệp",
                bieu_tuong::VUNG_BAO_VE,
                so(bi_chan as i64)
            ));
        }
        ui.add_space(10.0);

        if ui.button("Xem danh sách tệp sắp mất").clicked() {
            if let Some(q) = &mut self.quet {
                q.chot.danh_dau_da_xem();
                if q.loai_tep.is_empty() {
                    q.loai_tep = q
                        .tep
                        .iter()
                        .take(400)
                        .map(|t| ngui_tep(Path::new(&t.duong_dan)))
                        .collect();
                }
            }
            self.chay_giai_ma_anh();
            self.man = ManHinh::XemDanhSach;
        }

        // Sao lưu là ĐƯỜNG LUI, nên nó phải đứng trước nút xóa cả về vị trí lẫn
        // về câu chữ. Đặt sau nút xóa là mời người dùng đọc lướt qua nó.
        ui.add_space(6.0);
        if muc == MucRuiRo::NguyHiem {
            if self.sao_luu_sach() {
                ui.label(format!(
                    "{}  Đã có bản sao lưu sạch cho đúng lượt quét này.",
                    bieu_tuong::XONG
                ));
            } else {
                ui.label(format!(
                    "{}  Chưa sao lưu. Sao lưu là cách duy nhất để còn đường lui.",
                    bieu_tuong::CANH_BAO
                ));
            }
        }
        if ui.button("Sao lưu trước khi xóa").clicked() {
            self.man = ManHinh::SaoLuu;
        }
        ui.add_space(6.0);

        // CHỐT XEM TRƯỚC. Nút xóa không bật cho tới khi người dùng đã nhìn.
        let chot = self
            .quet
            .as_ref()
            .map(|q| q.chot.clone())
            .unwrap_or_default();
        let cho = chot.cho_sang_trang_xac_nhan();
        let nut = nut_co_the_tat(
            ui,
            cho,
            format!("{}  Xóa vĩnh viễn…", bieu_tuong::NGUY_HIEM),
        );
        if let Some(ly_do) = chot.ly_do_tat() {
            // ĐM-06: nút tắt phải NÓI được lý do, không chỉ xám đi.
            nut.on_disabled_hover_text(ly_do);
            ui.label(format!("{}  Nút xóa: {ly_do}", bieu_tuong::CANH_BAO));
        } else if nut.clicked() {
            self.xn = Some(TrangXacNhan::moi("XÓA", "XOA"));
            self.truoc_do = std::time::Instant::now();
            self.man = ManHinh::XacNhanXoa;
        }
        let _ = da_xem;
        ui.add_space(12.0);
        if ui.button("Bỏ kết quả này").clicked() {
            self.bo_ket_qua_quet();
            self.man = ManHinh::TrangChu;
        }
    }

    // ---------------------------------------------------------- xem danh sách

    fn ve_xem_danh_sach(&mut self, ui: &mut egui::Ui) {
        if self.quet.is_none() {
            self.man = ManHinh::TrangChu;
            return;
        }
        // Bốc số ra TRƯỚC rồi mới vẽ: `ve_luoi_anh` cần mượn `self` ở dạng sửa
        // được, mà tay mượn chỉ đọc của `self.quet` thì còn giữ suốt hàm.
        let tong_tep = self.quet.as_ref().unwrap().tep.len();
        let q = self.quet.as_ref().unwrap();
        self.tieu_de(ui, "Những tệp sắp mất");
        ui.label(format!(
            "{} tệp · {}. Đây là dữ liệu trên máy bạn.",
            so(q.tep.len() as i64),
            co(q.byte() as i64)
        ));
        // RB-43: nói thẳng phần đã ngửi loại chỉ là một phần, và nó không nói
        // được gì về phần còn lại.
        if q.tep.len() > q.loai_tep.len() {
            ui.label(format!(
                "Đã nhận dạng {} tệp đầu. Chúng không nói được gì về {} tệp còn lại.",
                so(q.loai_tep.len() as i64),
                so((q.tep.len() - q.loai_tep.len()) as i64)
            ));
        }
        ui.add_space(6.0);

        let goc = q.goc.clone();

        // ---- Lưới ảnh xem trước: ma sát mạnh nhất của cả giao diện.
        if self.anh_dang_giai_ma {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Đang giải mã ảnh xem trước…");
            });
        }
        if !self.o_anh.is_empty() {
            self.ve_luoi_anh(ui, tong_tep);
        }
        ui.add_space(6.0);

        // RB-129: ảo hóa. Không có gì tỉ lệ với N chạy trong luồng vẽ.
        let q = self.quet.as_ref().unwrap();
        let cao = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
        let tong = q.tep.len();
        egui::ScrollArea::vertical()
            .max_height(360.0)
            .show_rows(ui, cao, tong, |ui, dai| {
                for i in dai {
                    let t = &q.tep[i];
                    let nhan = match q.loai_tep.get(i) {
                        Some(LoaiTep::Jpeg) => "ảnh JPEG",
                        Some(LoaiTep::Png) => "ảnh PNG",
                        Some(LoaiTep::JpegXl) => "ảnh JPEG XL",
                        Some(LoaiTep::Mp4) => "video",
                        _ => "?",
                    };
                    ui.label(format!(
                        "{}   {}   [{}]",
                        nhan,
                        zalo_core::scan::duong_dan_tuong_doi(&t.duong_dan, &goc),
                        co(t.co as i64)
                    ));
                }
            });
        ui.add_space(10.0);
        self.nut_quay_lai(ui, ManHinh::KetQuaQuet);
    }

    // --------------------------------------------------------- xác nhận xóa

    fn ve_xac_nhan_xoa(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let (loai, n, byte, muc) = match &self.quet {
            Some(q) => (q.loai.clone(), q.tep.len(), q.byte(), q.muc()),
            None => {
                self.man = ManHinh::TrangChu;
                return;
            }
        };
        let troi = self.truoc_do.elapsed().as_millis() as u64;
        self.truoc_do = std::time::Instant::now();

        // Gom sự kiện thật thành SuKien rồi đưa hết vào máy trạng thái. Mã vẽ
        // KHÔNG tự quyết định gì — nó chỉ dịch sự kiện và hỏi lại.
        let mut sk: Vec<SuKien> = vec![SuKien::ThoiGianTroi(troi)];
        // Một lỗ THẬT của egui, không phải chuyện lý thuyết: nút đang có tiêu
        // điểm kích hoạt được bằng **Enter và Space**, và `Response::clicked()`
        // trả `true` y như bấm chuột. Tức điều 1 và điều 2 của BP-05 bị lách
        // ngay ở tầng thư viện, không nhìn thấy được từ mã của ta.
        //
        // Nên khung nào có Enter hoặc Space thì **nuốt sạch** mọi cú bấm của
        // khung ấy. Người dùng bấm chuột đúng lúc đang gõ Enter thì mất một cú
        // nhấp — đổi lại, không có đường nào từ bàn phím tới lệnh xóa.
        let mut nuot_bam = false;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Enter) {
                sk.push(SuKien::Enter);
                nuot_bam = true;
            }
            if i.key_down(egui::Key::Space) || i.key_pressed(egui::Key::Space) {
                // Space giữ lâu sinh ra chuỗi sự kiện tự lặp — điều 6.
                sk.push(SuKien::PhimTuLap);
                nuot_bam = true;
            }
            if i.key_pressed(egui::Key::Escape) {
                sk.push(SuKien::Esc);
            }
            // Điều 9: chặn dán, cả Ctrl+V lẫn menu chuột phải.
            if i.events.iter().any(|e| matches!(e, egui::Event::Paste(_))) {
                sk.push(SuKien::Dan(String::new()));
            }
        });

        let xn = match &mut self.xn {
            Some(x) => x,
            None => {
                self.man = ManHinh::KetQuaQuet;
                return;
            }
        };
        let mut qd = QuyetDinh::KhongLam;
        for e in sk {
            let r = xn.nhan(e);
            if r != QuyetDinh::KhongLam {
                qd = r;
            }
        }

        self.tieu_de(ui, &format!("Xóa vĩnh viễn · {loai}"));
        ui.label(format!("{}  {}", muc.ky_hieu(), muc.chu()));
        ui.add_space(6.0);
        ui.label(format!("Số tệp     : {}", so(n as i64)));
        ui.label(format!("Dung lượng : {}", co(byte as i64)));
        ui.add_space(6.0);
        ui.label("Tệp sẽ bị xóa hẳn, không qua Thùng rác, không khôi phục được.");
        ui.add_space(10.0);

        // Cụm từ in ra dưới dạng nhãn — không bôi đen sao chép được (điều 9).
        ui.label("Gõ đúng chữ  XÓA  để xác nhận:");
        let mut o = self.xn.as_ref().unwrap().o_nhap().to_string();
        // Thứ tự dựng widget CHÍNH LÀ thứ tự Tab: ô nhập → Hủy → Xóa.
        // Nút phá hủy dựng CUỐI CÙNG (điều 4).
        let ph = ui.add(egui::TextEdit::singleline(&mut o).desired_width(220.0));
        if ph.changed() {
            self.xn.as_mut().unwrap().nhan(SuKien::Go(o));
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("Hủy").clicked() {
                qd = self.xn.as_mut().unwrap().nhan(SuKien::BamNutHuy);
            }
            ui.add_space(40.0);
            let x = self.xn.as_ref().unwrap();
            let cho = x.cho_bam_xoa();
            let ly_do = x.ly_do_nut_tat();
            let nut = nut_co_the_tat(ui, cho, format!("{}  Xóa vĩnh viễn", bieu_tuong::NGUY_HIEM));
            if let Some(l) = ly_do {
                nut.on_disabled_hover_text(l);
            } else if nut.clicked() && !nuot_bam {
                // `&& !nuot_bam` là chỗ bịt lỗ Enter/Space của egui. Xem chú
                // thích ở đầu hàm — bỏ nó đi là mở lại đường từ bàn phím tới
                // lệnh xóa, và không phép thử nào của máy trạng thái bắt được
                // vì lỗ nằm ở tầng thư viện chứ không ở tầng luật.
                qd = self.xn.as_mut().unwrap().nhan(SuKien::BamNutXoa);
            }
        });

        if let Some(l) = self.xn.as_ref().unwrap().ly_do_nut_tat() {
            ui.add_space(6.0);
            ui.label(format!("{}  Nút xóa chưa bật: {l}", bieu_tuong::CANH_BAO));
        }
        if self.xn.as_ref().unwrap().so_lan_chan_dan > 0 {
            ui.label(format!(
                "{}  Không dán được cụm từ. Phải gõ tay — đó là cả ý nghĩa của bước này.",
                bieu_tuong::CANH_BAO
            ));
        }

        match qd {
            QuyetDinh::Huy => {
                self.xn = None;
                self.man = ManHinh::KetQuaQuet;
            }
            QuyetDinh::BatDauXoa => self.chay_xoa(),
            _ => {}
        }
        // Trang này có ô nhập nên phải vẽ đều để khóa mồi đếm lùi đúng nhịp.
        ctx.request_repaint_after(std::time::Duration::from_millis(60));
    }

    // ------------------------------------------------------------- đang làm

    fn ve_dang_lam(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let (mo_ta, pt, da_xin) = match &self.viec {
            Some(v) => (v.mo_ta.clone(), v.phan_tram(), v.da_xin_huy()),
            None => {
                self.man = ManHinh::KetQua;
                return;
            }
        };
        self.tieu_de(ui, "Đang làm");
        ui.label(&mo_ta);
        match pt {
            Some(p) => {
                ui.add(egui::ProgressBar::new(p).show_percentage());
            }
            None => {
                ui.spinner();
            }
        }
        ui.add_space(10.0);
        // BP-08: Esc dừng được thao tác đang chạy.
        let xin_huy = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if (ui.button("Dừng (Esc)").clicked() || xin_huy) && !da_xin {
            if let Some(v) = &self.viec {
                v.xin_huy();
            }
        }
        if da_xin {
            ui.label("Đang dừng an toàn ở tệp gần nhất…");
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    fn ve_ket_qua(&mut self, ui: &mut egui::Ui) {
        self.tieu_de(ui, "Kết quả");
        if let Some(e) = &self.loi {
            ui.label(format!("{}  {e}", bieu_tuong::HONG));
        } else {
            ui.label(format!("{}  Xong.", bieu_tuong::XONG));
        }
        for d in &self.ket_qua {
            ui.label(d);
        }
        ui.add_space(12.0);

        // ĐƯỜNG VỀ KẾT QUẢ QUÉT. Thiếu nó là một ngõ cụt, và là ngõ cụt đúng ở
        // chỗ nguy hiểm nhất.
        //
        // Sao lưu xong thì màn này hiện ra, và nó nói thẳng "Sao lưu sạch. Đã
        // mở khóa bước xóa." Nhưng nút duy nhất ở đây từng là "Về trang chủ",
        // mà trang chủ **không có đường nào quay lại kết quả quét** — nó chỉ
        // đặt được từ lúc quét xong. Người dùng làm đúng thứ giao diện khuyên
        // họ làm, rồi mất luôn lượt quét vừa làm.
        //
        // Nặng hơn phần phiền toái: `sao_luu_gan_nhat` khóa theo `dau_quet`,
        // nên quét lại là bản sao lưu vừa tạo cũng hết hiệu lực và phải sao lưu
        // lần nữa. Một câu "đã mở khóa bước xóa" dẫn tới chỗ không có bước xóa
        // thì tệ hơn là không nói gì.
        //
        // Tìm ra khi chạy `BP-01` — kịch bản bàn phím đi tới đây rồi tắc.
        if self.quet.is_some() && ui.button("← Quay lại kết quả quét").clicked() {
            self.ket_qua.clear();
            self.loi = None;
            self.man = ManHinh::KetQuaQuet;
        }
        if ui.button("Về trang chủ").clicked() {
            self.ket_qua.clear();
            self.loi = None;
            self.man = ManHinh::TrangChu;
        }
    }

    fn ve_vung_bao_ve(&mut self, ui: &mut egui::Ui) {
        self.tieu_de(ui, "Vùng bảo vệ — chỉ báo cáo, không bao giờ xóa");
        for n in TEN_BAO_VE {
            let p = Path::new(&self.goc_du_lieu).join(n);
            if !p.is_dir() {
                continue;
            }
            let r = walk::duyet(&p);
            let b: u64 = r.tep.iter().map(|t| t.co).sum();
            ui.label(format!(
                "{}  {n}   {} tệp · {}",
                bieu_tuong::VUNG_BAO_VE,
                so(r.tep.len() as i64),
                co(b as i64)
            ));
        }
        ui.add_space(8.0);
        ui.label("Database   — cơ sở dữ liệu tin nhắn. Xóa là mất lịch sử chat vĩnh viễn.");
        ui.label("Partitions — dữ liệu phiên đăng nhập. Xóa sẽ phải đăng nhập lại.");
        ui.label("Công cụ chặn cứng hai thư mục trên ở tầng mã. Không bộ lọc nào chạm được.");
        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                for r in &self.luat {
                    let lv = match r.muc {
                        Muc::TatCa => "tất cả",
                        Muc::Goc => "gốc",
                    };
                    ui.label(format!("[{lv}]  {}", r.duong_dan));
                }
            });
        ui.add_space(10.0);
        self.nut_quay_lai(ui, ManHinh::TrangChu);
    }

    /// Lưới mười hai ảnh xem trước.
    ///
    /// Ô nào không giải mã được thì hiện dấu hỏi kèm tên tệp — **không bao giờ**
    /// bỏ ô đó đi. Bỏ đi là người dùng đếm mười hai ô rồi tưởng mình đã nhìn hết
    /// mười hai tệp, trong khi có tệp bị giấu.
    fn ve_luoi_anh(&mut self, ui: &mut egui::Ui, tong: usize) {
        if self.ket_cau.len() != self.o_anh.len() {
            self.ket_cau = (0..self.o_anh.len()).map(|_| None).collect();
        }
        let canh = crate::anh::CANH as f32;
        // XUỐNG DÒNG, không cuộn ngang.
        //
        // Bản đầu bọc lưới trong `ScrollArea::horizontal`. Đo ở 1092×614 dip —
        // đúng cỡ màn 1366×768 @125% của `DPI-04` — thì **4 trong 12 ô** nằm
        // ngoài mép phải, và không cách nào tới được bằng bàn phím vì ô ảnh
        // không nhận tiêu điểm.
        //
        // Hai điều hỏng cùng lúc, và điều thứ hai nặng hơn: lưới này tồn tại để
        // người dùng **nhìn thấy** thứ sắp mất. Giấu một phần ba số ảnh sau một
        // cú cuộn ngang là bỏ đi đúng phần ma sát mà nó sinh ra để tạo — người
        // ta đếm tám ô rồi tưởng mình đã nhìn hết.
        //
        // Chia hàng bằng tay chứ không dùng `horizontal_wrapped`: mỗi ô là một
        // `ui.vertical` lồng trong, mà egui chỉ ngắt dòng theo **widget rời**,
        // không theo bố cục con. Đã thử và đo — `horizontal_wrapped` cho ra
        // đúng cái bố cục tràn như cũ.
        let buoc = canh + ui.spacing().item_spacing.x;
        let moi_hang =
            (((ui.available_width() + ui.spacing().item_spacing.x) / buoc).floor() as usize).max(1);
        let mut dau_hang = 0usize;
        while dau_hang < self.o_anh.len() {
            let cuoi_hang = (dau_hang + moi_hang).min(self.o_anh.len());
            ui.horizontal(|ui| {
                for (i, o) in self.o_anh.iter().enumerate().take(cuoi_hang).skip(dau_hang) {
                    ui.vertical(|ui| {
                        match &o.anh {
                            Some(a) if a.rong > 0 => {
                                let tex = self.ket_cau[i].get_or_insert_with(|| {
                                    let ci = egui::ColorImage::from_rgba_unmultiplied(
                                        [a.rong, a.cao],
                                        &a.diem,
                                    );
                                    ui.ctx().load_texture(
                                        format!("xt{i}"),
                                        ci,
                                        egui::TextureOptions::LINEAR,
                                    )
                                });
                                ui.image((tex.id(), tex.size_vec2()));
                            }
                            _ => {
                                // Ô dấu hỏi. Vẫn chiếm đúng chỗ của một ảnh
                                // để người dùng thấy rõ là có một tệp ở đây.
                                let (r, _) = ui.allocate_exact_size(
                                    egui::vec2(canh, canh),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_stroke(
                                    r,
                                    2.0,
                                    egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
                                );
                                ui.painter().text(
                                    r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "?",
                                    egui::FontId::proportional(28.0),
                                    ui.visuals().text_color(),
                                );
                            }
                        }
                        let ten = o
                            .duong_dan
                            .file_name()
                            .map(|x| x.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let ngan: String = ten.chars().take(14).collect();
                        ui.small(ngan);
                    });
                }
            });
            dau_hang = cuoi_hang;
        }
        // RB-43: dòng này KHÔNG được nhỏ hơn chữ thường và KHÔNG được xám. Mười
        // hai ảnh trông như một bằng chứng đầy đủ nếu thiếu nó.
        ui.label(crate::anh::dong_ty_le_mau(self.o_anh.len(), tong));
    }

    // ---------------------------------------------------------------- sao lưu

    fn ve_sao_luu(&mut self, ui: &mut egui::Ui) {
        let (n, byte) = match &self.quet {
            Some(q) => (q.tep.len(), q.byte()),
            None => {
                self.man = ManHinh::TrangChu;
                return;
            }
        };
        self.tieu_de(ui, "Sao lưu và xác minh");
        ui.label(format!(
            "Sẽ chép {} tệp · {}",
            so(n as i64),
            co(byte as i64)
        ));
        ui.add_space(6.0);
        for o in sysinfo::cac_o_dia() {
            ui.label(format!(
                "Ổ {}  trống {}",
                sysinfo::nhan_o_dia(&o),
                co(sysinfo::byte_trong(&o))
            ));
        }
        ui.add_space(8.0);
        ui.label("Thư mục đích:");
        ui.add(
            egui::TextEdit::singleline(&mut self.dich_sao_luu)
                .desired_width(420.0)
                .hint_text(r"D:\SaoLuuZalo"),
        );

        ui.add_space(8.0);
        ui.label("Mức xác minh sau khi chép:");
        ui.radio_value(
            &mut self.xac_minh_toan_bo,
            false,
            "Kích thước toàn bộ, cộng SHA-256 mẫu 50 tệp  (nhanh)",
        );
        ui.radio_value(
            &mut self.xac_minh_toan_bo,
            true,
            "SHA-256 toàn bộ  (chậm nhưng chắc chắn tuyệt đối)",
        );

        // Chừa 2% cộng 100 MB: hệ tệp còn siêu dữ liệu, và một bản sao lưu vừa
        // khít ổ đĩa là một bản sao lưu sắp hỏng.
        let can = (byte as f64 * 1.02) as i64 + 100 * 1024 * 1024;
        let trong = if self.dich_sao_luu.trim().is_empty() {
            -1
        } else {
            sysinfo::byte_trong(&self.dich_sao_luu)
        };
        ui.add_space(8.0);
        ui.label(format!("Cần ít nhất: {}", co(can)));

        let du_cho = trong >= can;
        if trong >= 0 {
            ui.label(format!(
                "Ổ {} trống: {}",
                sysinfo::nhan_o_dia(&self.dich_sao_luu),
                co(trong)
            ));
            if !du_cho {
                ui.label(format!(
                    "{}  Không đủ chỗ, thiếu khoảng {}. Chưa chép tệp nào.",
                    bieu_tuong::CANH_BAO,
                    co(can - trong)
                ));
            }
        }

        ui.add_space(10.0);
        let nut = nut_co_the_tat(ui, du_cho, "Bắt đầu sao lưu");
        if !du_cho {
            nut.on_disabled_hover_text("nhập thư mục đích có đủ chỗ trống");
        } else if nut.clicked() {
            self.chay_sao_luu();
        }
        ui.add_space(12.0);
        self.nut_quay_lai(ui, ManHinh::KetQuaQuet);
    }

    // -------------------------------------------------------------- khôi phục

    fn ve_khoi_phuc(&mut self, ui: &mut egui::Ui) {
        self.tieu_de(ui, "Khôi phục dữ liệu đã sao lưu");
        if ui.button("Tìm các bản sao lưu trên máy").clicked() {
            let cd =
                zalo_core::store::doc_cai_dat(&sysinfo::thu_muc_cong_cu().join("settings.json"));
            self.ds_sao_luu =
                zalo_core::store::tim_ban_sao_luu(&cd.goc_sao_luu, &sysinfo::cac_o_dia());
        }
        ui.add_space(6.0);
        if self.ds_sao_luu.is_empty() {
            ui.label("Chưa tìm thấy bản sao lưu nào do công cụ này tạo ra.");
        } else {
            ui.label(format!(
                "Tìm thấy {} bản sao lưu:",
                so(self.ds_sao_luu.len() as i64)
            ));
        }
        ui.add_space(6.0);
        ui.checkbox(
            &mut self.ghi_de_khi_khoi_phuc,
            "Ghi đè tệp đã tồn tại  (mặc định là bỏ qua, an toàn hơn)",
        );
        ui.add_space(8.0);

        let ds = self.ds_sao_luu.clone();
        let mut chon: Option<zalo_core::store::BoSaoLuu> = None;
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for s in &ds {
                    let m = &s.ban_ke;
                    ui.separator();
                    ui.label(format!(
                        "{}   {} tệp · {}",
                        m.tao_luc,
                        so(m.so_tep),
                        co(m.so_byte)
                    ));
                    ui.label(format!("Nội dung : {}", m.loai_quet));
                    ui.label(format!("Nằm ở    : {}", s.thu_muc.display()));
                    ui.label(format!("Trả về   : {}", m.goc_nguon));
                    if m.chep_hong > 0 || m.xac_minh_hong > 0 {
                        ui.label(format!(
                            "{}  Bản này từng lỗi (chép {}, xác minh {})",
                            bieu_tuong::CANH_BAO,
                            so(m.chep_hong),
                            so(m.xac_minh_hong)
                        ));
                    }
                    if ui.button("Khôi phục bản này").clicked() {
                        chon = Some(s.clone());
                    }
                }
            });
        if let Some(s) = chon {
            self.chay_khoi_phuc(s);
        }
        ui.add_space(12.0);
        self.nut_quay_lai(ui, ManHinh::TrangChu);
    }

    // ==================================================== khởi động việc nền

    fn chay_quet_theo_tuoi(&mut self, thang: u32) {
        let goc = self.goc.clone();
        let vbv = self.vbv.clone();
        self.man = ManHinh::DangLam;
        self.viec = Some(ViecNen::chay(
            "Đang quét theo mốc thời gian…",
            move |g, _, x, t| {
                let moc = zalo_core::thoigian::hom_nay().lui_thang(thang).nua_dem();
                let r = walk::duyet(Path::new(&goc));
                t.store(r.tep.len(), Ordering::Relaxed);
                let mut ra = Vec::new();
                let mut chan = 0usize;
                for (i, tp) in r.tep.iter().enumerate() {
                    x.store(i, Ordering::Relaxed);
                    let s = tp.duong_dan.to_string_lossy().to_string();
                    if vbv.chan(&s) {
                        chan += 1;
                        continue;
                    }
                    if zalo_core::scan::duoi_kieu_dotnet(zalo_core::scan::ten_tep(&s))
                        .eq_ignore_ascii_case(".rescache")
                    {
                        continue;
                    }
                    if tp.sua_luc < moc {
                        ra.push(TepQuet::moi(s, tp.co));
                    }
                }
                let _ = g.send(Tin::QuetXong {
                    tep: ra,
                    loai: "DỮ LIỆU ZALO".into(),
                    goc: goc.clone(),
                    goc_don_dep: vec![goc],
                    bi_chan: chan,
                    loi: r.loi,
                });
            },
        ));
    }

    fn chay_quet_cache(&mut self) {
        let du_lieu = self.goc_du_lieu.clone();
        let vbv = self.vbv.clone();
        self.man = ManHinh::DangLam;
        self.viec = Some(ViecNen::chay(
            "Đang quét cache Zalo…",
            move |g, _, _, _| {
                let mut ra = Vec::new();
                let mut goc_don = Vec::new();
                let mut loi = 0usize;
                for rel in CACHE_UNG_DUNG {
                    let p = Path::new(&du_lieu).join(rel);
                    if !p.is_dir() {
                        continue;
                    }
                    let ps = p.to_string_lossy().to_string();
                    if vbv.chan_thu_muc_goc(&ps) {
                        continue;
                    }
                    goc_don.push(ps.clone());
                    let r = walk::duyet(&p);
                    loi += r.loi;
                    for t in r.tep {
                        let s = t.duong_dan.to_string_lossy().to_string();
                        if vbv.chan(&s) {
                            continue;
                        }
                        ra.push(TepQuet::moi(s, t.co));
                    }
                }
                let _ = g.send(Tin::QuetXong {
                    tep: ra,
                    loai: "CACHE ZALO".into(),
                    goc: du_lieu,
                    goc_don_dep: goc_don,
                    bi_chan: 0,
                    loi,
                });
            },
        ));
    }

    fn chay_quet_trung_lap(&mut self) {
        let goc = self.goc.clone();
        let vbv = self.vbv.clone();
        self.man = ManHinh::DangLam;
        self.viec = Some(ViecNen::chay(
            "Đang đối chiếu nội dung bằng SHA-256…",
            move |g, co_huy, x, t| {
                let mut giu: std::collections::HashMap<u64, Vec<String>> = Default::default();
                for d in THU_MUC_DOC_LAP {
                    let p = Path::new(&goc).join(d);
                    if !p.is_dir() {
                        continue;
                    }
                    for tp in walk::duyet(&p).tep {
                        let s = tp.duong_dan.to_string_lossy().to_string();
                        if tp.co == 0 || vbv.chan(&s) {
                            continue;
                        }
                        giu.entry(tp.co).or_default().push(s);
                    }
                }
                let res = Path::new(&goc).join("resource");
                let uv: Vec<(String, u64)> = walk::duyet(&res)
                    .tep
                    .into_iter()
                    .map(|tp| (tp.duong_dan.to_string_lossy().to_string(), tp.co))
                    .filter(|(s, c)| *c > 0 && !s.contains("\\Cache\\") && !vbv.chan(s))
                    .filter(|(_, c)| giu.contains_key(c))
                    .collect();
                t.store(uv.len(), Ordering::Relaxed);

                let mut trung = Vec::new();
                for (i, (s, c)) in uv.iter().enumerate() {
                    if co_huy.load(Ordering::Relaxed) {
                        break;
                    }
                    x.store(i, Ordering::Relaxed);
                    let ha = match zalo_core::hash::sha256_toan_tep(Path::new(s)) {
                        Ok(h) => h,
                        Err(_) => continue,
                    };
                    if let Some(ds) = giu.get(c) {
                        for k in ds {
                            if zalo_core::hash::sha256_toan_tep(Path::new(k))
                                .map(|h| h == ha)
                                .unwrap_or(false)
                            {
                                trung.push(TepQuet {
                                    duong_dan: s.clone(),
                                    co: *c,
                                    giu_lai: k.clone(),
                                });
                                break;
                            }
                        }
                    }
                }
                let _ = g.send(Tin::QuetXong {
                    tep: trung,
                    loai: "BẢN TRÙNG LẶP".into(),
                    goc: goc.clone(),
                    goc_don_dep: vec![goc],
                    bi_chan: 0,
                    loi: 0,
                });
            },
        ));
    }

    /// Giải mã mười hai ảnh mẫu **ngoài luồng vẽ**.
    ///
    /// Đo trên dữ liệu Zalo thật ở bản gỡ lỗi: một tệp `.jxl` mất chừng ba giây.
    /// Mười hai tệp là quá lâu để chặn luồng vẽ — cửa sổ đứng hình giữa lượt xem
    /// trước là thứ khiến người ta bấm bừa hoặc tắt máy.
    fn chay_giai_ma_anh(&mut self) {
        let q = match &self.quet {
            Some(q) => q,
            None => return,
        };
        if !self.o_anh.is_empty() || self.anh_dang_giai_ma {
            return;
        }
        let ds: Vec<PathBuf> = q.tep.iter().map(|t| PathBuf::from(&t.duong_dan)).collect();
        // Hạt lấy từ dấu thời gian của lượt quét: cùng một lượt quét thì cùng
        // một mẫu, nên ảnh không nhảy loạn mỗi lần mở lại danh sách.
        let hat = q
            .dau_quet
            .bytes()
            .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
        self.anh_dang_giai_ma = true;
        self.viec_anh = Some(ViecNen::chay(
            "Đang giải mã ảnh xem trước…",
            move |g, co_huy, x, t| {
                let chon = crate::anh::chon_mau(ds.len(), hat);
                t.store(chon.len(), Ordering::Relaxed);
                let mut ra = Vec::new();
                for (i, k) in chon.iter().enumerate() {
                    if co_huy.load(Ordering::Relaxed) {
                        break;
                    }
                    x.store(i, Ordering::Relaxed);
                    let p = ds[*k].clone();
                    let (loai, anh) = crate::anh::giai_ma(&p);
                    ra.push(crate::anh::O {
                        duong_dan: p,
                        loai,
                        anh,
                    });
                }
                let _ = g.send(Tin::AnhXong(ra));
            },
        ));
    }

    fn chay_sao_luu(&mut self) {
        let q = match &self.quet {
            Some(q) => q,
            None => return,
        };
        let (ds, goc, loai) = (q.tep.clone(), q.goc.clone(), q.loai.clone());
        let dich = self.dich_sao_luu.trim().to_string();
        let toan_bo = self.xac_minh_toan_bo;
        self.man = ManHinh::DangLam;
        self.viec = Some(ViecNen::chay(
            "Đang sao lưu và xác minh…",
            move |g, _, x, t| {
                t.store(ds.len(), Ordering::Relaxed);
                x.store(0, Ordering::Relaxed);
                let dau = zalo_core::thoigian::luc_nay().dau_thoi_gian();
                let thu_muc = Path::new(&dich).join(&dau);
                match zalo_core::act::sao_luu(&ds, &goc, &thu_muc, toan_bo) {
                    Ok(r) => {
                        let _ = zalo_core::act::ghi_loai_quet(&thu_muc, &loai);
                        x.store(ds.len(), Ordering::Relaxed);
                        let _ = g.send(Tin::SaoLuuXong(
                            Box::new(r),
                            thu_muc.to_string_lossy().to_string(),
                        ));
                    }
                    Err(e) => {
                        let _ = g.send(Tin::Hong(format!("Không sao lưu được: {e}")));
                    }
                }
            },
        ));
    }

    fn chay_khoi_phuc(&mut self, bo: zalo_core::store::BoSaoLuu) {
        let dich = bo.ban_ke.goc_nguon.clone();
        let ghi_de = self.ghi_de_khi_khoi_phuc;
        let nhat_ky = sysinfo::thu_muc_cong_cu().join("logs");
        self.man = ManHinh::DangLam;
        self.viec = Some(ViecNen::chay("Đang khôi phục…", move |g, _, _, _| {
            match zalo_core::act::khoi_phuc(&bo.thu_muc, &dich, ghi_de, &nhat_ky) {
                Ok(r) => {
                    let _ = g.send(Tin::KhoiPhucXong(Box::new(r)));
                }
                Err(e) => {
                    let _ = g.send(Tin::Hong(format!("Không khôi phục được: {e}")));
                }
            }
        }));
    }

    fn chay_xoa(&mut self) {
        let q = match &self.quet {
            Some(q) => q,
            None => return,
        };
        let (ds, loai, dau, goc_don) = (
            q.tep.clone(),
            q.loai.clone(),
            q.dau_quet.clone(),
            q.goc_don_dep.clone(),
        );
        let vbv = self.vbv.clone();
        let nhat_ky = sysinfo::thu_muc_cong_cu().join("logs");
        let cat_cut =
            MucRuiRo::tu_loai_quet(&loai) != MucRuiRo::NguyHiem && loai != "BẢN TRÙNG LẶP";
        self.man = ManHinh::DangLam;
        self.viec = Some(ViecNen::chay("Đang xóa…", move |g, co_huy, x, t| {
            t.store(ds.len(), Ordering::Relaxed);
            let tien_do = |i: usize, _n: usize| x.store(i, Ordering::Relaxed);
            let r = zalo_core::act::xoa(
                &ds,
                &vbv,
                &loai,
                &dau,
                None,
                cat_cut,
                &nhat_ky,
                Some(&co_huy),
                Some(&tien_do),
            );
            match r {
                Ok(r) => {
                    let n = zalo_core::act::don_thu_muc_rong(&goc_don, true, &vbv);
                    let _ = g.send(Tin::XoaXong(Box::new(r), n));
                }
                Err(e) => {
                    let _ = g.send(Tin::Hong(format!("Không mở được nhật ký nên đã dừng: {e}")));
                }
            }
        }));
    }
}

// ==================================================================== tiện ích

/// Nút **có thể bị tắt**, dựng sao cho nó không nuốt mất một chặng Tab.
///
/// # Vì sao không dùng thẳng `ui.add_enabled`
///
/// egui 0.29 có một lỗ ở `Context::create_widget`:
///
/// ```text
/// if allow_focus && w.sense.focusable {
///     ctx.memory.interested_in_focus(w.id);      // chạy cho CẢ widget đang tắt
/// }
/// if allow_focus && (!w.enabled || …) {
///     mem.surrender_focus(w.id);                 // rồi lấy lại ngay
/// }
/// ```
///
/// Widget đang tắt vẫn **giành** tiêu điểm ở bước một — vì `give_to_next` đang
/// bật sau khi người dùng gõ Tab — rồi bị tước ở bước hai. Chặng Tab tiêu vào
/// khoảng giữa, `give_to_next` đã tắt, nên widget đứng **sau** nút tắt không
/// bao giờ nhận được tiêu điểm.
///
/// Đo tận nơi trên giao diện thật, không phải suy từ mã:
///
/// | Màn sao lưu | Vòng Tab đo được |
/// |---|---|
/// | "Bắt đầu sao lưu" đang TẮT | ô nhập → radio → radio → *(rỗng)* → ô nhập |
/// | "Bắt đầu sao lưu" đã BẬT   | ô nhập → radio → radio → Bắt đầu → ← Quay lại |
///
/// Nút `← Quay lại` **biến mất khỏi bàn phím** ở dòng trên. Chỗ này còn nặng
/// hơn ở trang chủ: `Lấy lại dung lượng ổ đĩa` tắt khi máy chưa cài Zalo, và
/// hai nút sau nó chết theo — cả màn hình không dùng được bằng bàn phím.
///
/// Nút đang tắt thì đằng nào cũng không giữ nổi tiêu điểm, nên khai thẳng nó
/// **không nhận tiêu điểm** là không mất gì, mà chuỗi Tab thì liền lại.
/// `on_disabled_hover_text` vẫn chạy — nó chỉ cần con trỏ rê qua (`ĐM-06`).
fn nut_co_the_tat(
    ui: &mut egui::Ui,
    bat: bool,
    nhan: impl Into<egui::WidgetText>,
) -> egui::Response {
    let n = egui::Button::new(nhan);
    if bat {
        ui.add(n)
    } else {
        ui.add_enabled(false, n.sense(egui::Sense::hover()))
    }
}

fn bao_cao_xoa(r: &zalo_core::act::KetQuaXoa, thu_muc_rong: usize) -> Vec<String> {
    let mut v = vec![
        format!("Đã xóa       : {} tệp", so(r.da_xoa as i64)),
        format!("Giải phóng   : {}", co(r.byte_thu_hoi as i64)),
        format!("Thư mục rỗng : {}", so(thu_muc_rong as i64)),
    ];
    if !r.hoan_tat {
        v.insert(
            0,
            "Đã dừng giữa chừng — số liệu dưới là phần đã làm xong.".into(),
        );
    }
    if r.bien_mat > 0 {
        v.push(format!(
            "Biến mất trước khi xóa: {} tệp",
            so(r.bien_mat as i64)
        ));
    }
    if r.vung_bao_ve > 0 {
        v.push(format!(
            "Chặn bởi vùng bảo vệ  : {} tệp",
            so(r.vung_bao_ve as i64)
        ));
    }
    if r.mat_ban_goc > 0 {
        v.push(format!(
            "Giữ lại vì mất bản gốc: {} tệp",
            so(r.mat_ban_goc as i64)
        ));
    }
    if r.that_bai > 0 {
        v.push(format!("Thất bại     : {} tệp", so(r.that_bai as i64)));
    }
    v.push(format!("Nhật ký      : {}", r.tep_nhat_ky.display()));
    v
}

fn tim_goc_zalo() -> Option<String> {
    let nen = Path::new(&std::env::var("APPDATA").ok()?).join("ZaloData\\media");
    let mut tot: Option<(u64, PathBuf)> = None;
    for e in std::fs::read_dir(&nen).ok()?.flatten() {
        let p = e.path().join("ZaloDownloads");
        if !p.is_dir() {
            continue;
        }
        let n = walk::duyet(&p).tep.len() as u64;
        if tot.as_ref().map(|(m, _)| n > *m).unwrap_or(true) {
            tot = Some((n, p));
        }
    }
    tot.map(|(_, p)| p.to_string_lossy().to_string())
}

fn dung_luat_bao_ve() -> Vec<Luat> {
    let b = |t: &str| std::env::var(t).unwrap_or_default();
    let (w, u, l, a) = (
        b("WINDIR"),
        b("USERPROFILE"),
        b("LOCALAPPDATA"),
        b("APPDATA"),
    );
    let goc_ht = sysinfo::goc_he_thong();
    let noi = |g: &str, p: &str| {
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
        sysinfo::thu_muc_cong_cu().to_string_lossy().to_string(),
    ];
    tat_ca.retain(|s| !s.trim().is_empty());
    let mut luat: Vec<Luat> = tat_ca
        .iter()
        .map(|p| Luat {
            duong_dan: p.trim_end_matches('\\').to_string(),
            muc: Muc::TatCa,
        })
        .collect();
    for p in [&w, &u, &l, &a, &goc_ht] {
        if p.trim().is_empty() {
            continue;
        }
        let mut t = p.trim_end_matches('\\').to_string();
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

/// **VM-01, cổng mức 1.** Định dạng số cố định, không đổi theo vùng miền.
fn so(n: i64) -> String {
    let am = n < 0;
    let mut s = n.unsigned_abs().to_string();
    let mut ra = String::new();
    while s.len() > 3 {
        let cat = s.split_off(s.len() - 3);
        ra.insert_str(0, &format!(".{cat}"));
    }
    ra.insert_str(0, &s);
    if am {
        ra.insert(0, '-');
    }
    ra
}

fn so_le(x: f64, le: usize) -> String {
    let t = format!("{x:.le$}");
    match t.split_once('.') {
        Some((ng, th)) => format!("{},{}", so(ng.parse().unwrap_or(0)), th),
        None => so(t.parse().unwrap_or(0)),
    }
}

/// Dung lượng dễ đọc, **kiểu Việt**: dấu chấm phân cách nghìn, dấu phẩy thập phân.
fn co(byte: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = 1024 * 1024;
    const GB: i64 = 1024 * 1024 * 1024;
    if byte >= GB {
        return format!("{} GB", so_le(byte as f64 / GB as f64, 2));
    }
    if byte >= MB {
        return format!("{} MB", so_le(byte as f64 / MB as f64, 1));
    }
    if byte >= KB {
        return format!("{} KB", so_le(byte as f64 / KB as f64, 0));
    }
    format!("{byte} B")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chạy một khung egui thật với ba nút, nút giữa **đang tắt**, rồi trả về
    /// mã của ba nút cùng ngữ cảnh để bơm phím Tab vào.
    fn ba_nut(tat_bang_helper: bool) -> (egui::Context, Vec<egui::Id>) {
        let ctx = egui::Context::default();
        ctx.set_fonts(egui::FontDefinitions::empty());
        let v = std::cell::RefCell::new(Vec::new());
        let _ = ctx.run(egui::RawInput::default(), |c| {
            egui::CentralPanel::default().show(c, |ui| {
                v.borrow_mut().push(ui.button("một").id);
                let giua = if tat_bang_helper {
                    nut_co_the_tat(ui, false, "hai")
                } else {
                    ui.add_enabled(false, egui::Button::new("hai"))
                };
                v.borrow_mut().push(giua.id);
                v.borrow_mut().push(ui.button("ba").id);
            });
        });
        (ctx, v.into_inner())
    }

    fn go_tab(ctx: &egui::Context, tat_bang_helper: bool) {
        let mut vao = egui::RawInput::default();
        vao.events.push(egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        let _ = ctx.run(vao, |c| {
            egui::CentralPanel::default().show(c, |ui| {
                let _ = ui.button("một");
                let _ = if tat_bang_helper {
                    nut_co_the_tat(ui, false, "hai")
                } else {
                    ui.add_enabled(false, egui::Button::new("hai"))
                };
                let _ = ui.button("ba");
            });
        });
    }

    /// **BP-01, cổng mức 1.** Nút đứng SAU một nút đang tắt vẫn phải tới được
    /// bằng Tab.
    ///
    /// Đây là phép thử đáng lẽ phải có từ mốc M5. Không có nó, giao diện ra
    /// bản phát hành với ba màn hình mà bàn phím không đi hết được, trong đó
    /// có trang chủ khi máy chưa cài Zalo. Xem [`nut_co_the_tat`].
    #[test]
    fn nut_dang_tat_khong_duoc_nuot_mat_mot_chang_tab() {
        let (ctx, id) = ba_nut(true);
        go_tab(&ctx, true);
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(id[0]),
            "Tab đầu tiên không vào nút thứ nhất"
        );
        go_tab(&ctx, true);
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(id[2]),
            "Tab thứ hai không tới được nút ĐỨNG SAU nút đang tắt — chặng Tab đã bị nuốt"
        );
    }

    /// Ghim **lỗ của egui 0.29**, thứ mà [`nut_co_the_tat`] sinh ra để vá.
    ///
    /// `Context::create_widget` gọi `interested_in_focus` cho cả widget đang
    /// tắt rồi mới `surrender_focus`, nên chặng Tab tiêu mất ở khoảng giữa.
    ///
    /// Ngày nào egui vá chỗ ấy thì phép thử này **đỏ**, và người nâng phiên bản
    /// buộc phải đọc lại đoạn này rồi bỏ hẳn cái vá đi thay vì để nó nằm lại
    /// mãi như một câu bùa không ai dám động.
    #[test]
    fn egui_029_van_con_nuot_chang_tab_o_nut_tat() {
        let (ctx, id) = ba_nut(false);
        go_tab(&ctx, false);
        assert_eq!(ctx.memory(|m| m.focused()), Some(id[0]));
        go_tab(&ctx, false);
        assert_eq!(
            ctx.memory(|m| m.focused()),
            None,
            "egui đã tự vá chỗ này — bỏ `nut_co_the_tat` đi và dùng lại `add_enabled`"
        );
    }

    /// **VM-01, cổng mức 1.** Chuỗi phải giống hệt nhau ở mọi vùng miền.
    ///
    /// Rust không có khái niệm vùng miền nên định dạng vốn đã cố định — phép
    /// thử này ghim điều đó lại, để ngày nào có người thay bằng một thư viện
    /// biết vùng miền thì nó đỏ ngay.
    #[test]
    fn dinh_dang_so_co_dinh_kieu_viet() {
        assert_eq!(so(0), "0");
        assert_eq!(so(999), "999");
        assert_eq!(so(1000), "1.000");
        assert_eq!(so(57351), "57.351");
        assert_eq!(so(1234567), "1.234.567");
        assert_eq!(so(-4321), "-4.321");
    }

    #[test]
    fn dinh_dang_dung_luong_kieu_viet() {
        assert_eq!(co(0), "0 B");
        assert_eq!(co(1023), "1023 B");
        assert_eq!(co(1024), "1 KB");
        assert_eq!(co(1024 * 1024), "1,0 MB");
        assert_eq!(co(1024 * 1024 * 1024), "1,00 GB");
        assert_eq!(co(1024 * 1024 * 1024 * 1234), "1.234,00 GB");
    }

    /// Bộ luật vùng bảo vệ của giao diện phải khớp bản dòng lệnh — cùng hai mức,
    /// và thư mục công cụ luôn bị chặn ở mức tất cả.
    #[test]
    fn bo_luat_bao_ve_co_du_hai_muc() {
        let l = dung_luat_bao_ve();
        assert!(l.iter().any(|r| matches!(r.muc, Muc::TatCa)));
        assert!(l.iter().any(|r| matches!(r.muc, Muc::Goc)));
    }

    /// Báo cáo xóa phải nói rõ khi lượt xóa **bị dừng giữa chừng**, và câu ấy
    /// phải nằm ở dòng ĐẦU, không lẫn xuống cuối.
    #[test]
    fn bao_cao_noi_ro_khi_bi_dung_giua_chung() {
        let mut r = zalo_core::act::KetQuaXoa {
            da_xoa: 5,
            ..Default::default()
        };
        r.hoan_tat = false;
        let v = bao_cao_xoa(&r, 0);
        assert!(v[0].contains("dừng giữa chừng"));

        r.hoan_tat = true;
        let v = bao_cao_xoa(&r, 0);
        assert!(!v[0].contains("dừng giữa chừng"));
    }
}
