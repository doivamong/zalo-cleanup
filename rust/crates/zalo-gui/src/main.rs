//! Vỏ đồ họa — egui, đã chốt ở `QĐ-01` dựa trên số đo thật.
//!
//! **Vỏ này không chứa một quyết định xóa nào.** Nó gọi lớp lệnh của
//! [`zalo_core`] và hiển thị kết quả. Mọi chốt an toàn nằm trong lõi, nên không
//! có đường nào để giao diện lách qua, kể cả do sơ ý.
//!
//! # Rủi ro lớn nhất của cả dự án nằm ở tệp này
//!
//! An toàn của bản dòng lệnh đến phần lớn từ **ma sát** — phải quét mới xóa
//! được, phải gõ đủ chữ `XÓA`, phải đi qua nhiều màn hình. Giao diện đồ họa xóa
//! sạch ma sát đó, mọi thứ cách nhau một cú nhấp.
//!
//! Một giao diện đẹp khiến người ta xóa nhầm 30 GB ảnh trong ba giây là **thất
//! bại của cả dự án**, không phải một lỗi nhỏ. Đọc `docs/ui-ux-council.md`
//! trước khi viết dòng giao diện đầu tiên.
//!
//! Mốc **M5**.

fn main() {
    eprintln!("zalo-gui: chưa triển khai. Mốc M5 — xem docs/ui-ux-council.md");
    std::process::exit(2);
}
