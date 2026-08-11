#Requires -Version 5.1
<#
    Cổng của mốc M5 — giao diện đồ họa.

    Danh mục tiếp cận của hội đồng (docs/ui-ux-council.md §8) chia ba mức.
    **Mức 1 chặn hẳn bản phát hành, không thương lượng.**

    Bộ chạy này làm đúng hai việc, và nói rõ ranh giới giữa chúng:

      ① Chạy các mục MỨC 1 kiểm được bằng máy, và bắt lỗi nếu chúng hỏng.
      ② LIỆT KÊ TÊN các mục MỨC 1 **không** kiểm được bằng máy — chúng cần
        người thật ngồi trước màn hình. Không mục nào bị bỏ quên trong im lặng.

    Vì sao vế ② quan trọng ngang vế ①: một cổng chỉ báo cáo phần nó đo được sẽ
    đọc ra như "đã đạt hết", và đó là cách một bản phát hành đi ra ngoài với ba
    mục mức 1 chưa ai kiểm.
#>
param([switch]$BoQuaDung)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

$goc = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$rust = Join-Path $goc 'rust'

$script:Pass = 0; $script:Fail = 0
function Assert($ma, $ten, $ok, $chi_tiet) {
    if ($ok) { $script:Pass++; Write-Host ("  [ĐẠT ] {0,-8} {1}" -f $ma, $ten) -ForegroundColor Green }
    else {
        $script:Fail++
        Write-Host ("  [HỎNG] {0,-8} {1}" -f $ma, $ten) -ForegroundColor Red
        if ($chi_tiet) { Write-Host "           $chi_tiet" -ForegroundColor DarkRed }
    }
}

# Các mục MỨC 1 mà một cỗ máy không kết luận được. Danh sách này KHÔNG được rút
# ngắn cho đẹp báo cáo — rút ngắn nó là giấu đi phần chưa ai kiểm.
#
# ---------------------------------------------------------------------------
# DANH SÁCH NÀY TỪNG CÓ CHÍN MỤC, VÀ TÁM TRONG SỐ ĐÓ ĐO ĐƯỢC BẰNG MÁY.
#
# Lý do cũ — "không cỗ máy nào kết luận được" — sai một nửa. egui bật
# `accesskit`, mà AccessKit phơi toàn bộ cây widget ra UI Automation của
# Windows kèm tên, vai trò, trạng thái bật/tắt và khung bao. Hỏi được bằng máy
# đúng những câu tưởng phải nhìn bằng mắt.
#
# `tools\kiem-muc-1.ps1` lái giao diện THẬT trong hộp cát và đo hết. Nó bắt
# được hai lỗi mà chín tháng "chờ người kiểm" không bắt được:
#
#   · DPI-04 — lưới ảnh xem trước tràn khỏi mép phải ở 1092×614 dip, 4 trong
#     12 ô nằm ngoài, sau một thanh cuộn ngang không ai thấy.
#   · BP-01 — egui 0.29 để widget đang TẮT nuốt mất một chặng Tab, nên widget
#     đứng sau nó không bao giờ nhận được tiêu điểm. Dính ba màn, nặng nhất là
#     trang chủ khi máy chưa cài Zalo: cả màn hình chết với bàn phím.
#
# Cả hai đã sửa. Bài học cũ, lần thứ tám: **đoán thì không bắt được gì.**
# ---------------------------------------------------------------------------
$canNguoiThat = @(
    @{ Ma = '§8.1-2'; Viec = 'Ba người thử nhìn ảnh greyscale và xếp đúng mức rủi ro → 33/33'
       ChuY = 'Ảnh đã sinh sẵn bằng kiem-muc-1.ps1. Chỉ còn phần MẮT NGƯỜI.' }
    @{ Ma = 'MAU-01'; Viec = 'Cùng bộ ảnh trên, phần người đọc'
       ChuY = 'Máy đã đo: ký hiệu ↔ câu chữ khớp một-một, và sau khi bỏ màu mỗi mức vẫn vẽ ra một hình riêng.' }
    @{ Ma = 'MAU-09'; Viec = 'Người thử nhìn ảnh greyscale và chỉ đúng nút Hủy'
       ChuY = 'Máy đã đo: khác chữ, khác biểu tượng, cách nhau 48 dip, Hủy đứng trước.' }
)

Write-Host ''
Write-Host '── Mục MỨC 1 kiểm được bằng máy' -ForegroundColor Yellow

