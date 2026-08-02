# Bản Rust

Đã qua **M0** (khung sườn + cổng kiến trúc), **M1** (lõi an toàn), **M2** (duyệt
cây, băm, bộ lọc quét), **M3** (vỏ dòng lệnh) và **M4** (xóa · sao lưu · khôi phục).

**M5** (giao diện đồ họa) đã xong phần kiểm được bằng máy. Còn đúng một thứ chặn nó: **9 mục mức 1 của danh mục tiếp cận cần người thật ngồi trước màn hình.**

Kế hoạch đầy đủ và bảng trạng thái từng mốc: [`../docs/ke-hoach-port.md`](../docs/ke-hoach-port.md).

> **Cả hai vỏ đều dọn dẹp được thật.** `zalo-cli` quét, sao lưu, xóa và khôi
> phục — cùng một bộ test lái được cả nó lẫn bản PowerShell và cho cùng kết quả.
> `zalo-gui` quét, xem trước ảnh, sao lưu, xóa và khôi phục được.
>
> Bản dành cho người dùng thường **vẫn là bản PowerShell** ở thư mục gốc: giao
> diện còn chín mục mức 1 của danh mục tiếp cận chưa ai kiểm.

---

## Nguyên tắc một câu

> `zalo-gui` phụ thuộc `zalo-core`. **`zalo-core` không bao giờ phụ thuộc giao diện.**

Dự phóng lúc lập kế hoạch: lõi **36 crate**, cộng `eframe` thành **112**. Nghĩa là 76 crate chỉ để vẽ cửa sổ — và chúng không được dính vào phần quyết định xóa gì.

Đo thật sau M5: lõi **17 crate**, cả cây có giao diện **145** — tức **128 crate chỉ để vẽ cửa sổ và giải mã ảnh**, nhiều hơn dự phóng 76. Exe giao diện **3,61 MiB**; trước khi thêm bộ giải mã JPEG XL nó là 2,64 MiB, nhỏ hơn dự phóng 2,86 MiB của hội đồng.

Đây không phải khẩu hiệu. `tools\check-deps.ps1` kiểm nó bằng máy sau mỗi commit, và CI chạy nó.

---

## Ba crate

| Crate | Là gì | Mốc | Trạng thái |
|:---|:---|:---|:---|
| `zalo-core` | Lõi: quyết định xóa gì, và chặn cái gì | M1–M4 | tất cả **đã có** trừ `lock` |
| `zalo-cli` | Vỏ dòng lệnh, nói đúng giao thức phím của bản PowerShell | M3–M4 | **đã có** |
| `zalo-gui` | Vỏ đồ họa egui, **không chứa quyết định xóa** | M5 | **đã có** |

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

## Cổng đối chiếu — một bộ test, hai công cụ

`ZaloCleanup.Tests.ps1` giờ lái được **cả hai bản**. Không có bản test thứ hai: chép ra làm hai bản là hai bộ test trôi khỏi nhau, và lúc đó "cả hai đều xanh" chẳng chứng minh được gì về hai công cụ.

Điểm hoán đổi là biến môi trường `ZALO_TOOL`. Bên trong bộ test, `$tool` giữ nguyên nghĩa **mã nguồn PowerShell** — hơn một trăm phép thử đem nó ra soi bằng AST — còn `$toolChay` mới là **thứ bị lái**.

| | |
|:---|---:|
| Phép thử đầu-cuối, kể cả `-Full` | **67** |
| Đạt khi lái bản Rust | **67** |
| Phép thử liên thông hai chiều | **19/19** |

Bộ chạy [`tools/cong-song-song.ps1`](tools/cong-song-song.ps1) tách riêng 135 phép thử **soi mã nguồn PowerShell** ra khỏi con số: chúng luôn xanh bất kể bản Rust đúng hay sai, nên gộp vào là tự thổi phồng bằng chứng.

Bộ test chạy mỗi bản một mình thì chứng minh được hai bản làm **đúng**, nhưng không chứng minh được chúng **đọc được của nhau**. Đó là việc của [`tools/cong-lien-thong.ps1`](tools/cong-lien-thong.ps1): sao lưu bằng bản này, khôi phục bằng bản kia, rồi so **SHA-256 từng tệp** — không so số lượng, vì số lượng khớp mà nội dung hỏng là đúng loại lỗi mà một bản sao lưu sinh ra để chống.

### Đột biến tìm ra những lỗ mà chính cổng này không bịt được

