//! Vỏ dòng lệnh — nhận **đúng giao thức phím qua stdin** như bản PowerShell.
//!
//! # Ba việc nó phục vụ cùng lúc
//!
//! 1. 69 phép thử đầu-cuối hiện có chạy được **không sửa một ký tự**, chỉ đổi
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

fn main() {
    eprintln!("zalo-cli: chưa triển khai. Mốc M3 — xem docs/ke-hoach-port.md");
    std::process::exit(2);
}
