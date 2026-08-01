//! Chốt phải qua trước khi xóa.
//!
//! `Test-KeeperAlive` — bản giữ lại của cặp trùng lặp còn sống và còn đúng cỡ.
//! `Test-BackupClean` — sao lưu **không lỗi VÀ trọn vẹn**, hai vế chứ không một.
//!
//! Cả hai từng là lỗ hổng thật trong bản PowerShell, xem `docs/viec-con-lai.md`.
//!
//! Tách riêng khỏi [`crate::act`] là có chủ ý: khe hở giữa lúc quét và lúc xóa
//! không dựng lại được bằng phép thử đầu-cuối, nên chốt phải gọi thẳng được.
//! Và phải có phép thử canh cả chỗ **nối dây**, không chỉ canh hàm. Mốc **M1**.
