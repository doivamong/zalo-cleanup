//! **CỔNG MỐC M1** — đối chiếu song song bản Rust với bản PowerShell.
//!
//! Đây là phép thử quan trọng nhất của cả đợt port. Nó không kiểm bản Rust có
//! "hợp lý" không; nó kiểm bản Rust trả lời **giống hệt** bản PowerShell trên
//! hàng vạn đầu vào. Khác một ly là hỏng.
//!
//! Kỹ thuật này đã được chứng minh: khi tăng tốc `Test-Protected` 46 lần, hai
//! bản cũ và mới được chạy song song trên 57.144 đầu vào và cho 0 khác biệt.
//! Nhờ vậy mới dám thay một lớp an toàn. Nay dùng lại đúng cách đó, lần này để
//! bắc cầu giữa hai ngôn ngữ.
//!
//! Bản thần chú phía PowerShell **bóc thẳng hàm ra khỏi `ZaloCleanup.ps1`** bằng
//! AST, không chép lại logic — chép là hai bản trôi khỏi nhau, mà bắt trôi mới
//! là điểm của bộ đối chiếu.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use zalo_core::confirm::bo_dau_thanh;
use zalo_core::protect::{Luat, Muc, VungBaoVe};

const GOC_DU_LIEU: &str = r"C:\Users\ADMIN\AppData\Roaming\ZaloData";
const TEN_BAO_VE: [&str; 2] = ["Database", "Partitions"];

fn duong_dan_oracle() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/oracle-protect.ps1")
}

fn thu_muc_cong_cu() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    p.canonicalize()
        .map(|c| {
            c.to_string_lossy()
                .trim_start_matches(r"\\?\")
                .trim_end_matches('\\')
                .to_string()
        })
        .unwrap_or_else(|_| p.to_string_lossy().to_string())
}

/// Chạy bản thần chú PowerShell, đưa `dau_vao` qua stdin, nhận từng dòng trả lời.
fn goi_oracle(che_do: &str, dau_vao: &[String]) -> Vec<String> {
    let mut con = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            duong_dan_oracle().to_str().unwrap(),
            "-Mode",
            che_do,
            "-DataRoot",
            GOC_DU_LIEU,
            "-ToolDir",
            &thu_muc_cong_cu(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("không chạy được powershell.exe");

    // Ghi stdin ở MỘT LUỒNG RIÊNG, đọc stdout ở luồng chính.
    //
    // Ghi hết stdin rồi mới đọc stdout là công thức của deadlock: đầu ra của
    // 57.000 dòng vượt bộ đệm ống 64 KB, tiến trình con kẹt ở lệnh ghi nên thôi
    // đọc stdin, còn ta thì kẹt ở lệnh ghi stdin. Cả hai đứng im chờ nhau.
    //
    // Đã dính đúng lỗi này khi thêm đường dẫn Zalo thật vào: với vài trăm ca thì
    // chạy được, với 57.000 ca thì treo. Đây là loại lỗi chỉ lộ ra khi tăng quy mô.
    let mut si = con.stdin.take().unwrap();
    let vao: Vec<String> = dau_vao.to_vec();
    let luong_ghi = std::thread::spawn(move || {
        let mut d = std::io::BufWriter::new(&mut si);
        for x in &vao {
            let _ = writeln!(d, "{x}");
        }
        let _ = d.flush();
    });

    let ra = con.wait_with_output().expect("oracle không trả kết quả");
    luong_ghi.join().expect("luồng ghi stdin hỏng");
    assert!(
        ra.status.success(),
        "oracle hỏng: {}",
        String::from_utf8_lossy(&ra.stderr)
    );
    String::from_utf8_lossy(&ra.stdout)
        .lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect()
}

/// Nạp bộ luật do CHÍNH bản PowerShell dựng, để hai bên xuất phát từ cùng đầu vào.
fn nap_luat_tu_oracle() -> Vec<Luat> {
    let dong = goi_oracle("rules", &[]);
    assert!(!dong.is_empty(), "oracle không trả về luật nào");
    dong.iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut p = l.splitn(2, '\t');
            let m = p.next().unwrap();
            let d = p.next().unwrap_or("").to_string();
            Luat {
                duong_dan: d,
                muc: if m == "tatca" { Muc::TatCa } else { Muc::Goc },
            }
        })
        .collect()
}

