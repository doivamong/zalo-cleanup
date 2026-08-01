#Requires -Version 5.1
<#
    CỔNG KIẾN TRÚC M0 — chạy sau mỗi commit, cả trên máy lẫn trên CI.

    Kiểm hai điều, cả hai đều đo được chứ không phải đánh giá cảm tính:

      1. Lõi KHÔNG phụ thuộc giao diện.
         Đây là ranh giới cốt lõi của cả kế hoạch port. Lõi quyết định xóa gì
         phải soát được ở quy mô nhỏ; giao diện muốn kéo bao nhiêu crate cũng
         được, miễn không dính vào phần đụng dữ liệu người dùng.

      2. Lõi không vượt trần số crate.
         Tiêu chí dừng D-4 trong docs/ke-hoach-port.md. Đo lúc lập kế hoạch:
         lõi 36 crate, cộng eframe thành 112 — tức 76 crate chỉ để vẽ cửa sổ.

    Chạy tay:  powershell -NoProfile -ExecutionPolicy Bypass -File rust\tools\check-deps.ps1
#>
param(
    [int]$TranCrate = 60
)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Leaf
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    $raw = & cargo tree -p zalo-core --edges normal --prefix none 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) {
        Write-Host 'cargo tree thất bại:' -ForegroundColor Red
        Write-Host $raw
        exit 1
    }
} finally { Pop-Location }

# Mỗi dòng có dạng "<tên> v<phiên bản> ...". Lấy tên, bỏ trùng.
$pkgs = @($raw -split "`r?`n" |
          Where-Object { $_ -match '^([A-Za-z0-9_\-]+)\s+v[0-9]' } |
          ForEach-Object { ($_ -split '\s+')[0] } |
          Sort-Object -Unique)

$cam = @('eframe', 'egui', 'egui-winit', 'egui_glow', 'winit', 'accesskit', 'wgpu', 'glutin')
$dinh = @($pkgs | Where-Object { $cam -contains $_ })

Write-Host ''
Write-Host '── Cổng kiến trúc M0' -ForegroundColor Cyan
Write-Host ("  Lõi zalo-core kéo theo : {0} crate" -f $pkgs.Count)

$hong = $false

if ($dinh.Count -gt 0) {
    Write-Host ''
    Write-Host '  HỎNG: lõi đã dính phụ thuộc giao diện' -ForegroundColor Red
    $dinh | ForEach-Object { Write-Host ("    - " + $_) -ForegroundColor Red }
    Write-Host ''
    Write-Host '  zalo-core phải kiểm thử được mà không có một dòng giao diện nào' -ForegroundColor Red
    Write-Host '  trong cây phụ thuộc. Chuyển phần vừa thêm sang zalo-gui.' -ForegroundColor Red
    $hong = $true
} else {
    Write-Host '  Không dính phụ thuộc giao diện           OK' -ForegroundColor Green
}

if ($pkgs.Count -gt $TranCrate) {
    Write-Host ''
    Write-Host ("  HỎNG: vượt trần {0} crate — chạm tiêu chí dừng D-4" -f $TranCrate) -ForegroundColor Red
    Write-Host '  Lý lẽ kiến trúc của kế hoạch là lõi soát được ở quy mô nhỏ.' -ForegroundColor Red
    Write-Host '  Vượt trần là mất chính lý lẽ đó. Dừng lại và báo cáo.' -ForegroundColor Red
    $hong = $true
} else {
    Write-Host ("  Dưới trần {0} crate                      OK" -f $TranCrate) -ForegroundColor Green
}

Write-Host ''
if ($hong) { exit 1 }
Write-Host '  Cổng M0 ĐẠT' -ForegroundColor Green
Write-Host ''
exit 0
