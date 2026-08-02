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
$canNguoiThat = @(
    @{ Ma = '§8.1-2'; Viec = 'Ảnh chụp greyscale, 3 người thử phân loại mức rủi ro → 33/33 đúng'; ChuY = 'MAU-01' }
    @{ Ma = '§8.1-3'; Viec = 'Gõ XOÁ bằng Unikey kiểu đặt dấu mới → nút Xóa bật'; ChuY = 'Cần bộ gõ thật; phần so chuỗi đã có phép thử' }
    @{ Ma = 'BP-01';  Viec = 'Rút chuột, chạy trọn kịch bản chỉ bằng bàn phím'; ChuY = '' }
    @{ Ma = 'BP-04';  Viec = 'Hộp thoại giam tiêu điểm — nhấn Tab 30 lần, tiêu điểm không thoát ra ngoài'; ChuY = '' }
    @{ Ma = 'DPI-04'; Viec = 'Vừa màn 1366×768 @125%, không cuộn ngang ở mọi màn hình'; ChuY = 'Cỡ cửa sổ đã ràng buộc trong mã' }
    @{ Ma = 'DPI-08'; Viec = 'Trang xác nhận mở canh giữa cửa sổ cha, cùng màn hình'; ChuY = '' }
    @{ Ma = 'MAU-01'; Viec = 'Bỏ hết màu vẫn hiểu — 3 người thử, 33/33'; ChuY = 'Chữ và ký hiệu đã có phép thử; phần người đọc thì chưa' }
    @{ Ma = 'MAU-09'; Viec = 'Ảnh greyscale, người thử chỉ đúng nút Hủy'; ChuY = '' }
    @{ Ma = 'ĐM-08';  Viec = 'Bật NVDA thật → dải thông báo hiện ra và nút mở bản dòng lệnh chạy'
       ChuY = 'ĐÃ CÀI; phần dò và phần mở đã có phép thử, phần "bật NVDA thật" thì chưa' }
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
Write-Host '── §8.1-1 — cần một phiên màn hình, không chạy được ở đây' -ForegroundColor Yellow
Write-Host '   Đã tự động hóa và ĐÃ ĐẠT 8/8 trên hộp cát, nhưng nó lái chuột và bàn' -ForegroundColor DarkGray
Write-Host '   phím thật nên không chạy được trên máy chủ CI hay trong lượt này.' -ForegroundColor DarkGray
Write-Host '   Chạy tay:  powershell -File rust\tools\phep-thu-ma-sat.ps1' -ForegroundColor DarkGray

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