/// Dựng tập đầu vào máy móc quanh từng luật, đúng kiểu bộ so sánh của phiên trước.
fn dung_dau_vao(luat: &[Luat]) -> Vec<String> {
    let mut ca: Vec<String> = Vec::new();

    for l in luat {
        let p = l.duong_dan.trim_end_matches('\\').to_string();
        ca.push(p.clone());
        ca.push(p.to_uppercase());
        ca.push(p.to_lowercase());
        ca.push(format!("{p}\\"));
        ca.push(format!("{p}\\con.txt"));
        ca.push(format!("{p}\\a\\b\\c\\sau.bin"));
        // Tên GẦN GIỐNG — phải KHÔNG bị chặn.
        ca.push(format!("{p}x\\ten_gan_giong.txt"));
        ca.push(format!("{p}_khac\\z.txt"));
        // Chữ có dấu trong đường dẫn: chỗ mà OrdinalIgnoreCase dễ lệch nhất.
        ca.push(format!("{p}\\Tài Liệu\\ảnh cũ.jxl"));
        ca.push(format!("{p}\\TÀI LIỆU\\ẢNH CŨ.JXL"));
    }

    for n in TEN_BAO_VE {
        let p = format!("{GOC_DU_LIEU}\\{n}");
        ca.push(p.clone());
        ca.push(format!("{p}\\"));
        ca.push(format!("{p}\\x.db"));
        ca.push(format!("{p}\\sau\\sau\\y.bin"));
        ca.push(format!("{p}X\\z.txt"));
        ca.push(p.to_uppercase());
        ca.push(p.to_lowercase());
    }

    for e in [
        "",
        "\\",
        "C:\\",
        "C:",
        "khong_co_gach_cheo",
        "\\\\may\\chiase\\tep.txt",
        r"C:\WINDOWS\system32\DRIVERS\etc\hosts",
        r"c:\windows\System32",
        r"C:\Users\ADMIN\AppData\Roaming\ZaloData\media\x\ZaloDownloads\video\1",
    ] {
        ca.push(e.to_string());
    }

    ca
}

/// Gom đường dẫn Zalo THẬT trên máy này, nếu có.
///
/// Cổng M1 đòi chạy lại đúng bộ so sánh 57.144 đầu vào của phiên trước, mà phần
/// lớn trong đó là 56.913 đường dẫn thật. Máy chủ CI không có dữ liệu Zalo nên
/// phần này chỉ chạy trên máy phát triển; khi vắng, phép thử **nói rõ là đã bỏ
/// qua** chứ không lặng lẽ báo xanh với vài trăm đầu vào.
///
/// Đặt biến môi trường `ZALO_DOI_CHIEU_GOC` để trỏ sang thư mục khác.
fn duong_dan_zalo_that() -> Vec<String> {
    let goc = std::env::var("ZALO_DOI_CHIEU_GOC")
        .unwrap_or_else(|_| format!("{GOC_DU_LIEU}\\media\\2068096368017928379\\ZaloDownloads"));
    let mut ra = Vec::new();
    let mut hang = vec![PathBuf::from(&goc)];
    while let Some(d) = hang.pop() {
        let doc = match std::fs::read_dir(&d) {
            Ok(x) => x,
            Err(_) => continue,
        };
        for e in doc.flatten() {
            match e.file_type() {
                Ok(t) if t.is_dir() => hang.push(e.path()),
                Ok(t) if t.is_file() => ra.push(e.path().to_string_lossy().to_string()),
                _ => {}
            }
        }
    }
    ra
}

