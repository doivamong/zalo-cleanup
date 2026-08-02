//! SHA-256 toàn tệp và chữ ký nhanh.
//!
//! Đo trên máy thật: đĩa cho 41 MB/s một luồng và 53 MB/s tám luồng, trong khi
//! SHA-256 chạy 840 MB/s khi dữ liệu đã ở trong RAM. **Nút cổ chai là đĩa,
//! không phải CPU** — đừng đặt kỳ vọng vào việc băm nhanh hơn.
//!
//! Tệp từ 128 KB trở xuống thì chữ ký nhanh đã đọc trọn cả tệp, nên nó CHÍNH
//! LÀ SHA-256 toàn tệp; đừng đọc lại lần nữa. Mốc **M2**.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Một nửa của bộ đệm chữ ký nhanh: 64 KB đầu và 64 KB cuối.
pub const KHOI: usize = 65536;

/// Ngưỡng dưới ngưỡng này thì chữ ký nhanh đọc trọn cả tệp.
pub const NGUONG_DOC_TRON: u64 = (KHOI * 2) as u64;

/// Chuỗi hex **viết HOA, không dấu ngăn** — đúng dạng bản PowerShell sinh ra
/// bằng `[BitConverter]::ToString(...).Replace('-','')`.
fn hex_hoa(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        s.push_str(&format!("{x:02X}"));
    }
    s
}

/// SHA-256 toàn bộ nội dung tệp, trả về hex viết hoa **không có tiền tố**.
///
/// Tương ứng `Get-Sha256Full` của bản PowerShell.
pub fn sha256_toan_tep(duong_dan: &Path) -> std::io::Result<String> {
    let mut f = File::open(duong_dan)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex_hoa(&h.finalize()))
}

/// Chữ ký nhanh: 64 KB đầu + 64 KB cuối.
///
/// # Hai tiền tố, và vì sao chúng bắt buộc
///
/// Tệp từ **128 KB trở xuống** thì phép này đọc trọn cả tệp, nên kết quả CHÍNH
/// LÀ SHA-256 toàn tệp — trả tiền tố `FULL:`. Tệp lớn hơn thì chỉ là chữ ký một
/// phần — trả tiền tố `Q:`.
///
/// Hai tiền tố khác nhau để chữ ký một phần **không bao giờ** bị đem so với chữ
/// ký đầy đủ rồi kết luận nhầm là trùng nội dung.
pub fn chu_ky_nhanh(duong_dan: &Path) -> std::io::Result<String> {
    let mut f = File::open(duong_dan)?;
    let co = f.metadata()?.len();

    if co <= NGUONG_DOC_TRON {
        let mut h = Sha256::new();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        h.update(&buf);
        return Ok(format!("FULL:{}", hex_hoa(&h.finalize())));
    }

    // Bộ đệm 128 KB, băm NGUYÊN cả bộ đệm kể cả phần chưa đầy — giống hệt bản
    // PowerShell, vốn cấp phát mảng 128 KB rồi băm cả mảng. Đọc cho bằng đủ chứ
    // không tin một lần Read trả về đúng số byte đã xin: đọc thiếu thì đuôi bộ
    // đệm còn rác, và hai tệp giống hệt nhau có thể ra hai chữ ký khác nhau.
    let mut buf = vec![0u8; KHOI * 2];
    doc_cho_du(&mut f, &mut buf[..KHOI])?;
    f.seek(SeekFrom::End(-(KHOI as i64)))?;
    doc_cho_du(&mut f, &mut buf[KHOI..])?;

    let mut h = Sha256::new();
    h.update(&buf);
    Ok(format!("Q:{}", hex_hoa(&h.finalize())))
}

fn doc_cho_du(f: &mut File, dich: &mut [u8]) -> std::io::Result<()> {
    let mut da = 0usize;
    while da < dich.len() {
        let n = f.read(&mut dich[da..])?;
        if n == 0 {
            break;
        }
        da += n;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tam(ten: &str, noi_dung: &[u8]) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("zhash_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join(ten);
        std::fs::write(&p, noi_dung).unwrap();
        p
    }

    #[test]
    fn sha256_khop_gia_tri_da_biet() {
        // SHA-256 của chuỗi rỗng, viết hoa như bản PowerShell.
        let p = tam("rong.bin", b"");
        assert_eq!(
            sha256_toan_tep(&p).unwrap(),
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
        );
        let p = tam("abc.bin", b"abc");
        assert_eq!(
            sha256_toan_tep(&p).unwrap(),
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        );
    }

    #[test]
    fn tep_nho_thi_chu_ky_nhanh_chinh_la_bam_toan_tep() {
        let p = tam("nho.bin", &vec![7u8; 1000]);
        let nhanh = chu_ky_nhanh(&p).unwrap();
        let day_du = sha256_toan_tep(&p).unwrap();
        assert_eq!(nhanh, format!("FULL:{day_du}"));
    }

    #[test]
    fn dung_nguong_128kb_van_la_full() {
        let p = tam("nguong.bin", &vec![3u8; KHOI * 2]);
        assert!(chu_ky_nhanh(&p).unwrap().starts_with("FULL:"));
        let p = tam("tren_nguong.bin", &vec![3u8; KHOI * 2 + 1]);
        assert!(chu_ky_nhanh(&p).unwrap().starts_with("Q:"));
    }

    #[test]
    fn hai_tien_to_khong_bao_gio_lan_nhau() {
        let nho = tam("a_nho.bin", &[1u8; 100]);
        let lon = tam("a_lon.bin", &[1u8; KHOI * 4]);
        let a = chu_ky_nhanh(&nho).unwrap();
        let b = chu_ky_nhanh(&lon).unwrap();
        assert!(a.starts_with("FULL:") && b.starts_with("Q:"));
        assert_ne!(a, b);
    }

    #[test]
    fn tep_lon_khac_nhau_o_duoi_thi_chu_ky_nhanh_van_khac() {
        let mut a = [9u8; KHOI * 4];
        let mut b = a;
        *a.last_mut().unwrap() = 1;
        *b.last_mut().unwrap() = 2;
        let pa = tam("duoi_a.bin", &a);
        let pb = tam("duoi_b.bin", &b);
        assert_ne!(chu_ky_nhanh(&pa).unwrap(), chu_ky_nhanh(&pb).unwrap());
    }
}
