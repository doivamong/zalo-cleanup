//! Đo **cổng ③ của mốc M2**: quét cây Zalo thật phải xong trong không quá 1,5 giây.
//!
//! Bản PowerShell hiện mất **5,2 giây** cho cùng việc, sau khi đã được tối ưu
//! 20 lần từ mốc 105 giây ban đầu.
//!
//! Cố ý để ở dạng `example` chứ không phải `#[test]`: mốc thời gian phụ thuộc
//! máy, mà máy chủ CI thì chậm và tải không đều. Ghim một ngưỡng giây vào bộ
//! test là mời một phép thử chập chờn — và một phép thử chập chờn thì sớm muộn
//! cũng bị người ta tắt đi, kéo theo cả những phép thử thật nằm cạnh.
//!
//! Chạy:
//! ```text
//! cargo run --release --example do_toc_do -p zalo-core
//! ```

use std::time::Instant;
use zalo_core::protect::{Luat, Muc, VungBaoVe};
use zalo_core::scan::BoLoc;

fn main() {
    let goc = std::env::args().nth(1).unwrap_or_else(|| {
        std::env::var("ZALO_DOI_CHIEU_GOC").unwrap_or_else(|_| {
            r"C:\Users\ADMIN\AppData\Roaming\ZaloData\media\2068096368017928379\ZaloDownloads"
                .to_string()
        })
    });
    if !std::path::Path::new(&goc).is_dir() {
        eprintln!("không thấy cây để đo: {goc}");
        std::process::exit(2);
    }

    // Bộ luật vùng bảo vệ tối thiểu, đủ để phép đo chạm cùng đường mã như lượt
    // quét thật. Con số chính xác của bộ luật không ảnh hưởng thời gian.
    let w = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    let luat = vec![
        Luat {
            duong_dan: format!(r"{w}\System32"),
            muc: Muc::TatCa,
        },
        Luat {
            duong_dan: format!(r"{w}\WinSxS"),
            muc: Muc::TatCa,
        },
        Luat {
            duong_dan: r"D:\zalo-tool".into(),
            muc: Muc::TatCa,
        },
    ];
    let vbv = VungBaoVe::dung(
        &luat,
        r"C:\Users\ADMIN\AppData\Roaming\ZaloData",
        &["Database", "Partitions"],
    );
    let loc = BoLoc {
        giu_rescache: true,
        goc: goc.clone(),
        ..Default::default()
    };

    // Lượt làm nóng bộ đệm metadata, để đo đúng thứ cần đo chứ không đo lần đọc
    // đĩa nguội đầu tiên — bản PowerShell cũng được đo trong cùng điều kiện.
    let _ = zalo_core::walk::duyet(std::path::Path::new(&goc));

    let mut lan: Vec<f64> = Vec::new();
    let mut nhan = 0usize;
    let mut chan = 0usize;
    let mut tong = 0usize;

    for _ in 0..3 {
        let t0 = Instant::now();
        let r = zalo_core::walk::duyet(std::path::Path::new(&goc));
        let mut n = 0usize;
        let mut c = 0usize;
        for t in &r.tep {
            let s = t.duong_dan.to_string_lossy();
            if vbv.chan(&s) {
                c += 1;
                continue;
            }
            if loc.qua(&s) {
                n += 1;
            }
        }
        lan.push(t0.elapsed().as_secs_f64());
        nhan = n;
        chan = c;
        tong = r.tep.len();
    }

    let nhanh_nhat = lan.iter().cloned().fold(f64::MAX, f64::min);
    println!("cây      : {goc}");
    println!("duyệt    : {tong} tệp · chặn {chan} · qua bộ lọc {nhan}");
    println!(
        "thời gian: {}",
        lan.iter()
            .map(|x| format!("{x:.3}s"))
            .collect::<Vec<_>>()
            .join(" · ")
    );
    println!();
    println!("nhanh nhất : {nhanh_nhat:.3} s");
    println!("bản PowerShell : 5,2 s");
    println!("cổng M2 ③  : ≤ 1,5 s");
    if nhanh_nhat <= 1.5 {
        println!(
            "KẾT LUẬN   : ĐẠT ({:.1}× nhanh hơn bản PowerShell)",
            5.2 / nhanh_nhat
        );
    } else {
        println!("KẾT LUẬN   : KHÔNG ĐẠT — chạm tiêu chí dừng D-2, phải báo cáo chứ đừng đi tiếp");
        std::process::exit(1);
    }
}