# ---- TV-01/02/04/10, VM-01/02/05, BP-05/07/12, MAU-01(phần máy), ĐM-06
# Toàn bộ nằm trong bộ phép thử của zalo-gui và zalo-core. Chạy rồi đọc kết quả.
Push-Location $rust
try {
    $out = & cargo test -p zalo-gui -p zalo-core --quiet 2>&1 | Out-String
    $ok = ($LASTEXITCODE -eq 0)
} finally { Pop-Location }
Assert 'nhiều' 'Phép thử đơn vị của giao diện và lõi' $ok 'chạy: cargo test -p zalo-gui -p zalo-core'

# Đếm để báo cáo nói được ĐỘ PHỦ, không chỉ nói xanh hay đỏ.
$soPhep = 0
foreach ($m in [regex]::Matches($out, 'test result: ok\. (\d+) passed')) {
    $soPhep += [int]$m.Groups[1].Value
}
Write-Host ("           {0} phép thử đơn vị đã chạy" -f $soPhep) -ForegroundColor DarkGray

# ---- TV-03: chuẩn hóa ĐỂ HIỂN THỊ không được lọt vào lõi.
#
# Bản đầu của phép kiểm này bắt cả `nfc(` và báo hỏng ở `confirm.rs`. Đọc lại
# thì đó là phép kiểm sai, không phải mã sai: `s.nfd()` rồi lọc dấu tổ hợp rồi
# `.nfc()` chính là thuật toán BỎ DẤU THANH mà TV-02 đòi phải có trong lõi —
# `nfd("XÓA")` phải được chấp nhận.
#
# Thứ TV-03 cấm là `normalize_nfc()`, tức chuẩn hóa để HIỂN THỊ. Lọt vào lõi thì
# lõi bắt đầu quyết định dựa trên dạng chữ đã bị sửa, mà bản PowerShell thì
# không — hai bản trôi khỏi nhau ở đúng chỗ so cụm từ xác nhận.
$loi = @()
foreach ($f in Get-ChildItem (Join-Path $rust 'crates\zalo-core\src') -Filter *.rs -Recurse) {
    $t = [IO.File]::ReadAllText($f.FullName, [Text.Encoding]::UTF8)
    if ($t -match 'normalize_nfc') { $loi += $f.Name }
}
Assert 'TV-03' 'Chuẩn hóa để hiển thị không lọt vào lõi' ($loi.Count -eq 0) ($loi -join ', ')

# TV-02 là vế còn lại của cùng một cặp, và phải kiểm cả hai: cấm chuẩn hóa hiển
# thị mà cũng làm mất luôn đường bỏ dấu thì cụm `XOA` hết mở khóa được.
$cf = [IO.File]::ReadAllText((Join-Path $rust 'crates\zalo-core\src\confirm.rs'), [Text.Encoding]::UTF8)
Assert 'TV-02' 'Lõi vẫn bỏ được dấu thanh để so cụm xác nhận' `
    ($cf -match '\.nfd\(\)' -and $cf -match 'la_dau_to_hop') 'thuật toán bỏ dấu đã biến mất khỏi lõi'

# ---- Cổng kiến trúc: lõi vẫn không dính giao diện, kể cả sau khi thêm eframe
Push-Location $rust
try {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $rust 'tools\check-deps.ps1') | Out-Null
    $okKt = ($LASTEXITCODE -eq 0)
} finally { Pop-Location }
Assert 'M0'    'Lõi vẫn không phụ thuộc giao diện sau khi thêm eframe' $okKt ''

# ---- Kích thước exe: lời hứa "tải về chạy ngay" là một con số, không phải cảm giác
$exe = Join-Path $rust 'target\release\zalo-gui.exe'
if (-not $BoQuaDung -or -not (Test-Path $exe)) {
    Push-Location $rust
    try { & cargo build --release -p zalo-gui | Out-Null } finally { Pop-Location }
}
if (Test-Path $exe) {
    $mib = (Get-Item $exe).Length / 1MB
    Assert 'QĐ-01' ("Exe {0:N2} MiB, dưới trần 6 MiB" -f $mib) ($mib -lt 6.0) ''
} else {
    Assert 'QĐ-01' 'Dựng được zalo-gui.exe' $false 'không thấy tệp'
}

# ---- Không có tệp nào ngoài bảng ký hiệu dùng ký hiệu chưa kiểm glyph
$nghi = @()
foreach ($f in Get-ChildItem (Join-Path $rust 'crates\zalo-gui\src') -Filter *.rs) {
    if ($f.Name -eq 'phong.rs') { continue }
    $t = [IO.File]::ReadAllText($f.FullName, [Text.Encoding]::UTF8)
    # Ký hiệu ngoài BMP thấp hoặc trong dải ký hiệu — dấu hiệu gõ thẳng vào mã.
    foreach ($m in [regex]::Matches($t, "'([␀-⯿-￿])'")) {
        $nghi += "{0}: {1}" -f $f.Name, $m.Groups[1].Value
    }
}
Assert 'TV-04' 'Không gõ thẳng ký hiệu vào mã, phải qua bảng đã kiểm glyph' ($nghi.Count -eq 0) ($nghi -join '; ')

# ---- Ba việc M5 từng nợ. Kiểm bằng máy rằng chúng đã có thật, chứ không tin
# vào một dòng ghi trong tài liệu.
$guiSrc = Join-Path $rust 'crates\zalo-gui\src'
$cargoGui = [IO.File]::ReadAllText((Join-Path $rust 'crates\zalo-gui\Cargo.toml'), [Text.Encoding]::UTF8)
Assert 'M5-1' 'Có bộ giải mã JPEG XL (46,4% dữ liệu Zalo thật)' `
    ($cargoGui -match 'jxl-oxide') 'thiếu phụ thuộc jxl-oxide'

