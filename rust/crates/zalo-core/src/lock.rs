//! Khóa một tiến trình một lúc, dùng chung với bản PowerShell.
//!
//! Tên khóa ở [`crate::contract::LOCK_NAME`] là hợp đồng giữa hai bản (`R-16`).
//!
//! Ba chi tiết bản PowerShell đã chốt, bản Rust phải theo: mutex bị bỏ rơi do
//! tiến trình trước chết được xử lý như **đã nhận khóa** chứ không phải lỗi;
//! dựng khóa thất bại thì **không chặn người dùng**; và tệp khóa mang PID để
//! câu thông báo nói được ai đang giữ. Mốc **M1**.
//!
//! # Vì sao mô-đun này từng chỉ có tám dòng chú thích và không có mã
//!
//! Nó đúng là thế cho tới khi `Q15` bị đem ra đo. `contract.rs` khai `LOCK_NAME`
//! và có cả phép thử đối chiếu tên ấy với `ZaloCleanup.ps1` — nên nhìn vào thì
//! tưởng khóa đã có. Nhưng **không chỗ nào gọi tới nó**: bản Rust chưa bao giờ
//! xin khóa, và `zalo-gui`, `zalo-cli`, `ZaloCleanup.ps1` cùng chạy được trên
//! một tập tệp. Đó là mối đe dọa `B8`, hội đồng xếp **NẶNG**.
//!
//! Bài học lặp lại lần thứ tám của dự án này, chỉ khác hình dạng: **một hằng số
//! có phép thử không có nghĩa là cái nó đặt tên cho đã tồn tại.** Phép thử ở
//! `contract.rs` canh tên khóa khớp nhau; nó không canh có ai lấy khóa.

use crate::contract::LOCK_NAME;
use crate::sysinfo::{MutexDatTen, XinKhoa};

/// Tệp khóa mang PID, để câu thông báo nói được **ai** đang giữ.
///
/// Cùng đường dẫn với bản PowerShell — đây cũng là hợp đồng. Nội dung ba trường
/// ngăn bằng ký tự tab: `PID`, tên tiến trình, thời điểm mở theo `dd/MM/yyyy
/// HH:mm:ss`. Bản PowerShell đọc đúng ba trường ấy ở `Get-LockHolder`.
pub fn duong_dan_tep_khoa() -> std::path::PathBuf {
    std::env::temp_dir().join("zalocleanup.lock")
}

/// Khóa đang giữ. Nhả ra khi rơi khỏi phạm vi, hoặc khi gọi [`Khoa::nha`].
pub struct Khoa {
    mutex: Option<MutexDatTen>,
    co_tep: bool,
}

/// Kết quả xin khóa, ở mức **chính sách** chứ không phải mức Win32.
pub enum KetQuaKhoa {
    /// Đi tiếp được. Kèm khóa phải giữ sống suốt lượt chạy.
    DiTiep(Khoa),
    /// Một bản khác đang mở. Kèm câu mô tả ai đang giữ, để nói ra cho người dùng.
    BanKhacDangMo(String),
}

/// Xin khóa cho tiến trình hiện tại.
///
/// `ten_tien_trinh` đi vào tệp khóa và hiện lại trong câu thông báo của bản kia,
/// nên phải là tên người đọc hiểu được: `zalo-gui`, `zalo-cli`.
pub fn vao(ten_tien_trinh: &str) -> KetQuaKhoa {
    let (mutex, ket) = MutexDatTen::xin(LOCK_NAME);
    match ket {
        // Dựng không nổi mutex thì **không chặn người dùng**. Thà chạy còn hơn
        // không mở được công cụ vì một cơ chế phụ trợ hỏng. Bản PowerShell
        // `return $true` ở đúng ngã này.
        XinKhoa::KhongDungDuoc => KetQuaKhoa::DiTiep(Khoa {
            mutex: None,
            co_tep: false,
        }),
        XinKhoa::NguoiKhacGiu => KetQuaKhoa::BanKhacDangMo(ai_dang_giu()),
        XinKhoa::Duoc => {
            let co_tep = ghi_tep_khoa(ten_tien_trinh).is_ok();
            KetQuaKhoa::DiTiep(Khoa { mutex, co_tep })
        }
    }
}

fn ghi_tep_khoa(ten_tien_trinh: &str) -> std::io::Result<()> {
    let luc = crate::thoigian::luc_nay().dinh_dang();
    std::fs::write(
        duong_dan_tep_khoa(),
        format!("{}\t{}\t{}", std::process::id(), ten_tien_trinh, luc),
    )
}

/// Câu mô tả ai đang giữ khóa, đọc từ tệp khóa.
///
/// Tệp không đọc được thì vẫn phải trả về một câu dùng được — người dùng cần
/// biết là **có** bản khác đang mở, kể cả khi không biết bản nào.
pub fn ai_dang_giu() -> String {
    let tho = match std::fs::read_to_string(duong_dan_tep_khoa()) {
        Ok(s) => s,
        Err(_) => return "một bản khác đang mở".into(),
    };
    let p: Vec<&str> = tho.trim().split('\t').collect();
    if p.len() >= 3 {
        format!("tiến trình {} ({}), mở lúc {}", p[0], p[1], p[2])
    } else {
        "một bản khác đang mở".into()
    }
}

