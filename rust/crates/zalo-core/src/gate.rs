//! Chốt phải qua trước khi xóa.
//!
//! `Test-KeeperAlive` — bản giữ lại của cặp trùng lặp còn sống và còn đúng cỡ.
//! `Test-BackupClean` — sao lưu **không lỗi VÀ trọn vẹn**, hai vế chứ không một.
//!
//! Cả hai từng là lỗ hổng thật trong bản PowerShell, xem `docs/viec-con-lai.md`.
//!
//! Tách riêng khỏi [`crate::act`] là có chủ ý: khe hở giữa lúc quét và lúc xóa
//! không dựng lại được bằng phép thử đầu-cuối, nên chốt phải gọi thẳng được.
//! Và phải có phép thử canh cả chỗ **nối dây**, không chỉ canh hàm. Mốc **M1**.

/// Bản giữ lại của một cặp trùng lặp còn sống và còn đúng cỡ hay không.
///
/// Chuỗi rỗng nghĩa là kết quả quét này **không phải** chế độ khử trùng lặp,
/// nên không có ràng buộc nào — trả `true` để các chế độ khác đi qua.
///
/// # Vì sao phải kiểm lại dù lúc quét đã đối chiếu SHA-256 toàn tệp
///
/// Giữa lúc quét và lúc xóa có một khe hở: người dùng xóa hội thoại trong Zalo,
/// hoặc Zalo tự dọn, mà kết quả quét được giữ tới hai giờ. Bản giữ lại biến mất
/// trong khe đó thì tệp sắp bị xóa không còn là bản thừa nữa mà là bản **duy
/// nhất**.
///
/// Nặng hơn vẻ ngoài vì chế độ khử trùng lặp cố ý dùng xác nhận **nhẹ** — chỉ
/// `c/k` chứ không bắt gõ `XÓA` — và mức nhẹ ấy chỉ chính đáng nhờ tiền đề
/// "bạn không mất gì, còn một bản giống hệt". Tiền đề sai thì phải dừng tay.
///
/// # Vì sao chỉ so tồn tại và cỡ, không băm lại
///
/// Băm lại nghĩa là đọc trọn cả hai tệp cho từng cặp, tức **nhân đôi** lượng
/// đọc đĩa của cả lượt xóa. Lúc quét đã đối chiếu SHA-256 toàn tệp rồi; ở đây
/// chỉ cần bắt được bản giữ lại đã biến mất hoặc đã đổi.
pub fn ban_giu_lai_con_song(duong_dan: &str, co_mong_doi: u64) -> bool {
    if duong_dan.is_empty() {
        return true;
    }
    match std::fs::metadata(duong_dan) {
        Ok(m) if m.is_file() => m.len() == co_mong_doi,
        _ => false,
    }
}

/// Kết quả của một lượt sao lưu, đủ để quyết định có mở khóa bước xóa không.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KetQuaSaoLuu {
    /// Dấu thời gian của lượt quét mà bản sao lưu này phục vụ.
    pub dau_quet: String,
    /// Tổng số tệp lẽ ra phải chép.
    pub tong: u64,
    /// Số tệp chép được.
    pub xong: u64,
    /// Số tệp chép lỗi.
    pub loi_chep: u64,
    /// Số tệp xác minh lỗi.
    pub loi_xac_minh: u64,
    /// Ổ đích hết chỗ giữa chừng.
    pub het_cho: bool,
}

