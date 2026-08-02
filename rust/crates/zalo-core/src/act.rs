//! Xóa · sao lưu · khôi phục · dọn thư mục rỗng.
//!
//! Nhận kết quả quét **đã được [`crate::gate`] duyệt**, không tự quyết.
//!
//! **Không bao giờ xóa đệ quy** (`R-10`): giữa lúc kết luận thư mục rỗng và lúc
//! ra lệnh xóa có một khe hở, tiến trình khác kịp ghi tệp vào đó thì xóa đệ quy
//! cuốn luôn tệp ấy mà không qua lớp kiểm vùng bảo vệ.
//!
//! **Chỉ đếm là đã xóa khi tệp thật sự biến mất** (`R-13`).
//!
//! Bản sao lưu phải tương thích ngược — xem [`crate::contract`]. Mốc **M4**.
//!
//! # Đọc tệp này với tay để xa bàn phím
//!
//! Đây là mô-đun duy nhất trong dự án thật sự **hủy dữ liệu của người dùng**, và
//! không qua Thùng rác. Chủ dự án đã dùng công cụ này xóa 149.309 tệp / 37 GB
//! ảnh và video thật. Mọi `if` ở đây đều là một cửa; sửa một cửa mà không kiểm
//! bằng đột biến thì coi như chưa sửa.
//!
//! Ba cửa dễ tưởng là thừa nhất, và cả ba đều không thừa:
//!
//! 1. **Hỏi vùng bảo vệ lần nữa ngay trước khi xóa**, dù lúc quét đã hỏi rồi.
//!    Kết quả quét được giữ tới hai giờ; bộ luật có thể đã đổi trong khoảng ấy.
//! 2. **Kiểm bản giữ lại còn sống** — xem [`crate::gate::ban_giu_lai_con_song`].
//! 3. **Đọc lại cỡ tệp lúc xóa** thay vì dùng cỡ ghi lúc quét. Con số ấy đi
//!    thẳng vào nhật ký và vào tổng dung lượng báo đã thu hồi.

use crate::protect::VungBaoVe;
use crate::thoigian::luc_nay;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Một tệp trong kết quả quét, kèm bản giữ lại nếu là chế độ khử trùng lặp.
#[derive(Clone, Debug)]
pub struct TepQuet {
    pub duong_dan: String,
    pub co: u64,
    /// Bản độc lập giống hệt đang được giữ. Rỗng nghĩa là không phải chế độ
    /// khử trùng lặp, tức không có ràng buộc nào.
    pub giu_lai: String,
}

impl TepQuet {
    pub fn moi(duong_dan: String, co: u64) -> Self {
        TepQuet {
            duong_dan,
            co,
            giu_lai: String::new(),
        }
    }
}

/// Thêm tiền tố đường dẫn dài khi cần.
///
/// Windows chỉ nhận đường dẫn quá 260 ký tự khi có tiền tố `\\?\`. Ngưỡng 240
/// lấy đúng của bản PowerShell — chừa chỗ cho phần đuôi mà lời gọi có thể nối
/// thêm. Đường dẫn đã có tiền tố thì để yên, nối hai lần là hỏng.
pub fn duong_dan_dai(p: &str) -> String {
    if p.len() < 240 || p.starts_with(r"\\?\") {
        return p.to_string();
    }
    if let Some(x) = p.strip_prefix(r"\\") {
        return format!(r"\\?\UNC\{x}");
    }
    format!(r"\\?\{p}")
}

// ==================================================================== xóa

/// Kết quả một lượt xóa. Bảy con số, và **không con số nào được gộp**.
///
/// Gộp "biến mất" vào "đã xóa" là báo cáo một việc mình không làm. Gộp "mất bản
/// gốc" vào "thất bại" là giấu mất lý do duy nhất người dùng cần biết để quét
/// lại. Mỗi ngã ra một con số riêng, và nhật ký ghi riêng từng dòng.
#[derive(Debug, Default, Clone)]
pub struct KetQuaXoa {
    pub da_xoa: usize,
    pub cat_cut: usize,
    pub that_bai: usize,
    pub bien_mat: usize,
    pub vung_bao_ve: usize,
    pub mat_ban_goc: usize,
    pub byte_thu_hoi: u64,
    pub hoan_tat: bool,
    /// Tối đa 30 dòng, đúng như bản PowerShell.
    pub loi: Vec<String>,
    pub tep_nhat_ky: PathBuf,
}

/// Cắt cụt một tệp đang bị khóa về 0 byte.
///
/// **Chỉ dành cho cache.** Với dữ liệu thật thì cắt cụt là hủy nội dung mà vẫn
/// để lại cái tên, tức người dùng tưởng tệp còn đó. Với cache thì tên còn mà
/// ruột rỗng là chuyện ứng dụng tự xử lý được, và dung lượng thu về thật.
pub fn cat_cut(duong_dan: &str) -> bool {
    match std::fs::OpenOptions::new()
        .write(true)
        .open(duong_dan_dai(duong_dan))
    {
        Ok(f) => f.set_len(0).is_ok(),
        Err(_) => false,
    }
}

fn bo_co_chi_doc(p: &Path) -> Option<u64> {
    let md = std::fs::metadata(p).ok()?;
    let co = md.len();
    let mut q = md.permissions();
    if q.readonly() {
        #[allow(clippy::permissions_set_readonly_false)]
        q.set_readonly(false);
        let _ = std::fs::set_permissions(p, q);
    }
    Some(co)
}

