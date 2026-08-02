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
            Ok(Box::new(ung_dung::UngDung::moi(nguon)))
        }),
    )
}

/// Cài phông đã nạp làm phông duy nhất.
///
/// Ghi đè cả hai họ `Proportional` và `Monospace`: phông mặc định của egui bị
/// tắt hẳn ở `Cargo.toml`, nên để trống một họ nghĩa là chỗ đó không vẽ được
/// chữ nào.
fn dat_phong(ctx: &egui::Context, byte: Vec<u8>) {
    let mut f = egui::FontDefinitions::empty();
    f.font_data
        .insert("viet".to_owned(), egui::FontData::from_owned(byte));
    for ho in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        f.families
            .entry(ho)
            .or_default()
            .insert(0, "viet".to_owned());
    }
    ctx.set_fonts(f);

    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        // DPI-05: nút Hủy và nút Xóa không được nằm sát nhau. Đệm rộng cộng
        // khoảng cách mục đẩy hai nút ra xa nhau ở mọi màn hình.
        s.spacing.button_padding = egui::vec2(14.0, 10.0);
    });
}