#[test]
fn vung_bao_ve_khop_tung_ly_voi_ban_powershell() {
    let luat = nap_luat_tu_oracle();
    let v = VungBaoVe::dung(&luat, GOC_DU_LIEU, &TEN_BAO_VE);
    let mut ca = dung_dau_vao(&luat);

    let that = duong_dan_zalo_that();
    if that.is_empty() {
        eprintln!(
            "CHÚ Ý: không thấy dữ liệu Zalo thật trên máy này nên chỉ đối chiếu \
             {} ca dựng máy móc. Cổng M1 đầy đủ cần chạy thêm trên máy có dữ liệu.",
            ca.len()
        );
    } else {
        eprintln!("gom được {} đường dẫn Zalo thật", that.len());
        ca.extend(that);
    }

    let ps = goi_oracle("protect", &ca);
    assert_eq!(
        ps.len(),
        ca.len(),
        "oracle trả về {} dòng cho {} đầu vào",
        ps.len(),
        ca.len()
    );

    let mut lech: Vec<String> = Vec::new();
    let mut so_chan = 0usize;
    for (i, c) in ca.iter().enumerate() {
        let r = v.chan(c);
        let p = ps[i] == "1";
        if p {
            so_chan += 1;
        }
        if r != p {
            lech.push(format!("PS={p} Rust={r} :: {c:?}"));
        }
    }

    assert!(
        lech.is_empty(),
        "Lệch {} trên {} đầu vào (PowerShell chặn {}). Ví dụ:\n  {}",
        lech.len(),
        ca.len(),
        so_chan,
        lech.iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    assert!(
        so_chan > 0,
        "không đầu vào nào bị chặn — bộ đối chiếu vô nghĩa"
    );
    eprintln!(
        "đối chiếu vùng bảo vệ: {} đầu vào, {} bị chặn, 0 khác biệt",
        ca.len(),
        so_chan
    );
}

#[test]
fn chan_thu_muc_goc_khop_tung_ly_voi_ban_powershell() {
    let luat = nap_luat_tu_oracle();
    let v = VungBaoVe::dung(&luat, GOC_DU_LIEU, &TEN_BAO_VE);

    // Chiều ngược chỉ có nghĩa với thư mục, nên tập đầu vào là các thư mục CHA.
    let mut ca: Vec<String> = Vec::new();
    for l in &luat {
        let p = l.duong_dan.trim_end_matches('\\');
        ca.push(p.to_string());
        if let Some(i) = p.rfind('\\') {
            ca.push(p[..i].to_string());
            if let Some(j) = p[..i].rfind('\\') {
                ca.push(p[..j].to_string());
            }
        }
    }
    ca.push(GOC_DU_LIEU.to_string());
    ca.push(r"C:\Users".to_string());
    ca.push(r"C:\".to_string());
    ca.push(r"D:\khong_lien_quan".to_string());
    ca.sort();
    ca.dedup();

    let ps = goi_oracle("root", &ca);
    assert_eq!(ps.len(), ca.len());

    let lech: Vec<String> = ca
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let r = v.chan_thu_muc_goc(c);
            let p = ps[i] == "1";
            if r != p {
                Some(format!("PS={p} Rust={r} :: {c:?}"))
            } else {
                None
            }
        })
        .collect();

    assert!(
        lech.is_empty(),
        "Lệch {} trên {} thư mục gốc:\n  {}",
        lech.len(),
        ca.len(),
        lech.join("\n  ")
    );
    eprintln!("đối chiếu thư mục gốc: {} đầu vào, 0 khác biệt", ca.len());
}

#[test]
fn bo_dau_thanh_khop_tung_ly_voi_ban_powershell() {
    let mut ca: Vec<String> = vec![
        "XÓA",
        "XOÁ",
        "XOA",
        "xóa",
        "xoá",
        "Xóa",
        "XÓA ",
        "XÓAA",
        "CÓ",
        "TÔI CHẤP NHẬN MẤT",
        "TOI CHAP NHAN MAT",
        "GHI ĐÈ",
        "GHI DE",
        "GHI ĐE",
        "XÓA HẾT BẢN CHỤP",
        "XOÁ HẾT BẢN CHỤP",
        "XOA HET BAN CHUP",
        "ĐÃ XÓA",
        "đã xóa",
        "Đường dẫn",
        "ưƯơƠăĂâÂêÊôÔ",
        "ạảãáàặẳẵắằ",
        "C:\\Windows\\System32",
        "Tài Liệu",
        "abc",
        "123",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    // Mọi nguyên âm tiếng Việt có dấu, cả hoa lẫn thường.
    for c in "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ".chars()
    {
        ca.push(c.to_string());
        ca.push(c.to_uppercase().to_string());
    }

    let ps = goi_oracle("tones", &ca);
    assert_eq!(ps.len(), ca.len());

    let lech: Vec<String> = ca
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let r = bo_dau_thanh(c);
            if r != ps[i] {
                Some(format!("{c:?}: PS={:?} Rust={:?}", ps[i], r))
            } else {
                None
            }
        })
        .collect();

    assert!(
        lech.is_empty(),
        "Lệch {} trên {} chuỗi:\n  {}",
        lech.len(),
        ca.len(),
        lech.join("\n  ")
    );
    eprintln!("đối chiếu bỏ dấu thanh: {} chuỗi, 0 khác biệt", ca.len());
}

