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
