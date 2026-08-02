#Requires -Version 5.1
<#
    Dựng bản phát hành sao cho **TÁI LẬP ĐƯỢC** — cổng của mốc M6.

    "Build tái lập được từ CI công khai, mã băm khớp bản tải về."

    Cổng ấy không phải chuyện hình thức. Người dùng mất khả năng đọc mã nguồn
    khi ta chuyển từ `.ps1` sang `.exe`; thứ thay thế duy nhất là họ tự dựng lại
    và thấy mã băm khớp. Băm không khớp thì lời hứa ấy rỗng.

    ------------------------------------------------------------------
    HAI THỨ PHÁ TÍNH TÁI LẬP, cả hai đều đo được chứ không phải suy đoán

    ① DẤU THỜI GIAN TRONG ĐẦU TỆP PE.
       Đo: hai lần dựng sạch trên CÙNG máy, CÙNG đường dẫn → hai tệp khác nhau.
       `link.exe` của MSVC nhét thời điểm build vào header. Cờ `/Brepro` bảo nó
       thay bằng một giá trị băm từ nội dung.

    ② ĐƯỜNG DẪN TUYỆT ĐỐI CỦA THƯ MỤC BUILD.
       Đo: dựng cùng mã nguồn ở hai đường dẫn khác nhau → `zalo-cli.exe` khớp,
       `zalo-gui.exe` KHÔNG. Bới chuỗi trong tệp nhị phân thì thấy:

         D:\<gốc>\rust\target\release\build\glutin_egl_sys-*/out/egl_bindings.rs

       Build script của `glutin_egl_sys` và `glutin_wgl_sys` sinh mã vào
       `OUT_DIR`, và đường dẫn tuyệt đối của tệp sinh ra lọt vào chuỗi thông báo
       lỗi. `--remap-path-prefix` đổi mọi tiền tố ấy về một tên cố định.

    Cả hai cờ phải đi CÙNG NHAU trong một biến `RUSTFLAGS`: đặt biến này là ghi
    đè hẳn `rustflags` trong `.cargo/config.toml`, nên thiếu một cờ là mất cờ đó.
    ------------------------------------------------------------------
#>
param(
    [switch]$ChiIn   # chỉ in ra cờ rồi thoát, không dựng
)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

$rust = Split-Path $PSScriptRoot -Parent
$goc = (Resolve-Path $rust).Path.TrimEnd('\')
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
$cargoHome = $cargoHome.TrimEnd('\')

# Thứ tự cờ không quan trọng, nhưng NỘI DUNG thì có: hai máy phải sinh ra cùng
# một chuỗi cờ sau khi thay thế, nếu không đầu ra vẫn khác nhau.
# ③ TRÌNH LIÊN KẾT KHÁC BẢN.
#    Đo: hai lượt CI liên tiếp trên cùng mã nguồn ra hai tệp khác nhau, vì ảnh
#    máy chủ đổi giữa hai lượt — Visual Studio 18.8.12023.21 rồi 18.7.11925.98.
#    Tức CI còn không tái lập được với chính nó.
#
#    `link.exe` của MSVC không ghim được từ repo. `rust-lld` thì có: nó đi kèm
#    đúng bộ toolchain mà `rust-toolchain.toml` đã ghim, nên ghim rustc là ghim
#    luôn trình liên kết.
$sysroot = (& rustc --print sysroot).Trim()
$lld = Join-Path $sysroot 'lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe'
if (-not (Test-Path $lld)) { throw "Không thấy rust-lld ở $lld" }

$co = @(
    '-Clink-arg=/Brepro'
    '-Clinker-flavor=lld-link'
    "-Clinker=$lld"
    "--remap-path-prefix=$goc=/z"
    "--remap-path-prefix=$cargoHome=/c"
) -join ' '

Write-Host "Gốc mã nguồn : $goc" -ForegroundColor DarkGray
Write-Host "CARGO_HOME   : $cargoHome" -ForegroundColor DarkGray
Write-Host "rustc        : $((& rustc --version).Trim())" -ForegroundColor DarkGray
Write-Host "Trình liên kết: rust-lld đi kèm toolchain" -ForegroundColor DarkGray
Write-Host "RUSTFLAGS    : $co" -ForegroundColor DarkGray
if ($ChiIn) { exit 0 }

$env:RUSTFLAGS = $co
Push-Location $rust
try {
    & cargo build --release -p zalo-cli -p zalo-gui
    if ($LASTEXITCODE -ne 0) { throw 'cargo build hỏng' }
} finally {
    Pop-Location
    Remove-Item Env:\RUSTFLAGS -ErrorAction SilentlyContinue
}

Write-Host ''
Write-Host 'SHA-256 của bản phát hành:' -ForegroundColor Cyan
foreach ($t in 'zalo-cli.exe', 'zalo-gui.exe') {
    $p = Join-Path $rust "target\release\$t"
    $h = (Get-FileHash $p -Algorithm SHA256).Hash
    Write-Host ("  {0,-14} {1}  ({2:N0} byte)" -f $t, $h, (Get-Item $p).Length)
}
