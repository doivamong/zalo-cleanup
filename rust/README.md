# Bản Rust

Đã qua **M0** (khung sườn + cổng kiến trúc), **M1** (lõi an toàn) và **M2** (duyệt cây, băm, bộ lọc quét).

Ngay sau M2 là **mốc kiểm điểm bắt buộc** của kế hoạch: đã biết lõi port có đúng
không và có nhanh hơn thật không, mà chưa tiêu công vào giao diện — chỗ rẻ nhất
để đổi ý. Tiếp theo là **M3**, chế độ headless.

Kế hoạch đầy đủ và bảng trạng thái từng mốc: [`../docs/ke-hoach-port.md`](../docs/ke-hoach-port.md).

> **Chưa dùng được.** `zalo-cli` và `zalo-gui` mới chỉ là vỏ rỗng. Công cụ thật đang chạy vẫn là bản PowerShell ở thư mục gốc.

---

## Nguyên tắc một câu

> `zalo-gui` phụ thuộc `zalo-core`. **`zalo-core` không bao giờ phụ thuộc giao diện.**

Dự phóng lúc lập kế hoạch: lõi **36 crate**, cộng `eframe` thành **112**. Nghĩa là 76 crate chỉ để vẽ cửa sổ — và chúng không được dính vào phần quyết định xóa gì.

Đo thật tới hết M2: lõi mới dùng **13 crate**. Và `walkdir` đã bị loại ở M2, nên trần cuối cùng thấp hơn con số 36 kia.

Đây không phải khẩu hiệu. `tools\check-deps.ps1` kiểm nó bằng máy sau mỗi commit, và CI chạy nó.

---

## Ba crate

| Crate | Là gì | Mốc | Trạng thái |
|:---|:---|:---|:---|
| `zalo-core` | Lõi: quyết định xóa gì, và chặn cái gì | M1–M4 | `protect` `confirm` `gate` `contract` `walk` `hash` `scan` **đã có**; `act` `sysinfo` `store` `lock` còn là vỏ rỗng |
| `zalo-cli` | Vỏ dòng lệnh, nói đúng giao thức phím của bản PowerShell | M3 | vỏ rỗng |
| `zalo-gui` | Vỏ đồ họa egui, **không chứa quyết định xóa** | M5 | vỏ rỗng |

## Bộ đối chiếu song song — thứ chứng minh hai bản còn khớp

[`crates/zalo-core/tests/doi_chieu_song_song.rs`](crates/zalo-core/tests/doi_chieu_song_song.rs) đẩy cùng một tập đầu vào qua cả hai bản rồi so từng kết quả. Đo trên dữ liệu Zalo thật:

| Đối chiếu | Đầu vào | Khác biệt |
|:---|---:|---:|
| Vùng bảo vệ | 57.604 | **0** |
| Duyệt cây | 57.351 tệp · 0 lỗi | **0** |
| Băm (31 nhánh `FULL:` · 39 nhánh `Q:`) | 70 tệp | **0** |
| Phần mở rộng kiểu .NET | 310 tên tệp | **0** |
| Bỏ dấu thanh | 160 chuỗi | **0** |
| Thư mục gốc | 31 | **0** |

Bản thần chú phía PowerShell — [`tests/oracle-protect.ps1`](tests/oracle-protect.ps1) — **bóc thẳng hàm ra khỏi `ZaloCleanup.ps1` bằng AST**, không chép lại logic. Chép là hai bản trôi khỏi nhau, mà bắt trôi mới là điểm của bộ đối chiếu.

Trên máy chủ CI không có dữ liệu Zalo thật nên nó chỉ chạy phần ca dựng máy móc, và **tự in ra dòng `CHÚ Ý`** nói rõ đã bỏ qua phần nào. Chạy đầy đủ cần một máy có dữ liệu, hoặc đặt biến `ZALO_DOI_CHIEU_GOC`.

## Cổng M2 đã đạt

Ngoài đối chiếu ở trên, hai cổng còn lại:

**Junction.** Junction NTFS không phải symlink, nên "thư viện không theo symlink" chưa chắc đã chặn nó. Đã dựng junction thật bằng `mklink /J` rồi đo: không đi xuyên. Phép đo này **loại luôn `walkdir`** khỏi danh sách phụ thuộc — nó cũng không đi xuyên, nhưng bản tự duyệt bằng `std::fs` cũng vậy mà không tốn thêm crate nào.

**Tốc độ.** `cargo run --release --example do_toc_do -p zalo-core` quét cây thật: **0,507 s** so với ngưỡng 1,5 s, tức nhanh hơn bản PowerShell **10,2 lần**. Cố ý để ở dạng `example` chứ không phải `#[test]` — ghim một ngưỡng giây vào bộ test trên máy chủ CI là mời một phép thử chập chờn, mà phép thử chập chờn thì sớm muộn cũng bị tắt đi, kéo theo cả những phép thử thật nằm cạnh.

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

Sửa một lớp an toàn thì phải **kiểm bằng đột biến**: cố tình phá rồi xem test có đỏ không. Test không đỏ nghĩa là test vô dụng, không phải mã đúng — chuyện đó đã xảy ra **năm lần** trong dự án này, gần nhất là ở chính mốc M2: gỡ hẳn chốt reparse point mà bộ test vẫn xanh, vì có một lớp phòng thủ thứ hai che mất.
