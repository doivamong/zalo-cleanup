//! Máy trạng thái của trang xác nhận xóa — **mười điều của `BP-05`**.
//!
//! # Vì sao đây là một mô-đun thuần, không phải mã vẽ
//!
//! An toàn của bản dòng lệnh đến phần lớn từ **ma sát**: phải quét mới xóa được,
//! phải gõ đủ chữ `XÓA`, phải đi qua nhiều màn hình. Giao diện đồ họa xóa sạch
//! ma sát đó — mọi thứ cách nhau một cú nhấp. Trang xác nhận là chỗ dựng lại
//! toàn bộ ma sát ấy, nên nó là **chi tiết nguy hiểm nhất của cả mốc M5**.
//!
//! Ma sát nằm trong mã vẽ thì không có phép thử nào canh được: không thể "giữ
//! phím Enter năm giây" trong một hàm `#[test]`. Nên toàn bộ luật nằm ở đây,
//! dưới dạng một máy trạng thái nhận **sự kiện** và trả **quyết định** — và
//! phép thử bơm sự kiện vào thẳng.
//!
//! Lớp vẽ chỉ được phép làm đúng hai việc: chuyển sự kiện thật thành [`SuKien`],
//! và hỏi [`TrangXacNhan::cho_bam_xoa`] xem có được bấm không.
//!
//! # Mười điều, và mã của chúng trong tệp này
//!
//! 1. Không có nút mặc định — Enter không kích hoạt gì → [`SuKien::Enter`]
//! 2. Ô nhập không submit khi Enter → cùng chỗ
//! 3. Nút xóa vô hiệu tới khi chuỗi khớp → [`TrangXacNhan::khop_cum_tu`]
//! 4. Thứ tự Tab: ô nhập → Hủy → Xóa → [`THU_TU_TAB`]
//! 5. Khóa mồi 600 ms tính từ **mỗi lần** nút chuyển sang bật → [`KHOA_MOI_MS`]
//! 6. Bỏ phím tự lặp; chỉ nhận một lần nhấn **trọn vẹn** → [`SuKien::PhimTuLap`]
//! 7. Esc = Hủy, luôn luôn → [`SuKien::Esc`]
//! 8. Không phím tắt nào trỏ vào nút xóa → không có mã nào sinh ra `BamXoa`
//!    ngoài [`SuKien::BamNutXoa`]
//! 9. Chặn dán → [`SuKien::Dan`]
//! 10. Bấm rồi thì không nhận thêm lần bấm nào → [`Trang::DangXoa`]

use zalo_core::confirm::khop_cum_xac_nhan;

/// Khóa mồi sau **mỗi lần** nút chuyển từ tắt sang bật.
///
/// Chống đúng một cảnh: người dùng đang giữ chuột hoặc phím ở màn trước, ký tự
/// cuối của cụm từ vừa khớp, nút bật lên ngay dưới con trỏ, và cú nhấp đang dở
/// rơi trúng nó. 600 ms là ngưỡng của hội đồng — đủ để tay người dừng lại, chưa
/// đủ để thành phiền.
pub const KHOA_MOI_MS: u64 = 600;

/// Thứ tự Tab. Nút phá hủy **dựng cuối cùng**, không bao giờ là nút đầu.
pub const THU_TU_TAB: [&str; 3] = ["ô nhập cụm từ", "Hủy", "Xóa vĩnh viễn"];

/// Trạng thái của cả trang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trang {
    /// Đang chờ người dùng gõ cụm từ.
    DangCho,
    /// Đã bấm xóa. Từ đây không nhận thêm lần bấm nào (điều 10).
    DangXoa,
    /// Đã hủy.
    DaHuy,
}

/// Sự kiện đi vào từ lớp vẽ. **Chỉ những sự kiện này**, không có đường nào khác.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuKien {
    /// Người dùng gõ thêm hoặc sửa nội dung ô nhập.
    Go(String),
    /// Nhấn Enter, ở bất kỳ đâu trên trang.
    Enter,
    /// Nhấn Esc.
    Esc,
    /// Dán bằng `Ctrl+V` hoặc chuột phải.
    Dan(String),
    /// Sự kiện phím **tự lặp** do giữ phím, không phải một lần nhấn mới.
    PhimTuLap,
    /// Một lần nhấn **trọn vẹn** lên nút xóa: cả nhấn xuống lẫn nhả ra đều xảy
    /// ra khi trang đang mở.
    BamNutXoa,
    /// Bấm nút Hủy.
    BamNutHuy,
    /// Thời gian trôi, tính bằng mili giây.
    ThoiGianTroi(u64),
}

