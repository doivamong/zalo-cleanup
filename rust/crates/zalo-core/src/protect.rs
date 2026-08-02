//! Vùng bảo vệ — lớp chặn cuối cùng trước khi một tệp bị xóa.
//!
//! Hai mức: `tất cả` chặn cả cây bên dưới, `gốc` chỉ chặn khi nhắm thẳng vào
//! chính nó. Thư mục gốc còn phải kiểm CHIỀU NGƯỢC: nhận một thư mục *chứa*
//! vùng bảo vệ cũng nguy hiểm y như nhận chính vùng bảo vệ.
//!
//! So chuỗi phải dùng **ordinal**, không theo vùng miền (`R-11`) — công cụ có
//! phép thử chạy dưới `vi-VN`, nơi bảng chữ so sánh khác.
//!
//! Mốc **M1**. Cổng: chạy lại bộ so sánh 57.144 đầu vào của phiên trước, lần
//! này đối chiếu PowerShell với Rust, phải ra 0 khác biệt.

use std::collections::HashSet;

/// Mức chặn của một luật.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Muc {
    /// Chặn chính nó **và mọi thứ bên dưới**.
    TatCa,
    /// Chỉ chặn khi nhắm thẳng vào chính thư mục đó; con vẫn cho phép.
    ///
    /// Dùng cho các gốc lớn như `%LOCALAPPDATA%`, nơi `%LOCALAPPDATA%\npm-cache`
    /// là mục hợp lệ nhưng bản thân `%LOCALAPPDATA%` thì không bao giờ được nhắm tới.
    Goc,
}

/// Một luật vùng bảo vệ.
#[derive(Clone, Debug)]
pub struct Luat {
    pub duong_dan: String,
    pub muc: Muc,
}

/// Viết hoa từng ký tự theo lối **đơn giản, một đổi một**.
///
/// Đây là chỗ dễ sai nhất của cả mô-đun. Bản PowerShell so bằng
/// `StringComparison::OrdinalIgnoreCase`, tức .NET viết hoa TỪNG KÝ TỰ bằng
/// `char.ToUpperInvariant` — một đổi một, không bao giờ nở dài ra.
///
/// `str::to_uppercase` của Rust thì viết hoa theo Unicode đầy đủ và CÓ THỂ nở
/// dài: `ß` thành `SS`, `ﬁ` thành `FI`. Dùng thẳng nó là lệch khỏi bản
/// PowerShell ở đúng những ký tự hiếm, và lệch trong một hàm canh cửa nghĩa là
/// một đường dẫn lọt lưới trên máy này mà không lọt trên máy khác.
///
/// Nên: ký tự nào viết hoa ra đúng một ký tự thì lấy; nở dài ra thì giữ nguyên
/// ký tự gốc — đúng như .NET làm.
fn hoa_don_gian(s: &str) -> String {
    let mut ra = String::with_capacity(s.len());
    for c in s.chars() {
        let mut it = c.to_uppercase();
        match (it.next(), it.next()) {
            (Some(u), None) => ra.push(u),
            _ => ra.push(c),
        }
    }
    ra
}

/// Chỉ mục tra cứu vùng bảo vệ, dựng sẵn một lần rồi tra nhiều lần.
///
/// Tách hai loại luật thành hai cấu trúc đúng với bản chất phép so:
/// `so_bang` cho phép so bằng, `so_tien_to` cho phép so tiền tố. Luật
/// [`Muc::Goc`] chỉ vào `so_bang` và **không bao giờ** vào `so_tien_to`.
///
/// `Clone` để đưa được **nguyên bộ luật** sang luồng nền: việc quét và việc xóa
/// chạy ngoài luồng giao diện, mà chúng phải hỏi đúng bộ luật ấy. Chép nguyên
/// bộ chứ không mượn tham chiếu — mượn thì vòng đời buộc luồng nền sống ngắn
/// hơn giao diện, và cách lách thường gặp là dựng lại bộ luật ở luồng kia, tức
/// hai bộ luật có thể khác nhau mà không ai hay.
#[derive(Clone, Debug)]
pub struct VungBaoVe {
    luat: Vec<Luat>,
    ten_bao_ve: Vec<String>,
    goc_du_lieu: String,
    so_bang: HashSet<String>,
    so_tien_to: Vec<String>,
}

