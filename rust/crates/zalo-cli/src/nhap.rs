//! Đọc phím từ stdin — **giao thức phím** mà bộ test đầu-cuối lái công cụ bằng.
//!
//! # Vì sao phải phân biệt "bấm Enter" với "hết luồng nhập"
//!
//! Bấm Enter trả về chuỗi rỗng; luồng nhập cạn cũng dễ bị hiểu thành chuỗi rỗng.
//! Lẫn hai thứ đó là vòng lặp menu quay vô tận khi chạy phi tương tác — đúng
//! cảnh mọi phép thử đầu-cuối đều rơi vào, vì chúng đẩy một chuỗi phím hữu hạn
//! rồi đóng luồng.
//!
//! Bản PowerShell giải bằng cách đếm số lần chạm đáy liên tiếp và thoát ở lần
//! thứ năm. Bản này giữ **đúng con số đó**: nó là hợp đồng quan sát được, vì một
//! phép thử đẩy ít phím hơn số menu sẽ dựa vào đúng chỗ ấy để công cụ tự thoát.

use std::io::{BufRead, Write};

/// Bóc mọi dấu BOM UTF-8 ở đầu chuỗi.
///
/// # Đây là lỗi thật, tìm ra bằng cách chạy chứ không bằng cách đọc
///
/// PowerShell 5.1 đẩy dữ liệu sang chương trình ngoài bằng `$OutputEncoding`,
/// mà `[Text.Encoding]::UTF8` của .NET **có phát BOM**. Nên phím đầu tiên tới
/// nơi không phải `0` mà là `\u{feff}0`, và đo tận nơi thì có tới **hai** dấu
/// BOM liền nhau.
///
/// Hậu quả quan sát được: mọi phím đều rơi vào nhánh "không hiểu lựa chọn", menu
/// quay vòng cho tới khi cạn luồng nhập. Bản PowerShell không dính vì `Read-Host`
/// tự bóc BOM giúp.
///
/// Bóc **lặp** chứ không bóc một lần: hai BOM là chuyện đã đo được, và một chuỗi
/// còn sót BOM thì im lặng sai chứ không báo lỗi.
fn bo_bom(s: &str) -> &str {
    let mut r = s;
    while let Some(x) = r.strip_prefix('\u{feff}') {
        r = x;
    }
    r
}

/// Số lần chạm đáy luồng nhập liên tiếp thì thoát. Hợp đồng với bản PowerShell.
pub const NGUONG_CAN_LUONG: u32 = 5;

pub struct Nhap {
    chuoi_can: u32,
}

impl Default for Nhap {
    fn default() -> Self {
        Nhap::moi()
    }
}

impl Nhap {
    pub fn moi() -> Self {
        Nhap { chuoi_can: 0 }
    }

    /// Đọc một dòng, đã cắt khoảng trắng hai đầu. Tương ứng `Read-Line`.
    ///
    /// Trả `None` khi luồng nhập đã cạn tới ngưỡng — người gọi phải hiểu đó là
    /// lệnh **thoát ngay**, không phải một lựa chọn rỗng.
    pub fn dong(&mut self, loi_nhac: &str) -> Option<String> {
        print!("{loi_nhac}: ");
        let _ = std::io::stdout().flush();
        let mut d = String::new();
        match std::io::stdin().lock().read_line(&mut d) {
            Ok(0) | Err(_) => {
                self.chuoi_can += 1;
                println!();
                if self.chuoi_can >= NGUONG_CAN_LUONG {
                    println!();
                    println!("  Luồng nhập đã cạn. Thoát.");
                    return None;
                }
                Some(String::new())
            }
            Ok(_) => {
                self.chuoi_can = 0;
                // Vọng lại phím vừa nhận. Khi stdin là đường ống thì console
                // không tự vọng, nên thiếu dòng này bản ghi màn hình đọc ra một
                // chuỗi câu hỏi không có câu trả lời nào.
                let s = bo_bom(d.trim()).trim().to_string();
                println!("{s}");
                Some(s)
            }
        }
    }

    /// Câu hỏi có/không. Tương ứng `Read-YesNo` — nhận `c`, `có`, `co`, `y`.
    pub fn co_khong(&mut self, loi_nhac: &str) -> Option<bool> {
        let v = self.dong(loi_nhac)?.to_lowercase();
        Some(v == "c" || v == "có" || v == "co" || v == "y")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ngưỡng năm lần là hợp đồng, không phải con số tùy ý. Xem chú thích đầu tệp.
    #[test]
    fn nguong_can_luong_dung_bang_nam() {
        assert_eq!(NGUONG_CAN_LUONG, 5);
    }

    /// Lỗi này đã làm mọi phím trượt hết. Xem chú thích ở [`bo_bom`].
    #[test]
    fn boc_duoc_ca_hai_dau_bom_lien_nhau() {
        assert_eq!(bo_bom("0"), "0");
        assert_eq!(bo_bom("\u{feff}0"), "0");
        assert_eq!(bo_bom("\u{feff}\u{feff}0"), "0");
        assert_eq!(bo_bom("\u{feff}\u{feff}XÓA"), "XÓA");
        assert_eq!(bo_bom(""), "");
        assert_eq!(bo_bom("\u{feff}"), "");
        assert_eq!(bo_bom("a\u{feff}b"), "a\u{feff}b", "chỉ bóc ở ĐẦU chuỗi");
    }
}