/// Việc mà lớp vẽ phải làm sau một sự kiện.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuyetDinh {
    /// Không làm gì.
    KhongLam,
    /// Bắt đầu xóa thật.
    BatDauXoa,
    /// Đóng trang, không xóa gì.
    Huy,
    /// Từ chối dán, và nói ra lý do.
    TuChoiDan,
}

pub struct TrangXacNhan {
    /// Cụm từ phải gõ, dạng có dấu. Ví dụ `XÓA`.
    co_dau: String,
    /// Cùng cụm từ, dạng không dấu. Ví dụ `XOA`.
    khong_dau: String,
    o_nhap: String,
    trang: Trang,
    /// Mili giây còn lại của khóa mồi. `0` nghĩa là hết khóa.
    khoa_con_lai: u64,
    /// Lần trước cụm từ đã khớp chưa — để biết lúc nào là **chuyển sang bật**.
    khop_lan_truoc: bool,
    /// Đã từ chối dán bao nhiêu lần, để giao diện nói được lý do.
    pub so_lan_chan_dan: u32,
}

impl TrangXacNhan {
    pub fn moi(co_dau: &str, khong_dau: &str) -> Self {
        TrangXacNhan {
            co_dau: co_dau.to_string(),
            khong_dau: khong_dau.to_string(),
            o_nhap: String::new(),
            trang: Trang::DangCho,
            khoa_con_lai: 0,
            khop_lan_truoc: false,
            so_lan_chan_dan: 0,
        }
    }

    pub fn trang(&self) -> Trang {
        self.trang
    }
    pub fn o_nhap(&self) -> &str {
        &self.o_nhap
    }

    /// Nội dung ô nhập có khớp cụm từ không.
    ///
    /// Dùng thẳng [`khop_cum_xac_nhan`] của lõi — chốt ấy sống cùng phép thử của
    /// riêng nó và đã qua bộ đối chiếu song song với bản PowerShell. Viết lại
    /// phép so ở đây là mời hai chỗ trôi khỏi nhau.
    pub fn khop_cum_tu(&self) -> bool {
        khop_cum_xac_nhan(&self.o_nhap, &self.co_dau, &self.khong_dau)
    }

    /// **Nút xóa có được phép bấm ngay lúc này không.**
    ///
    /// Đây là hàm mà lớp vẽ phải hỏi trước khi cho bấm, và là chỗ duy nhất trả
    /// lời câu ấy.
    pub fn cho_bam_xoa(&self) -> bool {
        self.trang == Trang::DangCho && self.khop_cum_tu() && self.khoa_con_lai == 0
    }

