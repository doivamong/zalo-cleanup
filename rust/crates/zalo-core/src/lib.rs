//! Lõi của công cụ Dọn dẹp Zalo.
//!
//! # Nguyên tắc một câu
//!
//! `zalo-gui` phụ thuộc `zalo-core`. **`zalo-core` không bao giờ phụ thuộc giao
//! diện.** Lõi phải biên dịch và kiểm thử được mà không có một dòng giao diện
//! nào trong cây phụ thuộc. Đây không phải khẩu hiệu — `rust\tools\check-deps.ps1`
//! kiểm nó bằng máy sau mỗi commit.
//!
//! # Ranh giới không được vượt
//!
//! - Mọi hàm trong [`act`] nhận kết quả quét **đã được [`gate`] duyệt**, không tự quyết.
//! - Giao diện không được gọi thẳng [`act`]; phải đi qua một lớp lệnh của lõi.
//! - Lõi **không in ra màn hình**. Trả về dữ liệu, người gọi tự hiển thị — nhờ
//!   vậy cùng một lõi phục vụ được cả `zalo-cli` lẫn `zalo-gui`.
//! - Không dùng `unsafe` ngoài [`sysinfo`].
//!
//! # Năm nguyên tắc bất biến của công cụ
//!
//! Bản Rust phải giữ đủ cả năm, y như bản PowerShell:
//!
//! 1. Không quét thì không thể xóa.
//! 2. Đổi bộ lọc là kết quả quét cũ bị hủy.
//! 3. Nhập sai thì giữ nguyên, không bao giờ tự mở rộng phạm vi.
//! 4. Vùng bảo vệ bị chặn cứng ở tầng code.
//! 5. Sao lưu chưa sạch thì không cho xóa.
//!
//! Cộng thêm: công cụ **không tự chạy** — không Scheduled Task, không hook,
//! không tiến trình nền, không kết nối mạng.
//!
//! Xem `docs/ke-hoach-port.md` để biết mốc nào làm mô-đun nào.

pub mod contract;

// ---------------------------------------------------------------- M1 · lõi an toàn
pub mod confirm;
pub mod gate;
pub mod protect;

// ---------------------------------------------------------------- M2 · quét và băm
pub mod hash;
pub mod scan;
pub mod walk;

// ---------------------------------------------------------------- M2/M4 · hạ tầng
pub mod lock;
pub mod store;
pub mod sysinfo;
pub mod thoigian;

// ---------------------------------------------------------------- M4 · động tới dữ liệu
pub mod act;