Cổng đạt ngay lượt đầu, cả ở M3 lẫn M4 — con số đó phải nghi ngờ chứ không mừng. Mười ba đột biến qua hai mốc, và **bốn trong số đó không bị bắt**:

| Đột biến | Vì sao cổng để lọt |
|:---|:---|
| Nhập sai thì âm thầm chọn tất cả | phép thử chỉ kiểm **câu chữ in ra**, không kiểm bộ lọc |
| Gỡ chốt vùng bảo vệ khỏi vòng quét | phép thử tương ứng nằm trong lượt **có xóa tệp** |
| Bản trùng lặp bị đòi gõ `XÓA` như dữ liệu thật | như trên |
| Dọn thư mục rỗng bằng xóa **đệ quy** | mọi thư mục đưa tới đó **đã rỗng sẵn**, hai hàm cho cùng kết quả |

Cái cuối là ví dụ đẹp nhất: khe hở thật nằm giữa lúc kết luận "thư mục này rỗng" và lúc hạ tay, mà bộ test không dựng lại được khe ấy. Bịt bằng cách tách ra `xoa_thu_muc_neu_rong` rồi hỏi thẳng — *đưa cho nó một thư mục CÓ tệp thì sao?*

Bốn lỗ đều đã bịt bằng hàm thuần có phép thử riêng: `phan_tich_chon_thu_muc`, `xet_tep`, `muc_xac_nhan`, `xoa_thu_muc_neu_rong`. Đo lại: **cả mười ba đột biến đều đỏ.**

---

## Giao diện đồ họa — ma sát là tính năng, không phải phiền toái

An toàn của bản dòng lệnh đến phần lớn từ **ma sát**: phải quét mới xóa được, phải gõ đủ chữ `XÓA`, phải đi qua nhiều màn hình. Giao diện đồ họa xóa sạch ma sát đó — mọi thứ cách nhau một cú nhấp.

Nên ma sát được dựng lại có chủ đích, và **mỗi mảnh là một mô-đun thuần kiểm được không cần vẽ**:

| Mô-đun | Canh điều gì |
|:---|:---|
| `xac_nhan` | Enter không kích hoạt gì · khóa mồi 600 ms tính lại **mỗi lần** nút bật · chặn dán · bỏ phím tự lặp · bấm rồi không nhận thêm |
| `xem_truoc` | Chưa mở danh sách tệp sắp mất thì nút xóa **không bật**. Quét lại là chốt đóng lại |
| `muc_rui_ro` | Chữ · ký hiệu · rồi mới tới màu. Bỏ hết màu vẫn phải phân loại đúng |

Không thể "giữ phím Enter năm giây" trong một hàm `#[test]` — nhưng bơm năm nghìn sự kiện `Enter` vào một máy trạng thái thì được.

### Ba lỗ thật, tìm ra bằng cách hỏi máy chứ không bằng cách nhìn màn hình

**Phông thiếu glyph.** Thiết kế dùng `⛨` làm huy hiệu vùng bảo vệ, mà phông nhúng không có nó — nó sẽ hiện thành **ô vuông rỗng**, thứ còn tệ hơn không có huy hiệu nào. Đã thay bằng `⊘` và gom mọi ký hiệu vào một bảng có phép thử quét toàn bộ.

**egui kích hoạt nút bằng Enter và Space.** `Response::clicked()` trả `true` y như bấm chuột, tức luật "Enter không kích hoạt gì" bị lách **ngay ở tầng thư viện** — chỗ máy trạng thái không nhìn thấy được.

**Lõi chưa có đường hủy.** Esc phải dừng được lượt xóa đang chạy *và* nhật ký phải ghi "đã hủy giữa chừng". Thiếu vế cuối là người dùng bấm Esc rồi mở nhật ký thấy `hoàn tất=True`.

### Một lỗ nữa, chỉ lộ ra khi mở ứng dụng lên

Màn hình hiện `? Xong.` thay vì `✓ Xong.` — dấu tích thành ô vuông rỗng.

Đo tận nơi: **Segoe UI phủ đủ 134 chữ cái tiếng Việt nhưng thiếu bốn trên tám ký hiệu** — `⊘ ⚠ ✓ ✖`. Phép thử phủ glyph chỉ hỏi phông **nhúng**, không hỏi phông hệ thống đang thật sự dùng, nên nó xanh trong khi màn hình hỏng.

Sửa bằng **chuỗi phông**: hệ thống cho chữ quen mắt, phông nhúng lấp glyph thiếu. Phép thử mới hỏi **cả chuỗi gộp lại**, không hỏi từng phông một.