// ==================================================================== CỔNG M2

/// Gọi thần chú với một tham số duy nhất, dùng cho chế độ `walk`.
fn goi_oracle_mot(che_do: &str, tham_so: &str) -> Vec<String> {
    goi_oracle(che_do, &[tham_so.to_string()])
}

/// **Cổng M2 ①** — hai bản duyệt cùng một cây thật, ra cùng tập tệp và cùng số lỗi.
#[test]
fn duyet_cay_khop_tung_tep_voi_ban_powershell() {
    let goc = std::env::var("ZALO_DOI_CHIEU_GOC")
        .unwrap_or_else(|_| format!("{GOC_DU_LIEU}\\media\\2068096368017928379\\ZaloDownloads"));
    if !std::path::Path::new(&goc).is_dir() {
        eprintln!("CHÚ Ý: không có cây thật để đối chiếu duyệt — bỏ qua cổng M2 ①");
        return;
    }

    let dong = goi_oracle_mot("walk", &goc);
    assert!(!dong.is_empty(), "thần chú không trả về gì");
    let loi_ps: usize = dong[0].trim().parse().expect("dòng đầu phải là số lỗi");
    let ps: std::collections::BTreeSet<String> =
        dong[1..].iter().map(|s| s.to_uppercase()).collect();

    let r = zalo_core::walk::duyet(std::path::Path::new(&goc));
    let rust: std::collections::BTreeSet<String> = r
        .tep
        .iter()
        .map(|t| t.duong_dan.to_string_lossy().to_uppercase())
        .collect();

    let chi_ps: Vec<&String> = ps.difference(&rust).take(5).collect();
    let chi_rust: Vec<&String> = rust.difference(&ps).take(5).collect();

    assert!(
        chi_ps.is_empty() && chi_rust.is_empty(),
        "Tập tệp lệch. Chỉ PowerShell thấy {}: {:?}. Chỉ Rust thấy {}: {:?}",
        ps.difference(&rust).count(),
        chi_ps,
        rust.difference(&ps).count(),
        chi_rust
    );
    assert_eq!(loi_ps, r.loi, "số lỗi truy cập lệch");
    eprintln!(
        "đối chiếu duyệt cây: {} tệp, {} lỗi, 0 khác biệt",
        ps.len(),
        loi_ps
    );
}