impl VungBaoVe {
    /// Dựng chỉ mục từ bộ luật, thư mục dữ liệu Zalo, và các tên bị bảo vệ
    /// bên trong thư mục đó (`Database`, `Partitions`).
    ///
    /// `goc_du_lieu` phải được chuẩn hóa TRƯỚC khi truyền vào — dạng đầy đủ,
    /// tên dài. Đưa vào dạng ngắn 8.3 là vùng bảo vệ biến mất không một lời
    /// cảnh báo; đó là một lỗ hổng thật đã xảy ra ở bản PowerShell.
    pub fn dung(luat: &[Luat], goc_du_lieu: &str, ten_bao_ve: &[&str]) -> Self {
        let mut so_bang: HashSet<String> = HashSet::new();
        let mut so_tien_to: Vec<String> = Vec::new();

        for l in luat {
            so_bang.insert(hoa_don_gian(&l.duong_dan));
            if l.muc == Muc::TatCa {
                so_tien_to.push(hoa_don_gian(&l.duong_dan));
            }
        }

        if !goc_du_lieu.trim().is_empty() {
            for n in ten_bao_ve {
                let p = format!("{}\\{}", goc_du_lieu.trim_end_matches('\\'), n);
                so_bang.insert(hoa_don_gian(&p));
                so_tien_to.push(hoa_don_gian(&p));
            }
        }

        Self {
            luat: luat.to_vec(),
            ten_bao_ve: ten_bao_ve.iter().map(|s| s.to_string()).collect(),
            goc_du_lieu: goc_du_lieu.to_string(),
            so_bang,
            so_tien_to,
        }
    }

    /// Đường dẫn này có bị chặn không.
    ///
    /// **Hàm canh cửa: không bao giờ được hoảng.** Ném lỗi ở đây là bỏ trống
    /// cửa. Cắt thư mục bằng `rfind` chứ không dùng hàm tách đường dẫn của thư
    /// viện, vì hàm kia có thể từ chối đường dẫn dị dạng.
    pub fn chan(&self, duong_dan: &str) -> bool {
        let hoa = hoa_don_gian(duong_dan);
        if self.so_bang.contains(&hoa) {
            return true;
        }

        // Nằm dưới vùng bảo vệ hay không CHỈ phụ thuộc thư mục chứa nó.
        let i = match duong_dan.rfind('\\') {
            Some(i) => i,
            None => return false,
        };
        let thu_muc = hoa_don_gian(&duong_dan[..i]);

        for p in &self.so_tien_to {
            if thu_muc == *p || thu_muc.starts_with(&format!("{p}\\")) {
                return true;
            }
        }
        false
    }

