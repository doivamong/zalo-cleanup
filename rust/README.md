# Bản Rust

Đang ở **mốc M0** — khung sườn và cổng kiến trúc. Chưa có tính năng nào.

Kế hoạch đầy đủ: [`../docs/ke-hoach-port.md`](../docs/ke-hoach-port.md).

---

## Nguyên tắc một câu

> `zalo-gui` phụ thuộc `zalo-core`. **`zalo-core` không bao giờ phụ thuộc giao diện.**

Đo lúc lập kế hoạch: lõi **36 crate**, cộng `eframe` thành **112**. Nghĩa là 76 crate chỉ để vẽ cửa sổ — và chúng không được dính vào phần quyết định xóa gì.

Đây không phải khẩu hiệu. `tools\check-deps.ps1` kiểm nó bằng máy sau mỗi commit, và CI chạy nó.

---

## Ba crate

| Crate | Là gì | Mốc |
|:---|:---|:---|
| `zalo-core` | Lõi: quyết định xóa gì, và chặn cái gì | M1–M4 |
| `zalo-cli` | Vỏ dòng lệnh, nói đúng giao thức phím của bản PowerShell | M3 |
| `zalo-gui` | Vỏ đồ họa egui, **không chứa quyết định xóa** | M5 |

---

## Chạy tay

```powershell
cd rust
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-deps.ps1
```

---

## Mô-đun trong `zalo-core`

Mỗi tệp mở đầu bằng khối chú thích ghi rõ **ràng buộc phải giữ** và **mốc nào làm**. Đọc chúng trước khi viết — phần lớn là bài học đã trả giá bằng dữ liệu thật, và gần như cái nào cũng phản trực giác.

`contract` là đặc biệt: nó giữ những giá trị **là hợp đồng giữa hai bản**, và phép thử trong đó **đọc thẳng `ZaloCleanup.ps1`** để bắt trôi. Đó là lưới đỡ duy nhất chống chuyện hai bản lặng lẽ trở thành hai công cụ khác nhau.

---

## Nhắc lại điều dễ quên nhất

Công cụ này **xóa vĩnh viễn dữ liệu cá nhân, không qua Thùng rác**. Chủ dự án đã dùng nó xóa 149.309 tệp / 37 GB ảnh và video thật.

Sửa một lớp an toàn thì phải **kiểm bằng đột biến**: cố tình phá rồi xem test có đỏ không. Test không đỏ nghĩa là test vô dụng, không phải mã đúng — chuyện đó đã xảy ra bốn lần trong dự án này.
