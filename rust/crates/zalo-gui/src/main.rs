//! Điểm khởi động của bản đồ họa. Toàn bộ mã nằm ở thư viện cùng tên —
//! xem [`zalo_gui`].
//!
//! Tách lib khỏi bin không phải để cho gọn: mọi chốt an toàn của giao diện đều
//! là **bề mặt công khai có phép thử**, và một tệp `main.rs` đơn độc thì mọi
//! thứ chỉ dùng trong `#[cfg(test)]` đều bị coi là mã chết.

// Không mở cửa sổ console phía sau cửa sổ đồ họa. `main` vẫn chạy như thường.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use zalo_gui::{phong, ung_dung};
fn main() -> eframe::Result<()> {
    let (byte_phong, nguon) = phong::nap();
    let goc = doc_tham_so_root();

    // KHÓA MỘT TIẾN TRÌNH MỘT LÚC (`R-16`), lấy trước khi mở cửa sổ.
    //
    // Bị chặn thì **vẫn mở cửa sổ**, chỉ vẽ một màn nói ai đang giữ. Chết lặng
    // là người dùng bấm biểu tượng, không thấy gì, rồi bấm tiếp — và kết luận
    // công cụ hỏng. Cửa sổ ấy cũng là thứ trình đọc màn hình đọc được.
    //
    // Khóa giao thẳng cho `UngDung` giữ, không giữ ở đây: cuộc bàn giao của
    // `RB-08` phải **nhả khóa** trước khi khởi chạy bản dòng lệnh, và chỗ bấm
    // nút nằm trong ứng dụng.
    let (khoa, chan_boi) = match zalo_core::lock::vao("zalo-gui") {
        zalo_core::lock::KetQuaKhoa::DiTiep(k) => (Some(k), None),
        zalo_core::lock::KetQuaKhoa::BanKhacDangMo(ai) => (None, Some(ai)),
    };

    let tuy_chon = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Dọn dẹp Zalo")
            // DPI-04: phải vừa màn 1366×768 ở 125%, tức 1092×614 dip. Cỡ mở đầu
            // nằm dưới ngưỡng đó.
            .with_inner_size([1040.0, 640.0])
            // DPI-05: dưới cỡ này thì nút Hủy bắt đầu bị đẩy khỏi màn hình.
            .with_min_inner_size([940.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Dọn dẹp Zalo",
        tuy_chon,
        Box::new(move |cc| {
            dat_phong(&cc.egui_ctx, byte_phong);
            Ok(Box::new(ung_dung::UngDung::moi(nguon, goc, chan_boi, khoa)))
        }),
    )
}

/// Cài phông đã nạp làm phông duy nhất.
///
/// Ghi đè cả hai họ `Proportional` và `Monospace`: phông mặc định của egui bị
/// tắt hẳn ở `Cargo.toml`, nên để trống một họ nghĩa là chỗ đó không vẽ được
/// chữ nào.
/// Đọc `-Root` từ dòng lệnh. Cùng cách viết với bản dòng lệnh, và cũng nhận cả
/// `-Root X` lẫn `-Root=X`, không phân biệt hoa thường.
fn doc_tham_so_root() -> Option<String> {
    let t: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < t.len() {
        let (ten, gan) = match t[i].split_once('=') {
            Some((a, b)) => (a.to_string(), Some(b.to_string())),
            None => (t[i].clone(), None),
        };
        if ten.trim_start_matches('-').eq_ignore_ascii_case("root") {
            if let Some(v) = gan {
                return Some(v);
            }
            return t.get(i + 1).cloned();
        }
        i += 1;
    }
    None
}

fn dat_phong(ctx: &egui::Context, chuoi: Vec<(String, Vec<u8>)>) {
    let mut f = egui::FontDefinitions::empty();
    let mut ten: Vec<String> = Vec::new();
    for (n, b) in chuoi {
        f.font_data.insert(n.clone(), egui::FontData::from_owned(b));
        ten.push(n);
    }
    // THỨ TỰ trong danh sách chính là thứ tự egui thử glyph. Phông hệ thống
    // đứng trước cho chữ quen mắt, phông nhúng đứng sau lấp glyph còn thiếu —
    // Segoe UI phủ đủ chữ Việt nhưng thiếu bốn ký hiệu của bảng, và thiếu đúng
    // ở những chỗ nói về an toàn.
    for ho in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        *f.families.entry(ho).or_default() = ten.clone();
    }
    ctx.set_fonts(f);

    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        // DPI-05: nút Hủy và nút Xóa không được nằm sát nhau. Đệm rộng cộng
        // khoảng cách mục đẩy hai nút ra xa nhau ở mọi màn hình.
        s.spacing.button_padding = egui::vec2(14.0, 10.0);
    });
}