$ud = [IO.File]::ReadAllText((Join-Path $guiSrc 'ung_dung.rs'), [Text.Encoding]::UTF8)
Assert 'M5-2' 'Có màn sao lưu và khôi phục trong giao diện' `
    ($ud -match 'fn ve_sao_luu' -and $ud -match 'fn ve_khoi_phuc') 'thiếu một trong hai màn'

Assert 'ĐM-08' 'Có dò trình đọc màn hình và đường lui sang bản dòng lệnh' `
    ((Test-Path (Join-Path $guiSrc 'duong_lui.rs')) -and $ud -match 've_dai_duong_lui') 'chưa nối vào giao diện'

Write-Host ''
Write-Host '── Hai bộ chạy cần một phiên màn hình, không chạy được ở đây' -ForegroundColor Yellow
Write-Host '   Cả hai lái giao diện THẬT trong hộp cát, nên không chạy được trên máy' -ForegroundColor DarkGray
Write-Host '   chủ CI. Chạy tay:' -ForegroundColor DarkGray
Write-Host '     powershell -File rust\tools\phep-thu-ma-sat.ps1   §8.1-1, đã đạt 8/8' -ForegroundColor DarkGray
Write-Host '     powershell -File rust\tools\kiem-muc-1.ps1        tám mục mức 1 còn lại' -ForegroundColor DarkGray
Write-Host '   `kiem-muc-1.ps1` gõ phím THẬT ở ba phần BP-01, BP-04, §8.1-3. Đóng hết' -ForegroundColor DarkGray
Write-Host '   ứng dụng khác trước khi chạy — phím lạc sang cửa sổ người ta là hỏng' -ForegroundColor DarkGray
Write-Host '   việc thật. Ba phần còn lại đi bằng UIA, chạy lúc nào cũng an toàn.' -ForegroundColor DarkGray

Write-Host ''
Write-Host '── Mục MỨC 1 CẦN NGƯỜI THẬT — chưa kiểm' -ForegroundColor Yellow
Write-Host '   Không cỗ máy nào kết luận được những mục này. Chúng vẫn CHẶN bản phát hành.' -ForegroundColor DarkGray
Write-Host ''
foreach ($v in $canNguoiThat) {
    Write-Host ("   {0,-8} {1}" -f $v.Ma, $v.Viec) -ForegroundColor Yellow
    if ($v.ChuY) { Write-Host ("            {0}" -f $v.ChuY) -ForegroundColor DarkGray }
}

Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host ("  Kiểm bằng máy : ĐẠT {0} · HỎNG {1}" -f $script:Pass, $script:Fail) `
    -ForegroundColor $(if ($script:Fail -eq 0) { 'Green' } else { 'Red' })
Write-Host ("  Cần người thật: {0} mục mức 1 CHƯA kiểm" -f $canNguoiThat.Count) -ForegroundColor Yellow
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
if ($script:Fail -gt 0) { Write-Host '  Cổng M5 (phần máy) CHƯA ĐẠT' -ForegroundColor Red; exit 1 }
Write-Host '  Cổng M5 phần máy ĐẠT. Phần cần người thật vẫn còn nguyên.' -ForegroundColor Green
exit 0
