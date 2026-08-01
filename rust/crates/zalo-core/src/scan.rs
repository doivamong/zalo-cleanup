//! Bốn chế độ quét và trạng thái kết quả quét.
//!
//! Trạng thái này là trung tâm của nguyên tắc bất biến 1 và 2: không quét thì
//! không xóa được, và đổi bộ lọc là kết quả cũ bị hủy.
//!
//! Khử trùng lặp kết luận **chỉ bằng SHA-256 toàn tệp**, không bao giờ bằng tên
//! tệp — hai nơi lưu đặt tên theo hai quy ước khác nhau. Mốc **M2**.
