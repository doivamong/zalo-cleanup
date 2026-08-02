//! Việc chạy nền — quét, sao lưu, xóa, khôi phục.
//!
//! # Vì sao không có gì tỉ lệ với N chạy trong luồng giao diện
//!
//! Ràng buộc `RB-129` của hội đồng: **không có gì tỉ lệ với N chạy trong luồng
//! giao diện**. Dữ liệu Zalo thật có 57.000 tệp; một vòng lặp qua chúng trong
//! luồng vẽ là cửa sổ đứng hình, và một cửa sổ đứng hình giữa lượt xóa là thứ
//! khiến người ta bấm bừa hoặc tắt máy.
//!
//! Mọi việc nặng chạy trong luồng riêng và báo về bằng kênh. Luồng giao diện
//! chỉ đọc trạng thái mới nhất rồi vẽ.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use zalo_core::act::{KetQuaKhoiPhuc, KetQuaSaoLuuChiTiet, KetQuaXoa, TepQuet};

/// Tin báo về từ luồng nền.
pub enum Tin {
    /// Đang làm, kèm câu mô tả cho người dùng đọc.
    DangLam(String),
    QuetXong {
        tep: Vec<TepQuet>,
        loai: String,
        goc: String,
        goc_don_dep: Vec<String>,
        bi_chan: usize,
        loi: usize,
    },
    XoaXong(Box<KetQuaXoa>, usize),
    SaoLuuXong(Box<KetQuaSaoLuuChiTiet>, String),
    KhoiPhucXong(Box<KetQuaKhoiPhuc>),
    Hong(String),
}

/// Tay cầm của một việc đang chạy nền.
pub struct ViecNen {
    pub nhan: Receiver<Tin>,
    /// Bật lên là luồng nền dừng ở vòng lặp gần nhất.
    pub co_huy: Arc<AtomicBool>,
    /// Số việc đã làm và tổng số, để vẽ thanh tiến độ.
    pub xong: Arc<AtomicUsize>,
    pub tong: Arc<AtomicUsize>,
    /// Câu mô tả gần nhất.
    pub mo_ta: String,
    pub dang_chay: bool,
}

impl ViecNen {
    /// Chạy `viec` trong một luồng riêng.
    ///
    /// `viec` nhận bộ gửi tin, cờ hủy và hai bộ đếm tiến độ.
    pub fn chay<F>(mo_ta_dau: &str, viec: F) -> Self
    where
        F: FnOnce(Sender<Tin>, Arc<AtomicBool>, Arc<AtomicUsize>, Arc<AtomicUsize>)
            + Send
            + 'static,
    {
        let (gui, nhan) = channel();
        let co_huy = Arc::new(AtomicBool::new(false));
        let xong = Arc::new(AtomicUsize::new(0));
        let tong = Arc::new(AtomicUsize::new(0));
        let (c, x, t) = (co_huy.clone(), xong.clone(), tong.clone());
        std::thread::spawn(move || viec(gui, c, x, t));
        ViecNen {
            nhan,
            co_huy,
            xong,
            tong,
            mo_ta: mo_ta_dau.to_string(),
            dang_chay: true,
        }
    }

    pub fn xin_huy(&self) {
        self.co_huy.store(true, Ordering::Relaxed);
    }

    pub fn da_xin_huy(&self) -> bool {
        self.co_huy.load(Ordering::Relaxed)
    }

    /// Phần trăm đã xong, `None` khi chưa biết tổng.
    pub fn phan_tram(&self) -> Option<f32> {
        let t = self.tong.load(Ordering::Relaxed);
        if t == 0 {
            return None;
        }
        Some(self.xong.load(Ordering::Relaxed) as f32 / t as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cờ hủy phải tới được luồng nền. Đây là đường sống của `BP-08`.
    #[test]
    fn co_huy_toi_duoc_luong_nen() {
        let v = ViecNen::chay("thử", |gui, co_huy, xong, tong| {
            tong.store(1_000_000, Ordering::Relaxed);
            for i in 0..1_000_000 {
                if co_huy.load(Ordering::Relaxed) {
                    let _ = gui.send(Tin::Hong("đã hủy".into()));
                    return;
                }
                xong.store(i, Ordering::Relaxed);
                std::thread::yield_now();
            }
            let _ = gui.send(Tin::DangLam("chạy hết".into()));
        });
        v.xin_huy();
        assert!(v.da_xin_huy());
        // Luồng nền phải kết thúc bằng nhánh hủy, không phải nhánh chạy hết.
        let mut thay_huy = false;
        for _ in 0..200 {
            if let Ok(t) = v.nhan.recv_timeout(std::time::Duration::from_millis(50)) {
                match t {
                    Tin::Hong(m) => {
                        assert_eq!(m, "đã hủy");
                        thay_huy = true;
                        break;
                    }
                    Tin::DangLam(m) => panic!("luồng nền chạy hết dù đã xin hủy: {m}"),
                    _ => {}
                }
            }
        }
        assert!(thay_huy, "không nhận được tin hủy từ luồng nền");
    }

    #[test]
    fn phan_tram_chua_biet_tong_thi_tra_none() {
        let v = ViecNen::chay("thử", |_, _, _, _| {});
        assert_eq!(v.phan_tram(), None);
        v.tong.store(4, Ordering::Relaxed);
        v.xong.store(1, Ordering::Relaxed);
        assert_eq!(v.phan_tram(), Some(0.25));
    }
}