    /// Lý do nút đang tắt, để trình đọc màn hình đọc được (`ĐM-06`).
    pub fn ly_do_nut_tat(&self) -> Option<&'static str> {
        if self.trang == Trang::DangXoa {
            return Some("đang xóa, không nhận thêm lần bấm nào");
        }
        if !self.khop_cum_tu() {
            return Some("cần gõ đúng chữ XÓA");
        }
        if self.khoa_con_lai > 0 {
            return Some("vừa bật, chờ một chút để tránh nhấp nhầm");
        }
        None
    }

    fn dat_o_nhap(&mut self, s: String) {
        self.o_nhap = s;
        let khop = self.khop_cum_tu();
        // Điều 5: khóa mồi tính từ MỖI LẦN chuyển sang bật, không phải một lần
        // duy nhất lúc mở trang. Gõ đúng, xóa một ký tự, gõ lại — lần bật thứ
        // hai cũng phải chờ, nếu không thì né được khóa bằng cách gõ dư rồi xóa.
        if khop && !self.khop_lan_truoc {
            self.khoa_con_lai = KHOA_MOI_MS;
        }
        if !khop {
            self.khoa_con_lai = 0;
        }
        self.khop_lan_truoc = khop;
    }

    /// Đưa một sự kiện vào và nhận lại việc phải làm.
    pub fn nhan(&mut self, sk: SuKien) -> QuyetDinh {
        // Điều 10: đã bấm rồi thì chỉ còn Esc có tác dụng, và Esc lúc này nghĩa
        // là dừng thao tác đang chạy chứ không phải hủy trang.
        if self.trang != Trang::DangCho {
            return QuyetDinh::KhongLam;
        }
        match sk {
            SuKien::Go(s) => {
                self.dat_o_nhap(s);
                QuyetDinh::KhongLam
            }
            // Điều 1 và 2: Enter không kích hoạt gì, bất kể tiêu điểm ở đâu và
            // bất kể cụm từ đã khớp hay chưa.
            SuKien::Enter => QuyetDinh::KhongLam,
            // Điều 7: Esc luôn là Hủy.
            SuKien::Esc | SuKien::BamNutHuy => {
                self.trang = Trang::DaHuy;
                QuyetDinh::Huy
            }
            // Điều 9: chặn dán. Người ta dán được cụm từ thì ma sát biến mất —
            // mà ma sát chính là thứ trang này sinh ra để tạo.
            SuKien::Dan(_) => {
                self.so_lan_chan_dan += 1;
                QuyetDinh::TuChoiDan
            }
            // Điều 6: phím tự lặp do giữ phím không phải một lần nhấn mới.
            SuKien::PhimTuLap => QuyetDinh::KhongLam,
            SuKien::BamNutXoa => {
                if self.cho_bam_xoa() {
                    self.trang = Trang::DangXoa;
                    QuyetDinh::BatDauXoa
                } else {
                    QuyetDinh::KhongLam
                }
            }
            SuKien::ThoiGianTroi(ms) => {
                self.khoa_con_lai = self.khoa_con_lai.saturating_sub(ms);
                QuyetDinh::KhongLam
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trang() -> TrangXacNhan {
        TrangXacNhan::moi("XÓA", "XOA")
    }

    /// Gõ đúng rồi chờ hết khóa mồi thì mới bấm được. Đường đi bình thường.
    fn san_sang() -> TrangXacNhan {
        let mut t = trang();
        t.nhan(SuKien::Go("XÓA".into()));
        t.nhan(SuKien::ThoiGianTroi(KHOA_MOI_MS));
        assert!(t.cho_bam_xoa());
        t
    }

    /// **Điều 1 và 2.** Enter không kích hoạt gì, kể cả khi mọi thứ đã sẵn sàng.
    ///
    /// Đây là phép thử quan trọng nhất của cả tệp: phép thử §8.1 số 1 của hội
    /// đồng là "giữ phím Enter liên tục từ Trang chủ tới lúc trang xác nhận mở,
    /// giữ thêm 5 giây — **0 tệp biến mất**".
    #[test]
    fn enter_khong_bao_gio_kich_hoat_xoa() {
        let mut t = san_sang();
        for _ in 0..300 {
            assert_eq!(t.nhan(SuKien::Enter), QuyetDinh::KhongLam);
            assert_eq!(t.trang(), Trang::DangCho);
        }
    }

    /// **Điều 6.** Phím tự lặp do giữ phím không được tính là một lần nhấn.
    #[test]
    fn phim_tu_lap_khong_kich_hoat_xoa() {
        let mut t = san_sang();
        for _ in 0..300 {
            assert_eq!(t.nhan(SuKien::PhimTuLap), QuyetDinh::KhongLam);
        }
        assert_eq!(t.trang(), Trang::DangCho);
    }

    /// **Điều 3.** Chưa gõ đúng thì bấm cũng không xóa.
    #[test]
    fn chua_go_dung_thi_bam_khong_an_thua() {
        let mut t = trang();
        for sai in ["", "xóa", "XÓAA", "XO", " ", "TÔI CHẤP NHẬN MẤT"] {
            t.nhan(SuKien::Go(sai.into()));
            t.nhan(SuKien::ThoiGianTroi(10_000));
            assert!(!t.cho_bam_xoa(), "chuỗi sai {sai:?} lại mở được nút");
            assert_eq!(t.nhan(SuKien::BamNutXoa), QuyetDinh::KhongLam);
            assert_eq!(t.trang(), Trang::DangCho);
        }
    }

    /// **TV-01.** Ba kiểu viết đều mở khóa, chữ thường thì không.
    #[test]
    fn ba_kieu_viet_deu_mo_khoa_chu_thuong_thi_khong() {
        for dung in ["XÓA", "XOÁ", "XOA"] {
            let mut t = trang();
            t.nhan(SuKien::Go(dung.into()));
            t.nhan(SuKien::ThoiGianTroi(KHOA_MOI_MS));
            assert!(t.cho_bam_xoa(), "kiểu viết {dung:?} phải mở khóa");
        }
        for sai in ["xóa", "xoa", "Xóa"] {
            let mut t = trang();
            t.nhan(SuKien::Go(sai.into()));
            t.nhan(SuKien::ThoiGianTroi(KHOA_MOI_MS));
            assert!(!t.cho_bam_xoa(), "chữ thường {sai:?} KHÔNG được mở khóa");
        }
    }

    /// **Điều 5.** Khóa mồi chặn cú bấm ngay khi nút vừa bật.
    #[test]
    fn khoa_moi_chan_cu_bam_ngay_khi_nut_vua_bat() {
        let mut t = trang();
        t.nhan(SuKien::Go("XÓA".into()));
        assert!(
            !t.cho_bam_xoa(),
            "nút bật ngay lập tức — khóa mồi mất tác dụng"
        );
        assert_eq!(t.nhan(SuKien::BamNutXoa), QuyetDinh::KhongLam);

        t.nhan(SuKien::ThoiGianTroi(KHOA_MOI_MS - 1));
        assert!(!t.cho_bam_xoa(), "mở khóa sớm hơn ngưỡng");

        t.nhan(SuKien::ThoiGianTroi(1));
        assert!(t.cho_bam_xoa());
        assert_eq!(t.nhan(SuKien::BamNutXoa), QuyetDinh::BatDauXoa);
    }

    /// **Điều 5, vế dễ quên nhất.** Khóa tính lại từ **mỗi lần** chuyển sang bật.
    ///
    /// Không có vế này thì né được khóa: gõ đúng, chờ hết khóa, xóa một ký tự
    /// rồi gõ lại — lần bật thứ hai không còn khóa nào.
    #[test]
    fn khoa_moi_tinh_lai_moi_lan_nut_chuyen_sang_bat() {
        let mut t = san_sang();
        // Xóa một ký tự rồi gõ lại.
        t.nhan(SuKien::Go("XÓ".into()));
        assert!(!t.cho_bam_xoa());
        t.nhan(SuKien::Go("XÓA".into()));
        assert!(
            !t.cho_bam_xoa(),
            "lần bật thứ hai KHÔNG bị khóa — né được khóa mồi bằng cách gõ dư rồi xóa"
        );
        t.nhan(SuKien::ThoiGianTroi(KHOA_MOI_MS));
        assert!(t.cho_bam_xoa());
    }

    /// **Điều 7.** Esc luôn là Hủy.
    #[test]
    fn esc_luon_la_huy() {
        let mut t = san_sang();
        assert_eq!(t.nhan(SuKien::Esc), QuyetDinh::Huy);
        assert_eq!(t.trang(), Trang::DaHuy);

        // Kể cả khi chưa gõ gì.
        let mut t = trang();
        assert_eq!(t.nhan(SuKien::Esc), QuyetDinh::Huy);
    }

    /// **Điều 9.** Dán bị chặn, và chặn rồi thì ô nhập vẫn nguyên.
    #[test]
    fn dan_bi_chan_va_khong_lam_ban_o_nhap() {
        let mut t = trang();
        assert_eq!(t.nhan(SuKien::Dan("XÓA".into())), QuyetDinh::TuChoiDan);
        assert_eq!(t.o_nhap(), "", "nội dung dán đã lọt vào ô nhập");
        assert!(!t.cho_bam_xoa());
        assert_eq!(t.so_lan_chan_dan, 1);
    }

    /// **Điều 10.** Bấm rồi thì không nhận thêm lần bấm nào.
    #[test]
    fn bam_roi_thi_khong_nhan_them_lan_bam_nao() {
        let mut t = san_sang();
        assert_eq!(t.nhan(SuKien::BamNutXoa), QuyetDinh::BatDauXoa);
        assert_eq!(t.trang(), Trang::DangXoa);
        for _ in 0..50 {
            assert_eq!(
                t.nhan(SuKien::BamNutXoa),
                QuyetDinh::KhongLam,
                "nhận lần bấm thứ hai — sẽ xóa hai lượt"
            );
        }
        assert!(!t.cho_bam_xoa());
    }

    /// **Điều 4.** Nút phá hủy đứng cuối trong thứ tự Tab, không bao giờ đầu.
    #[test]
    fn nut_pha_huy_dung_cuoi_trong_thu_tu_tab() {
        assert_eq!(THU_TU_TAB.len(), 3);
        assert!(THU_TU_TAB[0].contains("ô nhập"));
        assert_eq!(THU_TU_TAB[1], "Hủy");
        assert!(
            THU_TU_TAB[2].contains("Xóa"),
            "nút xóa phải là chặng Tab CUỐI cùng"
        );
    }

    /// **ĐM-06.** Nút tắt phải nói được lý do, không chỉ xám đi.
    #[test]
    fn nut_tat_noi_duoc_ly_do() {
        let mut t = trang();
        assert_eq!(t.ly_do_nut_tat(), Some("cần gõ đúng chữ XÓA"));
        t.nhan(SuKien::Go("XÓA".into()));
        assert_eq!(
            t.ly_do_nut_tat(),
            Some("vừa bật, chờ một chút để tránh nhấp nhầm")
        );
        t.nhan(SuKien::ThoiGianTroi(KHOA_MOI_MS));
        assert_eq!(t.ly_do_nut_tat(), None);
        t.nhan(SuKien::BamNutXoa);
        assert_eq!(
            t.ly_do_nut_tat(),
            Some("đang xóa, không nhận thêm lần bấm nào")
        );
    }

    /// Phép thử §8.1 số 1 của hội đồng, dựng lại bằng sự kiện: giữ Enter, giữ
    /// Space, và nhấp liên tục vào tọa độ nút Xóa — **0 tệp biến mất** cả ba lần.
    ///
    /// Nhấp liên tục là ca đáng sợ nhất: người dùng nhấp sẵn ở chỗ nút sắp hiện
    /// ra. Ở đây nó rơi vào khóa mồi.
    #[test]
    fn ba_phep_thu_ma_sat_cua_hoi_dong() {
        // ① giữ Enter năm giây
        let mut t = san_sang();
        for _ in 0..5000 {
            t.nhan(SuKien::Enter);
        }
        assert_eq!(t.trang(), Trang::DangCho);

        // ② giữ Space — tới lớp vẽ nó là phím tự lặp
        let mut t = san_sang();
        for _ in 0..5000 {
            t.nhan(SuKien::PhimTuLap);
        }
        assert_eq!(t.trang(), Trang::DangCho);

        // ③ nhấp liên tục vào chỗ nút Xóa TRONG LÚC gõ cụm từ
        let mut t = trang();
        let mut so_lan_xoa = 0;
        for buoc in ["X", "XÓ", "XÓA"] {
            t.nhan(SuKien::Go(buoc.into()));
            for _ in 0..100 {
                if t.nhan(SuKien::BamNutXoa) == QuyetDinh::BatDauXoa {
                    so_lan_xoa += 1;
                }
            }
        }
        assert_eq!(
            so_lan_xoa, 0,
            "nhấp liên tục lọt qua được — đây đúng là cảnh làm mất dữ liệu"
        );
    }

    /// **TV-10.** Cụm xác nhận không dịch được: đổi hằng số thì phép thử phải đỏ.
    ///
    /// Chốt tường minh vì bản dịch của cụm từ này là bản dịch duy nhất có thể
    /// gây mất dữ liệu — người dùng gõ thứ họ đọc trên màn hình.
    #[test]
    fn cum_xac_nhan_la_hang_so_khong_dich_duoc() {
        let bang = zalo_core::confirm::CUM_XAC_NHAN;
        assert!(
            bang.iter().any(|(a, b)| *a == "XÓA" && *b == "XOA"),
            "bảng cụm xác nhận đã đổi — đây là hợp đồng với người dùng, không phải chuỗi hiển thị"
        );
        assert!(bang
            .iter()
            .any(|(a, b)| *a == "TÔI CHẤP NHẬN MẤT" && *b == "TOI CHAP NHAN MAT"));
    }
}