/// Xóa từng tệp trong kết quả quét đã được duyệt.
///
/// `co_the_cat_cut` chỉ được bật cho cache — xem [`cat_cut`].
///
/// Hàm này **không in ra màn hình**; nó ghi nhật ký ra tệp và trả về số liệu.
/// Nhật ký dùng chung định dạng với bản PowerShell vì hai bản ghi vào cùng một
/// thư mục `logs\`, và lịch sử dọn dẹp phải đọc được như một dòng duy nhất.
#[allow(clippy::too_many_arguments)]
pub fn xoa(
    danh_sach: &[TepQuet],
    vbv: &VungBaoVe,
    loai_quet: &str,
    dau_quet: &str,
    dich_sao_luu: Option<&str>,
    co_the_cat_cut: bool,
    thu_muc_nhat_ky: &Path,
) -> std::io::Result<KetQuaXoa> {
    let _ = std::fs::create_dir_all(thu_muc_nhat_ky);
    let nay = luc_nay();
    let tep_nhat_ky = thu_muc_nhat_ky.join(format!("daxoa_{}.log", nay.dau_thoi_gian()));
    let mut w = std::io::BufWriter::new(std::fs::File::create(&tep_nhat_ky)?);

    writeln!(w, "# Nhật ký xóa")?;
    writeln!(w, "# Chế độ  : {loai_quet}")?;
    match dich_sao_luu {
        Some(d) => writeln!(w, "# Sao lưu : có → {d}")?,
        None => writeln!(w, "# Sao lưu : không")?,
    }
    writeln!(w, "# Quét lúc: {dau_quet}")?;
    writeln!(w, "# Bắt đầu : {}", nay.dinh_dang())?;
    writeln!(w, "# Cột: TRẠNGTHÁI\tBYTES\tĐƯỜNGDẪN")?;

    let mut r = KetQuaXoa {
        tep_nhat_ky: tep_nhat_ky.clone(),
        ..Default::default()
    };

    for f in danh_sach {
        // Cửa 1. Hỏi lại dù lúc quét đã hỏi — xem chú thích đầu mô-đun.
        if vbv.chan(&f.duong_dan) {
            r.vung_bao_ve += 1;
            writeln!(w, "VÙNGBẢOVỆ\t0\t{}", f.duong_dan)?;
            continue;
        }
        let lp = duong_dan_dai(&f.duong_dan);
        let p = Path::new(&lp);
        if !p.is_file() {
            r.bien_mat += 1;
            writeln!(w, "BIẾNMẤT\t0\t{}", f.duong_dan)?;
            continue;
        }

        // Cửa 2. Bản giữ lại của một cặp trùng lặp phải còn sống và còn đúng cỡ.
        if !crate::gate::ban_giu_lai_con_song(&f.giu_lai, f.co) {
            r.mat_ban_goc += 1;
            writeln!(w, "MẤTBẢNGỐC\t0\t{}", f.duong_dan)?;
            continue;
        }

        // Cửa 3. Cỡ THẬT lúc này, không phải cỡ ghi lúc quét.
        let co_that = bo_co_chi_doc(p).unwrap_or(0);

        match std::fs::remove_file(p) {
            Ok(()) => {
                // R-13: chỉ đếm là đã xóa khi tệp THẬT SỰ biến mất. Vài hệ tệp
                // mạng trả về thành công rồi vẫn để tệp đó.
                if p.exists() {
                    r.that_bai += 1;
                    writeln!(w, "THẤTBẠI\t{co_that}\t{}", f.duong_dan)?;
                    if r.loi.len() < 30 {
                        r.loi
                            .push(format!("{} => vẫn còn sau khi xóa", f.duong_dan));
                    }
                } else {
                    r.da_xoa += 1;
                    r.byte_thu_hoi += co_that;
                    writeln!(w, "ĐÃXÓA\t{co_that}\t{}", f.duong_dan)?;
                }
            }
            Err(e) => {
                if co_the_cat_cut && co_that > 0 && cat_cut(&f.duong_dan) {
                    r.cat_cut += 1;
                    r.byte_thu_hoi += co_that;
                    writeln!(w, "CẮTCỤT\t{co_that}\t{}", f.duong_dan)?;
                } else {
                    r.that_bai += 1;
                    writeln!(w, "THẤTBẠI\t{co_that}\t{}", f.duong_dan)?;
                    if r.loi.len() < 30 {
                        r.loi.push(format!("{} => {e}", f.duong_dan));
                    }
                }
            }
        }
    }
    r.hoan_tat = true;

    // `True`/`False` viết hoa chữ đầu: bản PowerShell in giá trị boolean ra như
    // vậy, và bộ test so thẳng chuỗi `hoàn tất=True`.
    writeln!(
        w,
        "# Tổng kết: đã xóa={} cắt cụt={} thất bại={} biến mất={} vùng bảo vệ={} mất bản gốc={} bytes={} hoàn tất={}",
        r.da_xoa,
        r.cat_cut,
        r.that_bai,
        r.bien_mat,
        r.vung_bao_ve,
        r.mat_ban_goc,
        r.byte_thu_hoi,
        if r.hoan_tat { "True" } else { "False" }
    )?;
    w.flush()?;
    Ok(r)
}

// ==================================================================== thư mục rỗng

