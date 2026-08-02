//! Vỏ dòng lệnh — nhận **đúng giao thức phím qua stdin** như bản PowerShell.
//!
//! # Ba việc nó phục vụ cùng lúc
//!
//! 1. Các phép thử đầu-cuối hiện có chạy được **không sửa một ký tự**, chỉ đổi
//!    đường dẫn công cụ. Đó là cổng của mốc **M3**.
//! 2. Bộ so sánh song song có công cụ lái cả hai bản y như nhau.
//! 3. Người muốn đọc mã trước khi chạy vẫn có đường dùng, không cần giao diện.
//!
//! # Một điều tuyệt đối cấm
//!
//! Vỏ này đi qua **đúng lõi và đúng mọi chốt** như giao diện. Nó là một cái vỏ
//! khác, không phải một đường tắt. Thêm cờ kiểu `--yes` để bỏ qua xác nhận là
//! phá nguyên tắc bất biến số 1, và biến công cụ thành đúng thứ nó ra đời để
//! chống lại.
//!
//! # Tham số
//!
//! Nhận `-Root` và `-DataRoot` **đúng cách viết của PowerShell** — bộ test gọi
//! thẳng bằng hai tên đó. Chấp nhận cả `-Root X` lẫn `-Root=X`, và không phân
//! biệt hoa thường, vì PowerShell cũng vậy.

mod hien;
mod nhap;
mod ung_dung;

fn main() {
    let mut goc = String::new();
    let mut goc_du_lieu = String::new();

    let tham_so: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < tham_so.len() {
        let (ten, gan) = match tham_so[i].split_once('=') {
            Some((a, b)) => (a.to_string(), Some(b.to_string())),
            None => (tham_so[i].clone(), None),
        };
        let ten = ten.trim_start_matches('-').to_lowercase();
        let dich: Option<&mut String> = match ten.as_str() {
            "root" => Some(&mut goc),
            "dataroot" => Some(&mut goc_du_lieu),
            _ => None,
        };
        if let Some(d) = dich {
            if let Some(v) = gan {
                *d = v;
            } else if i + 1 < tham_so.len() {
                *d = tham_so[i + 1].clone();
                i += 1;
            }
        }
        i += 1;
    }

    // Suy ra thư mục ZaloData từ gốc quét khi không được chỉ định: gốc quét là
    // ...\ZaloData\media\<tài khoản>\ZaloDownloads, nên lùi ba cấp.
    if goc_du_lieu.trim().is_empty() && !goc.trim().is_empty() {
        let p = std::path::Path::new(&goc);
        if let Some(x) = p.parent().and_then(|x| x.parent()).and_then(|x| x.parent()) {
            goc_du_lieu = x.to_string_lossy().to_string();
        }
    }

    let mut app = ung_dung::UngDung::moi(goc, goc_du_lieu);
    app.chay();
}