impl Khoa {
    /// Nhả khóa ngay, và dọn tệp khóa.
    ///
    /// Cuộc **bàn giao** của `RB-08` cần đúng hàm này: nhả khóa **trước** khi
    /// khởi chạy bản dòng lệnh. Không nhả trước thì bản kia mở lên, thấy khóa
    /// còn người giữ, rồi từ chối chạy — tức cái nút "Mở bản dòng lệnh" giao
    /// cho người dùng một công cụ không mở được.
    pub fn nha(&mut self) {
        if let Some(m) = &mut self.mutex {
            m.nha();
        }
        self.mutex = None;
        if self.co_tep {
            let _ = std::fs::remove_file(duong_dan_tep_khoa());
            self.co_tep = false;
        }
    }
}

impl Drop for Khoa {
    fn drop(&mut self) {
        self.nha();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tên khóa lấy thẳng từ hợp đồng, không viết lại ở đây. Viết lại là mở
    /// đường cho hai bản trôi khỏi nhau mà phép thử của `contract.rs` không thấy.
    #[test]
    fn ten_khoa_lay_tu_hop_dong() {
        assert_eq!(LOCK_NAME, crate::contract::LOCK_NAME);
        assert!(LOCK_NAME.starts_with(r"Local\"));
    }

    /// Xin khóa từ một LUỒNG KHÁC, trả về `(vào được không, câu mô tả)`.
    ///
    /// Phải là luồng khác chứ không phải lời gọi thứ hai trên cùng luồng: mutex
    /// của Win32 **đệ quy theo luồng**, nên luồng đang giữ xin lại thì được ngay
    /// và phép thử xanh mà chẳng chứng minh gì. Bản đầu của phép thử này dính
    /// đúng chỗ ấy — nó đỏ, và cái sai nằm ở phép thử chứ không ở mã.
    #[cfg(windows)]
    fn xin_o_luong_khac(ten: &'static str) -> (bool, String) {
        std::thread::spawn(move || match vao(ten) {
            KetQuaKhoa::DiTiep(k) => {
                drop(k);
                (true, String::new())
            }
            KetQuaKhoa::BanKhacDangMo(ai) => (false, ai),
        })
        .join()
        .expect("luồng phụ chết")
    }

    /// Bản thứ hai phải bị chặn, và câu thông báo phải nói được ai đang giữ.
    #[test]
    #[cfg(windows)]
    fn ban_thu_hai_bi_chan_va_noi_duoc_ai_dang_giu() {
        let mut a = match vao("phep-thu-a") {
            KetQuaKhoa::DiTiep(k) => k,
            KetQuaKhoa::BanKhacDangMo(ai) => {
                // Một bản thật đang mở trên máy này. Không phá phép thử vì
                // chuyện ấy, nhưng cũng đừng lặng lẽ báo xanh.
                eprintln!("bỏ qua: đã có bản khác giữ khóa — {ai}");
                return;
            }
        };
        let (vao_duoc, ai) = xin_o_luong_khac("phep-thu-b");
        assert!(
            !vao_duoc,
            "bản thứ hai vẫn vào được — khóa một tiến trình một lúc không có tác dụng"
        );
        assert!(
            ai.contains("phep-thu-a"),
            "câu thông báo không nói được ai đang giữ: {ai:?}"
        );

        // Nhả ra thì bản sau phải vào được. Đây là vế của `RB-08`: cuộc bàn giao
        // chỉ chạy được nếu nhả khóa thật sự trả khóa.
        a.nha();
        let (vao_duoc_sau, ai_sau) = xin_o_luong_khac("phep-thu-c");
        assert!(vao_duoc_sau, "đã nhả khóa mà bản sau vẫn bị chặn: {ai_sau}");
    }

    /// Tệp khóa mang đủ ba trường bản PowerShell đọc, ngăn bằng tab.
    #[test]
    #[cfg(windows)]
    fn tep_khoa_du_ba_truong_ngan_bang_tab() {
        let k = match vao("zalo-thu") {
            KetQuaKhoa::DiTiep(k) => k,
            KetQuaKhoa::BanKhacDangMo(_) => return,
        };
        let tho = std::fs::read_to_string(duong_dan_tep_khoa()).unwrap_or_default();
        let p: Vec<&str> = tho.trim().split('\t').collect();
        assert_eq!(p.len(), 3, "tệp khóa phải có ba trường: {tho:?}");
        assert_eq!(p[0], std::process::id().to_string());
        assert_eq!(p[1], "zalo-thu");
        assert!(
            ai_dang_giu().contains("zalo-thu"),
            "câu mô tả không đọc ra tên tiến trình"
        );
        drop(k);
        assert!(
            !duong_dan_tep_khoa().exists(),
            "nhả khóa rồi mà tệp khóa còn nằm lại — bản sau sẽ đọc ra người giữ đã chết"
        );
    }
}