/// Dọn thư mục rỗng, giới hạn trong đúng các gốc được chỉ định.
///
/// # Bốn điều tuyệt đối không làm ở đây
///
/// 1. **Không nhận gốc ổ đĩa.** Nhận `C:\` là duyệt cả ổ hệ thống.
/// 2. **Không xóa đệ quy.** Giữa lúc kết luận "thư mục này rỗng" và lúc ra lệnh
///    xóa có một khe hở; tiến trình khác kịp ghi tệp vào đó thì xóa đệ quy cuốn
///    luôn tệp ấy mà không qua lớp kiểm vùng bảo vệ. `remove_dir` — không phải
///    `remove_dir_all` — ném lỗi khi thư mục hết rỗng, đúng thứ ta muốn.
/// 3. **Không đụng reparse point.** Một junction bị chặn quyền đọc trả về danh
///    sách con rỗng, trông y hệt thư mục rỗng; xóa lên nó có thể lan sang thư
///    mục đích ở đầu bên kia.
/// 4. **Không đụng vùng bảo vệ**, kể cả khi nó đang rỗng.
///
/// `giu_cap_mot` giữ lại các thư mục cấp một dưới gốc: chúng là cấu trúc Zalo tự
/// dựng, xóa đi thì lần sau Zalo phải tạo lại, và người dùng thấy thư mục quen
/// thuộc biến mất.
pub fn don_thu_muc_rong(cac_goc: &[String], giu_cap_mot: bool, vbv: &VungBaoVe) -> usize {
    let mut da_xoa = 0usize;
    for r in cac_goc {
        if r.trim().is_empty() {
            continue;
        }
        // Gốc ổ đĩa: `C:` hoặc `C:\`.
        let t = r.trim_end_matches('\\');
        if t.len() == 2 && t.as_bytes()[1] == b':' {
            continue;
        }
        let goc = Path::new(r);
        if !goc.is_dir() || vbv.chan_thu_muc_goc(r) || la_reparse(goc) {
            continue;
        }

        let mut giu: Vec<PathBuf> = vec![goc.to_path_buf()];
        if giu_cap_mot {
            if let Ok(doc) = std::fs::read_dir(goc) {
                for e in doc.flatten() {
                    if e.file_type().map(|x| x.is_dir()).unwrap_or(false) {
                        giu.push(e.path());
                    }
                }
            }
        }

        // Nhiều lượt vì xóa thư mục lá xong mới lộ ra thư mục cha đã rỗng. Trần
        // 20 lượt để một cây dị dạng không làm công cụ quay mãi.
        for _ in 0..20 {
            let rong = tim_thu_muc_rong(goc, &giu, vbv);
            if rong.is_empty() {
                break;
            }
            let mut xong = 0usize;
            for d in &rong {
                if xoa_thu_muc_neu_rong(d) {
                    da_xoa += 1;
                    xong += 1;
                }
            }
            // Lượt này không xóa được gì thì lượt sau cũng vậy.
            if xong == 0 {
                break;
            }
        }
    }
    da_xoa
}

/// Xóa một thư mục **chỉ khi nó rỗng**. Trả `true` nếu đã xóa.
///
/// # Vì sao là hàm riêng chứ không phải một dòng trong vòng lặp
///
/// Đây là chỗ nguyên tắc "không bao giờ xóa đệ quy" thật sự nằm. Giữa lúc
/// [`tim_thu_muc_rong`] kết luận một thư mục rỗng và lúc ra lệnh xóa có một khe
/// hở; tiến trình khác kịp ghi tệp vào đó thì `remove_dir_all` cuốn luôn tệp ấy
/// mà không qua lớp kiểm vùng bảo vệ, còn `remove_dir` thì hỏng — đúng thứ ta
/// muốn.
///
/// Tách ra vì đột biến đã chứng minh nó cần: đổi `remove_dir` thành
/// `remove_dir_all` ngay trong vòng lặp mà **không phép thử nào đỏ**, bởi mọi
/// thư mục đưa tới đó đều đã rỗng sẵn nên hai hàm cho cùng kết quả. Khe hở chỉ
/// hiện ra khi hỏi thẳng: "đưa cho nó một thư mục CÓ tệp thì sao?"
pub fn xoa_thu_muc_neu_rong(p: &Path) -> bool {
    std::fs::remove_dir(Path::new(&duong_dan_dai(&p.to_string_lossy()))).is_ok()
}

fn la_reparse(p: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        std::fs::symlink_metadata(p)
            .map(|m| m.file_attributes() & 0x400 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        std::fs::symlink_metadata(p)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }
}

fn tim_thu_muc_rong(goc: &Path, giu: &[PathBuf], vbv: &VungBaoVe) -> Vec<PathBuf> {
    let mut ra = Vec::new();
    let mut hang = vec![goc.to_path_buf()];
    while let Some(d) = hang.pop() {
        let doc = match std::fs::read_dir(&d) {
            Ok(x) => x,
            Err(_) => continue,
        };
        let mut con: Vec<PathBuf> = Vec::new();
        let mut co_tep = false;
        for e in doc.flatten() {
            match e.file_type() {
                Ok(t) if t.is_dir() => con.push(e.path()),
                Ok(_) => co_tep = true,
                Err(_) => co_tep = true, // không biết thì coi như có, tức KHÔNG xóa
            }
        }
        for c in con {
            if la_reparse(&c) {
                continue;
            }
            hang.push(c.clone());
            if giu.contains(&c) || vbv.chan(&c.to_string_lossy()) {
                continue;
            }
            let rong = std::fs::read_dir(&c)
                .map(|mut it| it.next().is_none())
                .unwrap_or(false);
            if rong {
                ra.push(c);
            }
        }
        let _ = co_tep;
    }
    ra
}

// ==================================================================== sao lưu