/// **Cổng M2** — băm toàn tệp và chữ ký nhanh phải ra đúng chuỗi bản PowerShell sinh.
#[test]
fn bam_khop_tung_chuoi_voi_ban_powershell() {
    let goc = std::env::var("ZALO_DOI_CHIEU_GOC")
        .unwrap_or_else(|_| format!("{GOC_DU_LIEU}\\media\\2068096368017928379\\ZaloDownloads"));
    if !std::path::Path::new(&goc).is_dir() {
        eprintln!("CHÚ Ý: không có dữ liệu thật để đối chiếu băm — bỏ qua");
        return;
    }

    // Lấy mẫu trải đều cả tệp nhỏ lẫn tệp lớn, để chạm cả nhánh FULL: lẫn Q:.
    let mut tep = zalo_core::walk::duyet(std::path::Path::new(&goc)).tep;
    tep.sort_by_key(|t| t.co);
    let mut mau: Vec<String> = Vec::new();
    let n = tep.len();
    if n > 0 {
        for i in 0..60usize {
            let j = i * n / 60;
            if j < n {
                mau.push(tep[j].duong_dan.to_string_lossy().to_string());
            }
        }
        // Thêm hẳn 10 tệp lớn nhất để chắc chắn chạm nhánh Q:.
        for t in tep.iter().rev().take(10) {
            mau.push(t.duong_dan.to_string_lossy().to_string());
        }
    }
    mau.sort();
    mau.dedup();
    assert!(!mau.is_empty(), "không lấy được mẫu nào");

    let ps_full = goi_oracle("hash", &mau);
    let ps_quick = goi_oracle("quicksig", &mau);
    assert_eq!(ps_full.len(), mau.len());
    assert_eq!(ps_quick.len(), mau.len());

    let mut lech: Vec<String> = Vec::new();
    let mut so_full = 0usize;
    let mut so_q = 0usize;
    for (i, m) in mau.iter().enumerate() {
        let p = std::path::Path::new(m);
        match zalo_core::hash::sha256_toan_tep(p) {
            Ok(h) if h != ps_full[i] => {
                lech.push(format!("toàn tệp {m}: PS={} Rust={h}", ps_full[i]))
            }
            Err(e) if ps_full[i] != "LỖI" => lech.push(format!("toàn tệp {m}: Rust lỗi {e}")),
            _ => {}
        }
        match zalo_core::hash::chu_ky_nhanh(p) {
            Ok(q) => {
                if q.starts_with("FULL:") {
                    so_full += 1;
                } else {
                    so_q += 1;
                }
                if q != ps_quick[i] {
                    lech.push(format!("chữ ký nhanh {m}: PS={} Rust={q}", ps_quick[i]));
                }
            }
            Err(e) if ps_quick[i] != "LỖI" => {
                lech.push(format!("chữ ký nhanh {m}: Rust lỗi {e}"))
            }
            _ => {}
        }
    }

    assert!(
        lech.is_empty(),
        "Lệch {} trên {} tệp:\n  {}",
        lech.len(),
        mau.len(),
        lech.iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    assert!(so_q > 0, "mẫu không chạm nhánh Q: — phép thử chưa đủ nghĩa");
    eprintln!(
        "đối chiếu băm: {} tệp ({} nhánh FULL:, {} nhánh Q:), 0 khác biệt",
        mau.len(),
        so_full,
        so_q
    );
}

/// Phần mở rộng phải hiểu theo luật .NET, không theo luật Rust.
///
/// Dữ liệu Zalo thật có hàng nghìn tệp `.rescache`; `Path::extension()` của Rust
/// trả `None` cho chúng còn .NET trả `".rescache"`. Port ngây thơ là phân loại
/// sai hàng nghìn tệp.
#[test]
fn phan_mo_rong_khop_luat_dotnet() {
    let mut ca: Vec<String> = vec![
        ".rescache",
        ".gitignore",
        "video.jxl",
        "a.b.c",
        "a.",
        "a",
        "7594809871497",
        "",
        "x.JXL",
        "CHỮ.HOA",
        "tên có dấu.jpg",
        "..",
        "...",
        "a..b",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    // Thêm tên tệp thật lấy từ cây Zalo nếu có.
    let goc = std::env::var("ZALO_DOI_CHIEU_GOC")
        .unwrap_or_else(|_| format!("{GOC_DU_LIEU}\\media\\2068096368017928379\\ZaloDownloads"));
    if std::path::Path::new(&goc).is_dir() {
        let tep = zalo_core::walk::duyet(std::path::Path::new(&goc)).tep;
        for t in tep.iter().step_by(97).take(300) {
            if let Some(n) = t.duong_dan.file_name() {
                ca.push(n.to_string_lossy().to_string());
            }
        }
    }
    ca.sort();
    ca.dedup();

    let ps = goi_oracle("ext", &ca);
    assert_eq!(ps.len(), ca.len());

    let lech: Vec<String> = ca
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let r = format!("[{}]", zalo_core::scan::duoi_kieu_dotnet(c));
            if r != ps[i] {
                Some(format!("{c:?}: PS={} Rust={r}", ps[i]))
            } else {
                None
            }
        })
        .collect();

    assert!(
        lech.is_empty(),
        "Lệch {} trên {} tên tệp:\n  {}",
        lech.len(),
        ca.len(),
        lech.iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    eprintln!("đối chiếu phần mở rộng: {} tên tệp, 0 khác biệt", ca.len());
}
