//! Vỏ đồ họa — egui, đã chốt ở `QĐ-01` dựa trên số đo thật.
//!
//! **Vỏ này không chứa một quyết định xóa nào.** Nó gọi lõi [`zalo_core`] và
//! hiển thị kết quả. Mọi chốt an toàn nằm trong lõi hoặc trong ba mô-đun thuần
//! của chính vỏ này — [`xac_nhan`], [`xem_truoc`], [`muc_rui_ro`] — nên không có
//! đường nào để mã vẽ lách qua, kể cả do sơ ý.
//!
//! # Rủi ro lớn nhất của cả dự án nằm ở đây
//!
//! An toàn của bản dòng lệnh đến phần lớn từ **ma sát**: phải quét mới xóa được,
//! phải gõ đủ chữ `XÓA`, phải đi qua nhiều màn hình. Giao diện đồ họa xóa sạch
//! ma sát đó — mọi thứ cách nhau một cú nhấp.
//!
//! Một giao diện đẹp khiến người ta xóa nhầm 30 GB ảnh trong ba giây là **thất
//! bại của cả dự án**, không phải một lỗi nhỏ.
//!
//! Nên ma sát được dựng lại có chủ đích, và **mỗi mảnh ma sát là một mô-đun
//! thuần có phép thử riêng**: không thể "giữ phím Enter năm giây" trong một hàm
//! `#[test]`, nhưng bơm năm nghìn sự kiện `Enter` vào một máy trạng thái thì
//! được.
//!
//! Mốc **M5**.

pub mod muc_rui_ro;
pub mod nen;
pub mod phong;
pub mod ung_dung;
pub mod xac_nhan;
pub mod xem_truoc;