    /// Dùng cho **thư mục gốc**: gốc quét, gốc dọn thư mục rỗng, đường dẫn
    /// trong `catalog.json`.
    ///
    /// Ngoài việc hỏi như [`Self::chan`], nó chặn thêm **chiều ngược**: nhận
    /// một thư mục *chứa* vùng bảo vệ cũng nguy hiểm y như nhận chính vùng bảo
    /// vệ. `catalog.json` là tệp người dùng sửa được, nên một mục ghi `%WINDIR%`
    /// phải bị chặn ngay ở đây.
    pub fn chan_thu_muc_goc(&self, duong_dan: &str) -> bool {
        if self.chan(duong_dan) {
            return true;
        }
        let p = hoa_don_gian(duong_dan.trim_end_matches('\\'));
        let tien_to = format!("{p}\\");

        for l in &self.luat {
            if hoa_don_gian(&l.duong_dan).starts_with(&tien_to) {
                return true;
            }
        }
        if self.goc_du_lieu.trim().is_empty() {
            return false;
        }
        for n in &self.ten_bao_ve {
            let q = format!("{}\\{}", self.goc_du_lieu.trim_end_matches('\\'), n);
            if hoa_don_gian(&q).starts_with(&tien_to) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luat_thu() -> Vec<Luat> {
        vec![
            Luat {
                duong_dan: r"C:\Windows\System32".into(),
                muc: Muc::TatCa,
            },
            Luat {
                duong_dan: r"C:\Windows\WinSxS".into(),
                muc: Muc::TatCa,
            },
            Luat {
                duong_dan: r"C:\pagefile.sys".into(),
                muc: Muc::TatCa,
            },
            Luat {
                duong_dan: r"C:\Users\ADMIN\AppData\Local".into(),
                muc: Muc::Goc,
            },
            Luat {
                duong_dan: r"C:\".into(),
                muc: Muc::Goc,
            },
        ]
    }

    fn vbv() -> VungBaoVe {
        VungBaoVe::dung(
            &luat_thu(),
            r"C:\Users\ADMIN\AppData\Roaming\ZaloData",
            &["Database", "Partitions"],
        )
    }

    #[test]
    fn muc_tat_ca_chan_ca_cay_ben_duoi() {
        let v = vbv();
        assert!(v.chan(r"C:\Windows\System32"));
        assert!(v.chan(r"C:\Windows\System32\drivers\etc\hosts"));
        assert!(v.chan(r"c:\windows\system32\CON.TXT"));
    }

    #[test]
    fn muc_goc_chi_chan_chinh_no_khong_chan_con() {
        let v = vbv();
        assert!(v.chan(r"C:\Users\ADMIN\AppData\Local"));
        assert!(!v.chan(r"C:\Users\ADMIN\AppData\Local\npm-cache\x"));
    }

    #[test]
    fn ten_gan_giong_khong_bi_chan() {
        let v = vbv();
        assert!(!v.chan(r"C:\Windows\System32x\a.txt"));
        assert!(!v.chan(r"C:\Windows\System32_khac\a.txt"));
    }

    #[test]
    fn vung_bao_ve_theo_goc_du_lieu() {
        let v = vbv();
        assert!(v.chan(r"C:\Users\ADMIN\AppData\Roaming\ZaloData\Database"));
        assert!(v.chan(r"C:\Users\ADMIN\AppData\Roaming\ZaloData\Database\_production\chat.db"));
        assert!(v.chan(r"C:\Users\ADMIN\AppData\Roaming\ZaloData\Partitions\session\p1"));
        assert!(!v.chan(r"C:\Users\ADMIN\AppData\Roaming\ZaloData\DatabaseX\z.txt"));
    }

    #[test]
    fn dau_vao_di_dang_khong_lam_ham_hoang() {
        let v = vbv();
        assert!(!v.chan(""));
        assert!(!v.chan("khong_co_gach_cheo"));
        assert!(!v.chan("\\"));
        assert!(v.chan(r"C:\"));
    }

    #[test]
    fn chieu_nguoc_chi_ap_cho_thu_muc_goc() {
        let v = vbv();
        // C:\Windows CHỨA vùng bảo vệ nên không được nhận làm gốc quét,
        // nhưng một tệp nằm ngay trong C:\Windows thì không bị chặn.
        assert!(v.chan_thu_muc_goc(r"C:\Windows"));
        assert!(!v.chan(r"C:\Windows\notepad.exe"));
    }

    #[test]
    fn hoa_don_gian_khong_no_dai_ra() {
        // .NET ToUpperInvariant giữ nguyên ß; str::to_uppercase của Rust đổi
        // thành SS. Giữ nguyên mới đúng bản PowerShell.
        assert_eq!(hoa_don_gian("ß"), "ß");
        assert_eq!(hoa_don_gian("straße"), "STRAßE");
        assert_eq!(hoa_don_gian("Tài Liệu"), "TÀI LIỆU");
    }

    #[test]
    fn goc_du_lieu_rong_thi_khong_co_vung_theo_goc() {
        let v = VungBaoVe::dung(&luat_thu(), "", &["Database", "Partitions"]);
        assert!(!v.chan(r"C:\Users\ADMIN\AppData\Roaming\ZaloData\Database\x"));
        assert!(v.chan(r"C:\Windows\System32\x"));
    }
}