### Ảnh xem trước

Mười hai ảnh lấy ngẫu nhiên, giải mã **ngoài luồng vẽ**, nhận dạng bằng **magic byte** chứ không bằng phần mở rộng — 43,7% tệp Zalo không có phần mở rộng, mà 88,5% trong số đó là JPEG.

Có bộ giải mã **JPEG XL** vì `.jxl` chiếm **46,4%** dữ liệu thật; phép thử giải mã tệp `.jxl` **thật của Zalo**, không phải tệp dựng máy móc. Giá: exe từ 2,64 lên 3,61 MiB.

Tệp không xem trước được hiện ô `?` và **vẫn nằm trong danh sách**. Giấu đi là người dùng xóa một thứ họ chưa từng nhìn thấy mà lại tưởng mình đã xem hết.

### §8.1-1 đã chạy trên giao diện thật — 8/8

[`tools/phep-thu-ma-sat.ps1`](tools/phep-thu-ma-sat.ps1) lái chuột và bàn phím thật trên hộp cát 30 tệp trong `%TEMP%`: giữ **Enter** 5 giây, giữ **Space** 5 giây, **nhấp 200 lần** vào đúng tọa độ nút Xóa, gõ `xoa` chữ thường, gõ đúng cụm từ rồi nhấp **ngay lập tức**. Cả sáu: **0 tệp biến mất**.

Vế thứ bảy mới là vế quan trọng nhất — **kiểm ngược**: chờ hết khóa mồi rồi nhấp thì **xóa được thật**. Không có nó thì "0 tệp biến mất" có thể chỉ vì cú nhấp trượt, và sáu phép thử trên thành vô nghĩa.

Hai lần bộ chạy báo hỏng, **cả hai đều là lỗi của chính nó**: tọa độ tính theo `GetWindowRect` thay vì `ClientToScreen` (khung winit có viền vô hình), và `keybd_event` gửi phím trần nên ô nhập nhận `xoa` chữ thường — mà chữ thường thì đúng là không được mở khóa.

Không chạy được trên CI vì nó cần một phiên màn hình thật.

### Chưa xong, nói thẳng

Chín mục **mức 1** của danh mục tiếp cận cần người thật ngồi trước màn hình — ảnh greyscale ba người thử, gõ `XOÁ` bằng Unikey, chỉ dùng bàn phím, giam tiêu điểm, màn 1366×768, canh giữa cửa sổ cha, `MAU-01`, `MAU-09`, `ĐM-08` với NVDA thật.

[`tools/cong-m5.ps1`](tools/cong-m5.ps1) in thẳng tên chúng sau mỗi lần chạy. Đây là thứ duy nhất còn chặn M5, và nó chặn bằng người chứ không bằng mã.

---

## Chạy tay

```powershell
cd rust
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-deps.ps1
```

Hai cổng đối chiếu chạy từ gốc repo, và tự dựng lại exe:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File rust\tools\cong-song-song.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File rust\tools\cong-lien-thong.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File rust\tools\cong-m5.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File rust\tools\phep-thu-ma-sat.ps1
```

---

## Mô-đun trong `zalo-core`

Mỗi tệp mở đầu bằng khối chú thích ghi rõ **ràng buộc phải giữ** và **mốc nào làm**. Đọc chúng trước khi viết — phần lớn là bài học đã trả giá bằng dữ liệu thật, và gần như cái nào cũng phản trực giác.

`contract` là đặc biệt: nó giữ những giá trị **là hợp đồng giữa hai bản**, và phép thử trong đó **đọc thẳng `ZaloCleanup.ps1`** để bắt trôi. Đó là lưới đỡ duy nhất chống chuyện hai bản lặng lẽ trở thành hai công cụ khác nhau.

---

## Nhắc lại điều dễ quên nhất

Công cụ này **xóa vĩnh viễn dữ liệu cá nhân, không qua Thùng rác**. Chủ dự án đã dùng nó xóa 149.309 tệp / 37 GB ảnh và video thật.

Sửa một lớp an toàn thì phải **kiểm bằng đột biến**: cố tình phá rồi xem test có đỏ không. Test không đỏ nghĩa là test vô dụng, không phải mã đúng — chuyện đó đã xảy ra **bảy lần** trong dự án này. Gần nhất ở mốc M4: đổi `remove_dir` thành `remove_dir_all` trong vòng dọn thư mục rỗng mà không phép thử nào đỏ, vì mọi thư mục đưa tới đó đều đã rỗng sẵn nên hai hàm cho cùng kết quả.