/// Kết quả một lượt sao lưu.
#[derive(Debug, Default, Clone)]
pub struct KetQuaSaoLuuChiTiet {
    pub thu_muc: PathBuf,
    pub tong: usize,
    pub da_chep: usize,
    pub chep_hong: usize,
    pub xac_minh_hong: usize,
    pub da_xac_minh: usize,
    pub het_cho: bool,
    pub xac_minh_toan_bo: bool,
    pub nhat_ky_loi: Vec<String>,
}

/// Bộ sinh số giả ngẫu nhiên nhỏ, chỉ để **lấy mẫu** khi xác minh.
///
/// Không dùng cho việc gì cần tính ngẫu nhiên thật. Tự viết thay vì thêm crate
/// `rand`: mục đích duy nhất ở đây là để mẫu 50 tệp không phải lúc nào cũng rơi
/// vào cùng một chỗ trong danh sách — nếu lấy 50 tệp đầu thì một lỗi chép chỉ
/// xảy ra ở cuối lượt sẽ không bao giờ bị bắt.
struct Xorshift(u64);

impl Xorshift {
    fn moi(hat: u64) -> Self {
        Xorshift(if hat == 0 { 0x9E3779B97F4A7C15 } else { hat })
    }
    fn tiep(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// Chép kết quả quét sang thư mục đích rồi xác minh.
///
/// # Ổ đích hết chỗ thì DỪNG NGAY
///
/// Không thử nốt hàng vạn tệp còn lại: mỗi tệp sau đó cũng hỏng y hệt, chỉ tổ
/// mất thời gian và làm nhật ký ngập một loại lỗi duy nhất, che mất những lỗi
/// thật sự khác nhau.
///
/// Nhưng dừng bằng cách thoát vòng lặp nghĩa là `chep_hong` **vẫn bằng 0** dù
/// bản sao lưu dở dang. Đó chính là lý do [`crate::gate::sao_luu_sach`] phải xét
/// cả `het_cho` lẫn phép so `da_chep` với `tong`, chứ không chỉ xét số lỗi.
pub fn sao_luu(
    danh_sach: &[TepQuet],
    goc: &str,
    thu_muc_dich: &Path,
    xac_minh_toan_bo: bool,
) -> std::io::Result<KetQuaSaoLuuChiTiet> {
    std::fs::create_dir_all(thu_muc_dich)?;
    let mut r = KetQuaSaoLuuChiTiet {
        thu_muc: thu_muc_dich.to_path_buf(),
        tong: danh_sach.len(),
        xac_minh_toan_bo,
        ..Default::default()
    };
    let mut da_chep: Vec<(String, PathBuf, u64)> = Vec::new();

    for f in danh_sach {
        let rel = crate::scan::duong_dan_tuong_doi(&f.duong_dan, goc);
        // `duong_dan_tuong_doi` trả về NGUYÊN đường dẫn tuyệt đối khi tệp không
        // nằm dưới gốc. Nối một đường dẫn tuyệt đối vào thư mục đích thì phần
        // gốc bị vứt đi, và bản sao lưu ghi ra NGOÀI thư mục sao lưu — tệ nhất
        // là đè lên chính tệp nguồn. Chặn thẳng ở đây.
        if Path::new(rel).is_absolute() || rel.len() >= 2 && rel.as_bytes()[1] == b':' {
            r.chep_hong += 1;
            r.nhat_ky_loi
                .push(format!("{} => tệp không nằm dưới gốc quét", f.duong_dan));
            continue;
        }
        let dich = thu_muc_dich.join(rel);
        if let Some(d) = dich.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        match std::fs::copy(&f.duong_dan, &dich) {
            Ok(_) => {
                r.da_chep += 1;
                da_chep.push((f.duong_dan.clone(), dich, f.co));
            }
            Err(e) => {
                if la_het_cho(&e) {
                    r.het_cho = true;
                    r.nhat_ky_loi
                        .push(format!("{} => ổ đích hết chỗ, dừng tại đây", f.duong_dan));
                    break;
                }
                r.chep_hong += 1;
                // Không chặn trần số dòng nhật ký lỗi: sao lưu hỏng là chặn luôn
                // bước xóa, nên danh sách tệp hỏng chính là thứ người dùng cần
                // đọc để quyết định làm gì tiếp. Cắt bớt là giấu đúng thứ ấy.
                r.nhat_ky_loi.push(format!("{} => {e}", f.duong_dan));
            }
        }
    }

    // Xác minh kích thước cho TOÀN BỘ tệp đã chép.
    for (_, dst, co) in &da_chep {
        match std::fs::metadata(dst) {
            Err(_) => {
                r.xac_minh_hong += 1;
                r.nhat_ky_loi
                    .push(format!("{} => thiếu ở đích", dst.display()));
            }
            Ok(md) if md.len() != *co => {
                r.xac_minh_hong += 1;
                r.nhat_ky_loi
                    .push(format!("{} => lệch kích thước", dst.display()));
            }
            Ok(_) => {}
        }
    }

    // SHA-256 cho toàn bộ, hoặc cho mẫu 50 tệp.
    let mut mau: Vec<usize> = (0..da_chep.len()).collect();
    if !xac_minh_toan_bo && mau.len() > 50 {
        let hat = luc_nay()
            .dau_thoi_gian()
            .bytes()
            .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
        let mut rng = Xorshift::moi(hat ^ da_chep.len() as u64);
        // Xáo Fisher–Yates rồi lấy 50 đầu.
        for i in (1..mau.len()).rev() {
            let j = (rng.tiep() % (i as u64 + 1)) as usize;
            mau.swap(i, j);
        }
        mau.truncate(50);
    }
    for i in mau {
        let (src, dst, _) = &da_chep[i];
        match (
            crate::hash::sha256_toan_tep(Path::new(src)),
            crate::hash::sha256_toan_tep(dst),
        ) {
            (Ok(a), Ok(b)) => {
                if a != b {
                    r.xac_minh_hong += 1;
                    r.nhat_ky_loi
                        .push(format!("{} => hash không khớp", dst.display()));
                }
                r.da_xac_minh += 1;
            }
            _ => {
                r.xac_minh_hong += 1;
                r.nhat_ky_loi
                    .push(format!("{} => lỗi đọc khi xác minh", dst.display()));
            }
        }
    }

    let byte: u64 = da_chep.iter().map(|(_, _, c)| *c).sum();
    ghi_ban_ke(thu_muc_dich, goc, &r, byte)?;
    Ok(r)
}

/// `ERROR_DISK_FULL` (112) và `ERROR_HANDLE_DISK_FULL` (39).
fn la_het_cho(e: &std::io::Error) -> bool {
    match e.raw_os_error() {
        Some(112) | Some(39) => true,
        _ => {
            let s = e.to_string().to_lowercase();
            s.contains("not enough space") || s.contains("disk is full")
        }
    }
}

fn ghi_ban_ke(
    thu_muc: &Path,
    goc: &str,
    r: &KetQuaSaoLuuChiTiet,
    byte: u64,
) -> std::io::Result<()> {
    // Bản kê là HỢP ĐỒNG giữa hai bản: bản sao lưu do bản này tạo phải khôi phục
    // được bằng bản kia. Tên trường, kiểu dữ liệu và dạng ngày đều phải khớp.
    let v = serde_json::json!({
        "Tool": crate::contract::BACKUP_MANIFEST_TOOL,
        "Version": crate::contract::BACKUP_MANIFEST_VERSION,
        "Created": luc_nay().dang_ban_ke(),
        "SourceRoot": goc,
        "ScanKind": "",
        "Count": r.da_chep,
        "Bytes": byte,
        "FullVerify": r.xac_minh_toan_bo,
        "Verified": r.da_xac_minh,
        "VerifyFail": r.xac_minh_hong,
        "CopyFail": r.chep_hong,
    });
    std::fs::write(
        thu_muc.join(crate::contract::BACKUP_MANIFEST_FILE),
        serde_json::to_string_pretty(&v).unwrap_or_default(),
    )
}

/// Ghi lại `ScanKind` vào bản kê sau khi đã biết loại quét.
///
/// Tách riêng vì [`sao_luu`] nằm ở lõi và không biết gì về tên các chế độ quét —
/// đó là chuyện của lớp trên.
pub fn ghi_loai_quet(thu_muc: &Path, loai_quet: &str) -> std::io::Result<()> {
    let tep = thu_muc.join(crate::contract::BACKUP_MANIFEST_FILE);
    let s = std::fs::read_to_string(&tep)?;
    let mut v: serde_json::Value = serde_json::from_str(s.trim_start_matches('\u{feff}'))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    v["ScanKind"] = serde_json::Value::String(loai_quet.to_string());
    std::fs::write(&tep, serde_json::to_string_pretty(&v).unwrap_or_default())
}

// ==================================================================== khôi phục

#[derive(Debug, Default, Clone)]
pub struct KetQuaKhoiPhuc {
    pub da_khoi_phuc: usize,
    pub bo_qua: usize,
    pub that_bai: usize,
    pub het_cho: bool,
    pub tep_nhat_ky: PathBuf,
}

/// Chép ngược từ thư mục sao lưu về đích.
///
/// `ghi_de` sai thì tệp đã tồn tại được **giữ nguyên** — mặc định an toàn nhất.
/// Khôi phục mà đè mất bản đang dùng là hỏng theo chiều ngược lại với chiều
/// người dùng đang lo.
pub fn khoi_phuc(
    thu_muc_sao_luu: &Path,
    dich: &str,
    ghi_de: bool,
    thu_muc_nhat_ky: &Path,
) -> std::io::Result<KetQuaKhoiPhuc> {
    let _ = std::fs::create_dir_all(thu_muc_nhat_ky);
    let nay = luc_nay();
    let tep_nhat_ky = thu_muc_nhat_ky.join(format!("khoiphuc_{}.log", nay.dau_thoi_gian()));
    let mut w = std::io::BufWriter::new(std::fs::File::create(&tep_nhat_ky)?);
    writeln!(w, "# Nhật ký khôi phục")?;
    writeln!(w, "# Từ      : {}", thu_muc_sao_luu.display())?;
    writeln!(w, "# Về      : {dich}")?;
    writeln!(w, "# Bắt đầu : {}", nay.dinh_dang())?;

    let mut r = KetQuaKhoiPhuc {
        tep_nhat_ky: tep_nhat_ky.clone(),
        ..Default::default()
    };
    let goc = thu_muc_sao_luu.to_string_lossy().to_string();
    let tep: Vec<crate::walk::Tep> = crate::walk::duyet(thu_muc_sao_luu)
        .tep
        .into_iter()
        .filter(|t| {
            t.duong_dan
                .file_name()
                .map(|n| n != crate::contract::BACKUP_MANIFEST_FILE)
                .unwrap_or(true)
        })
        .collect();

    for f in &tep {
        let s = f.duong_dan.to_string_lossy().to_string();
        let rel = crate::scan::duong_dan_tuong_doi(&s, &goc);
        let dst = Path::new(dich).join(rel);
        if dst.exists() && !ghi_de {
            r.bo_qua += 1;
            writeln!(w, "BỎQUA\t{rel}")?;
            continue;
        }
        if let Some(d) = dst.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        match std::fs::copy(&f.duong_dan, &dst) {
            Ok(_) => {
                r.da_khoi_phuc += 1;
                writeln!(w, "KHÔIPHỤC\t{rel}")?;
            }
            Err(e) => {
                if la_het_cho(&e) {
                    r.het_cho = true;
                    writeln!(w, "HẾTCHỖ\t{s}")?;
                    break;
                }
                r.that_bai += 1;
                writeln!(w, "THẤTBẠI\t{s}\t{e}")?;
            }
        }
    }

    if r.het_cho {
        writeln!(w, "# Dừng sớm: ổ đích hết chỗ")?;
    }
    writeln!(
        w,
        "# Tổng kết: khôi phục={} bỏ qua={} thất bại={} hết chỗ={}",
        r.da_khoi_phuc,
        r.bo_qua,
        r.that_bai,
        if r.het_cho { "True" } else { "False" }
    )?;
    w.flush()?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protect::{Luat, Muc};

    struct Hop(PathBuf);
    impl Hop {
        fn moi(ten: &str) -> Self {
            let p = std::env::temp_dir().join(format!("zact_{}_{ten}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Hop(std::fs::canonicalize(&p).unwrap_or(p))
        }
        fn tep(&self, rel: &str, n: usize) -> String {
            let p = self.0.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, vec![b'z'; n]).unwrap();
            p.to_string_lossy().to_string()
        }
    }
    impl Drop for Hop {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn vbv_rong() -> VungBaoVe {
        VungBaoVe::dung(&[], "", &[])
    }

    #[test]
    fn xoa_dung_so_tep_va_ghi_du_nhat_ky() {
        let h = Hop::moi("xoa");
        let ds: Vec<TepQuet> = ["video/a", "video/b", "video/c"]
            .iter()
            .map(|r| TepQuet::moi(h.tep(r, 100), 100))
            .collect();
        let r = xoa(
            &ds,
            &vbv_rong(),
            "DỮ LIỆU ZALO",
            "x",
            None,
            false,
            &h.0.join("logs"),
        )
        .unwrap();
        assert_eq!(r.da_xoa, 3);
        assert_eq!(r.that_bai, 0);
        assert_eq!(r.byte_thu_hoi, 300);
        assert!(r.hoan_tat);

        let nk = std::fs::read_to_string(&r.tep_nhat_ky).unwrap();
        assert_eq!(nk.matches("ĐÃXÓA\t").count(), 3);
        assert!(nk.contains("# Sao lưu : không"));
        assert!(nk.contains("hoàn tất=True"));
        assert!(nk.contains("# Cột: TRẠNGTHÁI\tBYTES\tĐƯỜNGDẪN"));
    }

    /// Vùng bảo vệ phải chặn **ngay trong vòng xóa**, dù lúc quét đã kiểm.
    /// Kết quả quét được giữ tới hai giờ; bộ luật có thể đã đổi trong khoảng ấy.
    #[test]
    fn vung_bao_ve_chan_ngay_trong_vong_xoa() {
        let h = Hop::moi("chan");
        let cam = h.0.join("cam").to_string_lossy().to_string();
        let vbv = VungBaoVe::dung(
            &[Luat {
                duong_dan: cam.clone(),
                muc: Muc::TatCa,
            }],
            "",
            &[],
        );
        let ds = vec![
            TepQuet::moi(h.tep("cam/quan_trong", 50), 50),
            TepQuet::moi(h.tep("video/thuong", 50), 50),
        ];
        let r = xoa(
            &ds,
            &vbv,
            "DỮ LIỆU ZALO",
            "x",
            None,
            false,
            &h.0.join("logs"),
        )
        .unwrap();
        assert_eq!(r.vung_bao_ve, 1, "tệp trong vùng bảo vệ KHÔNG bị chặn");
        assert_eq!(r.da_xoa, 1);
        assert!(
            h.0.join("cam/quan_trong").exists(),
            "tệp trong vùng bảo vệ đã bị xóa"
        );
        let nk = std::fs::read_to_string(&r.tep_nhat_ky).unwrap();
        assert!(nk.contains("VÙNGBẢOVỆ\t0\t"));
    }

    /// Bản giữ lại biến mất giữa lúc quét và lúc xóa thì tệp kia không còn là
    /// bản thừa nữa mà là bản DUY NHẤT. Phải giữ lại.
    #[test]
    fn mat_ban_giu_lai_thi_khong_xoa() {
        let h = Hop::moi("keeper");
        let goc = h.tep("video/goc", 100);
        let thua = h.tep("resource/c1/thua", 100);
        std::fs::remove_file(&goc).unwrap();

        let ds = vec![TepQuet {
            duong_dan: thua.clone(),
            co: 100,
            giu_lai: goc,
        }];
        let r = xoa(
            &ds,
            &vbv_rong(),
            "BẢN TRÙNG LẶP",
            "x",
            None,
            false,
            &h.0.join("logs"),
        )
        .unwrap();
        assert_eq!(r.mat_ban_goc, 1);
        assert_eq!(r.da_xoa, 0);
        assert!(Path::new(&thua).exists(), "đã xóa bản duy nhất còn lại");
        let nk = std::fs::read_to_string(&r.tep_nhat_ky).unwrap();
        assert!(nk.contains("MẤTBẢNGỐC\t0\t"));
    }

    /// Bản giữ lại đổi cỡ cũng tính là mất — nó không còn giống bản kia nữa.
    #[test]
    fn ban_giu_lai_doi_co_cung_tinh_la_mat() {
        let h = Hop::moi("keepersize");
        let goc = h.tep("video/goc", 100);
        let thua = h.tep("resource/c1/thua", 100);
        std::fs::write(&goc, vec![b'z'; 55]).unwrap();
        let ds = vec![TepQuet {
            duong_dan: thua.clone(),
            co: 100,
            giu_lai: goc,
        }];
        let r = xoa(
            &ds,
            &vbv_rong(),
            "BẢN TRÙNG LẶP",
            "x",
            None,
            false,
            &h.0.join("logs"),
        )
        .unwrap();
        assert_eq!(r.mat_ban_goc, 1);
        assert!(Path::new(&thua).exists());
    }

    #[test]
    fn tep_bien_mat_truoc_khi_xoa_khong_duoc_dem_la_da_xoa() {
        let h = Hop::moi("bienmat");
        let p = h.tep("video/a", 10);
        std::fs::remove_file(&p).unwrap();
        let ds = vec![TepQuet::moi(p, 10)];
        let r = xoa(
            &ds,
            &vbv_rong(),
            "DỮ LIỆU ZALO",
            "x",
            None,
            false,
            &h.0.join("logs"),
        )
        .unwrap();
        assert_eq!(r.bien_mat, 1);
        assert_eq!(r.da_xoa, 0, "biến mất KHÔNG được gộp vào đã xóa");
        assert_eq!(r.byte_thu_hoi, 0);
    }

    /// Cỡ ghi vào nhật ký phải là cỡ THẬT lúc xóa, không phải cỡ ghi lúc quét.
    #[test]
    fn dung_co_that_luc_xoa_chu_khong_dung_co_luc_quet() {
        let h = Hop::moi("cothat");
        let p = h.tep("video/a", 100);
        std::fs::write(&p, vec![b'z'; 40]).unwrap();
        // Kết quả quét vẫn nhớ 100 byte.
        let ds = vec![TepQuet::moi(p, 100)];
        let r = xoa(
            &ds,
            &vbv_rong(),
            "DỮ LIỆU ZALO",
            "x",
            None,
            false,
            &h.0.join("logs"),
        )
        .unwrap();
        assert_eq!(r.da_xoa, 1);
        assert_eq!(r.byte_thu_hoi, 40, "báo thu hồi nhiều hơn thật");
    }

    #[test]
    fn sao_luu_roi_khoi_phuc_ra_dung_noi_dung() {
        let h = Hop::moi("saoluu");
        let goc = h.0.join("nguon");
        std::fs::create_dir_all(&goc).unwrap();
        let mut ds = Vec::new();
        for (r, n) in [("video/a", 300usize), ("picture/b.jxl", 120), ("c", 7)] {
            let p = goc.join(r);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, vec![b'q'; n]).unwrap();
            ds.push(TepQuet::moi(p.to_string_lossy().to_string(), n as u64));
        }
        let bk = h.0.join("bk");
        let r = sao_luu(&ds, &goc.to_string_lossy(), &bk, true).unwrap();
        assert_eq!(r.da_chep, 3);
        assert_eq!(r.chep_hong, 0);
        assert_eq!(r.xac_minh_hong, 0);
        assert_eq!(r.da_xac_minh, 3, "xác minh toàn bộ phải băm cả ba tệp");
        assert!(bk.join(crate::contract::BACKUP_MANIFEST_FILE).is_file());

        let ve = h.0.join("ve");
        let k = khoi_phuc(&bk, &ve.to_string_lossy(), false, &h.0.join("logs")).unwrap();
        assert_eq!(k.da_khoi_phuc, 3);
        assert_eq!(k.that_bai, 0);
        for (r, n) in [("video/a", 300usize), ("picture/b.jxl", 120), ("c", 7)] {
            let x = ve.join(r);
            assert!(x.is_file(), "thiếu {r} sau khi khôi phục");
            assert_eq!(std::fs::read(&x).unwrap().len(), n);
        }
        // Bản kê KHÔNG được khôi phục ra ngoài như một tệp dữ liệu.
        assert!(!ve.join(crate::contract::BACKUP_MANIFEST_FILE).exists());
    }

    #[test]
    fn khoi_phuc_khong_ghi_de_thi_giu_nguyen_tep_dang_co() {
        let h = Hop::moi("ghide");
        let goc = h.0.join("nguon");
        std::fs::create_dir_all(&goc).unwrap();
        let p = goc.join("a");
        std::fs::write(&p, b"BAN CU").unwrap();
        let ds = vec![TepQuet::moi(p.to_string_lossy().to_string(), 6)];
        let bk = h.0.join("bk");
        sao_luu(&ds, &goc.to_string_lossy(), &bk, true).unwrap();

        std::fs::write(&p, b"BAN MOI DANG DUNG").unwrap();
        let k = khoi_phuc(&bk, &goc.to_string_lossy(), false, &h.0.join("logs")).unwrap();
        assert_eq!(k.bo_qua, 1);
        assert_eq!(k.da_khoi_phuc, 0);
        assert_eq!(std::fs::read(&p).unwrap(), b"BAN MOI DANG DUNG");

        let k = khoi_phuc(&bk, &goc.to_string_lossy(), true, &h.0.join("logs")).unwrap();
        assert_eq!(k.da_khoi_phuc, 1);
        assert_eq!(std::fs::read(&p).unwrap(), b"BAN CU");
    }

    /// Tệp không nằm dưới gốc quét **không được** ghi ra ngoài thư mục sao lưu.
    #[test]
    fn tep_ngoai_goc_quet_bi_chan_chu_khong_ghi_ra_ngoai() {
        let h = Hop::moi("ngoaigoc");
        let goc = h.0.join("nguon");
        std::fs::create_dir_all(&goc).unwrap();
        let la = h.tep("o_noi_khac/x", 10);
        let ds = vec![TepQuet::moi(la.clone(), 10)];
        let bk = h.0.join("bk");
        let r = sao_luu(&ds, &goc.to_string_lossy(), &bk, true).unwrap();
        assert_eq!(r.da_chep, 0);
        assert_eq!(r.chep_hong, 1);
        assert!(r.nhat_ky_loi[0].contains("không nằm dưới gốc quét"));
        assert!(Path::new(&la).is_file(), "tệp nguồn bị đụng tới");
    }

    /// Phép xóa thư mục phải **từ chối** thư mục còn tệp bên trong.
    ///
    /// Đây là nguyên tắc "không bao giờ xóa đệ quy", hỏi thẳng vào chỗ nó sống.
    /// Vòng lặp gọi hàm này chỉ đưa tới những thư mục đã rỗng sẵn, nên đổi sang
    /// xóa đệ quy ở đó **không phép thử nào bắt được** — đã đo bằng đột biến.
    /// Khe hở thật nằm giữa lúc kết luận rỗng và lúc hạ tay: tiến trình khác kịp
    /// ghi một tệp vào thì xóa đệ quy cuốn luôn tệp ấy.
    #[test]
    fn xoa_thu_muc_tu_choi_thu_muc_con_tep_ben_trong() {
        let h = Hop::moi("khongdequy");
        let d = h.0.join("co_tep");
        std::fs::create_dir_all(&d).unwrap();
        let t = d.join("tep_vua_xuat_hien.bin");
        std::fs::write(&t, b"du lieu that").unwrap();

        assert!(
            !xoa_thu_muc_neu_rong(&d),
            "đã xóa một thư mục ĐANG CÓ tệp — đây là xóa đệ quy trá hình"
        );
        assert!(t.is_file(), "tệp bên trong đã bị cuốn theo");
        assert!(d.is_dir());

        // Rỗng rồi thì phải xóa được, nếu không hàm này vô dụng.
        std::fs::remove_file(&t).unwrap();
        assert!(xoa_thu_muc_neu_rong(&d));
        assert!(!d.exists());
    }

    #[test]
    fn don_thu_muc_rong_khong_dung_toi_goc_o_dia() {
        // Gốc ổ đĩa phải bị bỏ qua ở cả hai dạng viết.
        assert_eq!(
            don_thu_muc_rong(&[r"C:\".into(), "C:".into()], false, &vbv_rong()),
            0
        );
    }

    #[test]
    fn don_thu_muc_rong_xoa_ca_cay_rong_nhung_giu_cap_mot() {
        let h = Hop::moi("rong");
        std::fs::create_dir_all(h.0.join("video/c1/c2/c3")).unwrap();
        std::fs::create_dir_all(h.0.join("picture")).unwrap();
        let n = don_thu_muc_rong(&[h.0.to_string_lossy().to_string()], true, &vbv_rong());
        assert_eq!(n, 3, "phải xóa c1, c2, c3");
        assert!(h.0.join("video").is_dir(), "thư mục cấp một phải được giữ");
        assert!(h.0.join("picture").is_dir());
        assert!(!h.0.join("video/c1").exists());
    }

    #[test]
    fn don_thu_muc_rong_khong_dung_thu_muc_con_tep() {
        let h = Hop::moi("corong");
        h.tep("video/c1/co_tep", 5);
        std::fs::create_dir_all(h.0.join("video/c2")).unwrap();
        let n = don_thu_muc_rong(&[h.0.to_string_lossy().to_string()], true, &vbv_rong());
        assert_eq!(n, 1, "chỉ được xóa c2");
        assert!(h.0.join("video/c1").is_dir());
    }

    #[test]
    fn don_thu_muc_rong_khong_dung_vung_bao_ve() {
        let h = Hop::moi("rongchan");
        let cam = h.0.join("video/Database");
        std::fs::create_dir_all(&cam).unwrap();
        let vbv = VungBaoVe::dung(
            &[Luat {
                duong_dan: cam.to_string_lossy().to_string(),
                muc: Muc::TatCa,
            }],
            "",
            &[],
        );
        let n = don_thu_muc_rong(&[h.0.to_string_lossy().to_string()], true, &vbv);
        assert_eq!(n, 0, "đã xóa một thư mục thuộc vùng bảo vệ");
        assert!(cam.is_dir());
    }

    #[test]
    fn duong_dan_dai_chi_them_tien_to_khi_can_va_khong_them_hai_lan() {
        assert_eq!(duong_dan_dai(r"C:\a"), r"C:\a");
        let dai = format!(r"C:\{}", "x".repeat(300));
        assert_eq!(duong_dan_dai(&dai), format!(r"\\?\{dai}"));
        let da_co = format!(r"\\?\C:\{}", "x".repeat(300));
        assert_eq!(
            duong_dan_dai(&da_co),
            da_co,
            "không được nối tiền tố hai lần"
        );
        let unc = format!(r"\\may\chiase\{}", "x".repeat(300));
        assert!(duong_dan_dai(&unc).starts_with(r"\\?\UNC\"));
    }

    #[test]
    fn cat_cut_dua_tep_ve_khong_byte() {
        let h = Hop::moi("catcut");
        let p = h.tep("a.bin", 5000);
        assert!(cat_cut(&p));
        assert_eq!(std::fs::metadata(&p).unwrap().len(), 0);
    }
}