/// Bản sao lưu có **sạch** không — tức có được mở khóa bước xóa không.
///
/// # Sạch nghĩa là KHÔNG LỖI **VÀ** TRỌN VẸN. Hai vế, không phải một.
///
/// Vế thứ hai mới là vế dễ mất. Khi ổ đích hết chỗ giữa chừng, vòng chép thoát
/// sớm **trước** lúc kịp tăng số lỗi, nên chỉ xét số lỗi thì một bản sao lưu
/// thiếu tệp vẫn được chấm là sạch — và bước xóa được mở khóa cho một đường lui
/// không tồn tại. Đó là một lỗ hổng thật đã xảy ra ở bản PowerShell.
pub fn sao_luu_sach(bk: Option<&KetQuaSaoLuu>, dau_quet: &str) -> bool {
    let bk = match bk {
        Some(b) => b,
        None => return false,
    };
    if bk.dau_quet != dau_quet {
        return false;
    }
    if bk.loi_chep != 0 || bk.loi_xac_minh != 0 {
        return false;
    }
    if bk.het_cho {
        return false;
    }
    bk.xong == bk.tong
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn bk_mau() -> KetQuaSaoLuu {
        KetQuaSaoLuu {
            dau_quet: "S1".into(),
            tong: 100,
            xong: 100,
            loi_chep: 0,
            loi_xac_minh: 0,
            het_cho: false,
        }
    }

    #[test]
    fn sao_luu_sach_dung_ca_tam_ca() {
        // Bảng này khớp từng dòng với bảng trong ZaloCleanup.Tests.ps1.
        let ca: Vec<(KetQuaSaoLuu, bool, &str)> = vec![
            (bk_mau(), true, "đủ 100/100, không lỗi"),
            (
                KetQuaSaoLuu {
                    het_cho: true,
                    xong: 40,
                    ..bk_mau()
                },
                false,
                "hết chỗ giữa chừng — CA CHÍNH",
            ),
            (
                KetQuaSaoLuu {
                    het_cho: true,
                    ..bk_mau()
                },
                false,
                "cờ hết chỗ tự nó phải chặn",
            ),
            (
                KetQuaSaoLuu {
                    xong: 40,
                    ..bk_mau()
                },
                false,
                "thiếu tệp mà số lỗi vẫn 0",
            ),
            (
                KetQuaSaoLuu {
                    loi_chep: 1,
                    ..bk_mau()
                },
                false,
                "có lỗi chép",
            ),
            (
                KetQuaSaoLuu {
                    loi_xac_minh: 1,
                    ..bk_mau()
                },
                false,
                "có lỗi xác minh",
            ),
            (
                KetQuaSaoLuu {
                    dau_quet: "S2".into(),
                    ..bk_mau()
                },
                false,
                "sao lưu của lượt quét khác",
            ),
        ];
        for (bk, mong_doi, vi_sao) in ca {
            assert_eq!(sao_luu_sach(Some(&bk), "S1"), mong_doi, "{vi_sao}");
        }
        assert!(!sao_luu_sach(None, "S1"), "chưa sao lưu");
    }

    #[test]
    fn ban_giu_lai_con_song_dung_moi_ca() {
        let t = std::env::temp_dir().join(format!("zgate_{}", std::process::id()));
        std::fs::create_dir_all(&t).unwrap();
        let goc = t.join("goc");
        let cut = t.join("goc_bi_cut");
        std::fs::File::create(&goc)
            .unwrap()
            .write_all(&[0u8; 5000])
            .unwrap();
        std::fs::File::create(&cut)
            .unwrap()
            .write_all(&[0u8; 4999])
            .unwrap();

        assert!(
            ban_giu_lai_con_song(goc.to_str().unwrap(), 5000),
            "còn và đúng cỡ"
        );
        assert!(
            !ban_giu_lai_con_song(t.join("khong_ton_tai").to_str().unwrap(), 5000),
            "bản gốc biến mất"
        );
        assert!(!ban_giu_lai_con_song(cut.to_str().unwrap(), 5000), "đổi cỡ");
        assert!(
            ban_giu_lai_con_song("", 5000),
            "không phải chế độ trùng lặp"
        );
        assert!(
            !ban_giu_lai_con_song(t.to_str().unwrap(), 5000),
            "là thư mục chứ không phải tệp"
        );

        std::fs::remove_dir_all(&t).ok();
    }
}
