#Requires -Version 5.1
<#
    Cổng liên thông của mốc M4.

    ① Bản sao lưu do bản RUST tạo phải khôi phục được bằng bản POWERSHELL,
       và ngược lại.
    ② So SHA-256 TỪNG TỆP, không so số lượng. Số lượng khớp mà nội dung hỏng là
       đúng loại lỗi mà một bản sao lưu sinh ra để chống.
    ③ Nhật ký hai bản khớp nhau về số dòng và trạng thái.

    Vì sao cổng này tồn tại tách khỏi cổng đối chiếu song song: bộ test đầu-cuối
    chạy mỗi bản một mình. Nó chứng minh được hai bản làm ĐÚNG, nhưng không
    chứng minh được chúng ĐỌC ĐƯỢC CỦA NHAU. Hai định dạng bản sao lưu lệch nhau
    thì cả hai vẫn xanh, cho tới ngày ai đó cần khôi phục bằng bản còn lại.

    ------------------------------------------------------------------
    KHÔNG BAO GIỜ chạy trên dữ liệu Zalo thật. Mọi thứ ở đây dựng trong %TEMP%.
    ------------------------------------------------------------------
#>
param([switch]$BoQuaDung)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }
$OutputEncoding = [Text.Encoding]::UTF8

$goc    = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$psTool = Join-Path $goc 'ZaloCleanup.ps1'
$rsTool = Join-Path $goc 'zalo-cli.exe'
$setFile = Join-Path $goc 'settings.json'
$logDir  = Join-Path $goc 'logs'

$script:Pass = 0; $script:Fail = 0
function Assert($ten, $dieu_kien, $chi_tiet) {
    if ($dieu_kien) { $script:Pass++; Write-Host ("  [ĐẠT ] $ten") -ForegroundColor Green }
    else { $script:Fail++; Write-Host ("  [HỎNG] $ten") -ForegroundColor Red
           if ($chi_tiet) { Write-Host "         $chi_tiet" -ForegroundColor DarkRed } }
}

function Get-Bam($p) {
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        $fs = [IO.File]::OpenRead($p)
        try { return [BitConverter]::ToString($sha.ComputeHash($fs)).Replace('-', '') }
        finally { $fs.Dispose() }
    } finally { $sha.Dispose() }
}

# Bản đồ đường-dẫn-tương-đối → SHA-256 của mọi tệp dưới một gốc.
function Get-BanDoBam($goc) {
    $m = @{}
    if (-not (Test-Path -LiteralPath $goc)) { return $m }
    foreach ($f in Get-ChildItem -LiteralPath $goc -Recurse -File -Force -EA SilentlyContinue) {
        if ($f.Name -eq '_zalocleanup_backup.json') { continue }
        $rel = $f.FullName.Substring($goc.Length).TrimStart('\')
        $m[$rel] = Get-Bam $f.FullName
    }
    return $m
}

function Invoke-CongCu($duongDan, $root, $keys) {
    $s = ($keys -join "`r`n") + "`r`n"
    if ($duongDan -like '*.exe') {
        return ($s | & $duongDan -Root $root 2>&1 | Out-String -Width 300)
    }
    return ($s | powershell.exe -NoProfile -ExecutionPolicy Bypass -File $duongDan -Root $root 2>&1 |
            Out-String -Width 300)
}

function New-Cay($goc) {
    # Cố ý trộn nhiều dạng: thư mục lồng nhau, tên có dấu, tệp ngay ở gốc, tệp
    # rỗng, và một tệp lớn hơn ngưỡng 128 KB của chữ ký nhanh.
    $rnd = New-Object Random 20260802
    $ds = @(
        @{ P = 'video\v1';                 N = 300000 }
        @{ P = 'video\v2';                 N = 1000 }
        @{ P = 'picture\anh có dấu.jxl';   N = 50000 }
        @{ P = 'picture\c1\sâu\hơn.png';   N = 200 }
        @{ P = 'file\rong';                N = 0 }
        @{ P = 'ngay_o_goc.bin';           N = 131073 }
    )
    foreach ($x in $ds) {
        $p = Join-Path $goc $x.P
        New-Item -ItemType Directory -Force (Split-Path $p -Parent) | Out-Null
        $b = New-Object byte[] $x.N
        if ($x.N -gt 0) { $rnd.NextBytes($b) }
        [IO.File]::WriteAllBytes($p, $b)
        (Get-Item -LiteralPath $p).LastWriteTime = [datetime]'2024-06-15'
    }
    return $ds.Count
}

# Sao lưu rồi xóa sạch nguồn. Trả về thư mục bản sao lưu vừa tạo.
function Invoke-SaoLuuRoiXoa($congCu, $root, $khoLuu) {
    $truoc = @(Get-ChildItem -LiteralPath $khoLuu -Directory -EA SilentlyContinue | Select-Object -Expand FullName)
    # 9 nâng cao · 7 quét · Enter · 9 sao lưu · <đích> · 2 xác minh toàn bộ ·
    # Enter · X xóa · XÓA · Enter · Enter · 0 thoát
    $o = Invoke-CongCu $congCu $root @('9', '7', '', '9', $khoLuu, '2', '', 'X', 'XÓA', '', '', '0')
    $sau = @(Get-ChildItem -LiteralPath $khoLuu -Directory -EA SilentlyContinue | Select-Object -Expand FullName)
    $moi = @($sau | Where-Object { $truoc -notcontains $_ })
    return [pscustomobject]@{ Out = $o; ThuMuc = $(if ($moi.Count -eq 1) { $moi[0] } else { $null }) }
}

function Invoke-KhoiPhuc($congCu, $root) {
    # 3 khôi phục · 1 chọn bản đầu · Enter giữ mặc định bỏ qua · Enter · 0 thoát
    return Invoke-CongCu $congCu $root @('3', '1', '', '', '0')
}

# ---------------------------------------------------------------- dựng
if (-not $BoQuaDung) {
    Write-Host 'Đang dựng zalo-cli.exe...' -ForegroundColor DarkGray
    Push-Location (Join-Path $goc 'rust')
    try {
        & cargo build --release -p zalo-cli | Out-Null
        if ($LASTEXITCODE -ne 0) { throw 'cargo build hỏng' }
    } finally { Pop-Location }
    Copy-Item (Join-Path $goc 'rust\target\release\zalo-cli.exe') $rsTool -Force
}
foreach ($t in @($psTool, $rsTool)) {
    if (-not (Test-Path -LiteralPath $t)) { throw "Không thấy công cụ: $t" }
}

# settings.json là tệp THẬT cạnh công cụ. Cất đi rồi trả lại nguyên trạng.
$setBak = $null
# KHÔNG có dấu phẩy đầu dòng ở đây. Dấu phẩy chỉ cần khi TRẢ VỀ mảng byte từ
# một hàm, để PowerShell khỏi rải nó ra thành từng phần tử. Ở phép gán thẳng
# thì dấu phẩy lại BỌC THÊM một lớp, và WriteAllBytes ném lỗi lúc trả tệp về —
# tức đúng lúc không được phép hỏng. Đã dính thật một lần.
if (Test-Path -LiteralPath $setFile) { $setBak = [IO.File]::ReadAllBytes($setFile) }
$sb = Join-Path $env:TEMP ('zlt_' + [Guid]::NewGuid().ToString('N').Substring(0, 8))
$batDau = Get-Date

try {
    New-Item -ItemType Directory -Force $sb | Out-Null

    foreach ($chieu in @(
        @{ Ten = 'Rust sao lưu → PowerShell khôi phục'; Luu = $rsTool; Phuc = $psTool; Ma = 'rs2ps' }
        @{ Ten = 'PowerShell sao lưu → Rust khôi phục'; Luu = $psTool; Phuc = $rsTool; Ma = 'ps2rs' }
    )) {
        Write-Host ''
        Write-Host ('── ' + $chieu.Ten) -ForegroundColor Yellow

        $root = Join-Path $sb ($chieu.Ma + '\ZaloDownloads')
        New-Item -ItemType Directory -Force $root | Out-Null
        $soTep = New-Cay $root
        $banGoc = Get-BanDoBam $root
        Assert "$($chieu.Ma) · dựng được cây thử" ($banGoc.Count -eq $soTep) `
            "dựng $soTep tệp nhưng băm được $($banGoc.Count)"

        $kho = Join-Path $sb ($chieu.Ma + '_kho')
        New-Item -ItemType Directory -Force $kho | Out-Null
        # Trỏ settings vào kho để bên khôi phục tìm ra ngay, khỏi quét cả ổ đĩa.
        (@{ BackupPolicy = 'HOI'; BackupRoots = @($kho) } | ConvertTo-Json) |
            Set-Content -LiteralPath $setFile -Encoding UTF8

        $sl = Invoke-SaoLuuRoiXoa $chieu.Luu $root $kho
        Assert "$($chieu.Ma) · sao lưu tạo đúng một thư mục mới" ($null -ne $sl.ThuMuc) `
            'không xác định được thư mục bản sao lưu'
        if ($null -eq $sl.ThuMuc) { continue }

        Assert "$($chieu.Ma) · bản kê đọc được và ghi đúng số tệp" `
            (Test-Path -LiteralPath (Join-Path $sl.ThuMuc '_zalocleanup_backup.json')) 'thiếu bản kê'
        $meta = Get-Content -LiteralPath (Join-Path $sl.ThuMuc '_zalocleanup_backup.json') -Raw | ConvertFrom-Json
        Assert "$($chieu.Ma) · bản kê ghi đúng $soTep tệp" ($meta.Count -eq $soTep) `
            "bản kê ghi $($meta.Count)"

        $conLai = @(Get-ChildItem -LiteralPath $root -Recurse -File -Force -EA SilentlyContinue)
        Assert "$($chieu.Ma) · nguồn đã sạch sau khi xóa" ($conLai.Count -eq 0) `
            "còn $($conLai.Count) tệp"

        # ---- ① và ② : bên kia khôi phục, so SHA-256 từng tệp
        $oPhuc = Invoke-KhoiPhuc $chieu.Phuc $root
        $sauPhuc = Get-BanDoBam $root
        Assert "$($chieu.Ma) · khôi phục đủ số tệp" ($sauPhuc.Count -eq $banGoc.Count) `
            "khôi phục $($sauPhuc.Count)/$($banGoc.Count) tệp"

        $lech = @()
        foreach ($k in $banGoc.Keys) {
            if (-not $sauPhuc.ContainsKey($k)) { $lech += "$k (thiếu hẳn)"; continue }
            if ($sauPhuc[$k] -ne $banGoc[$k]) { $lech += "$k (SHA-256 khác)" }
        }
        Assert "$($chieu.Ma) · SHA-256 khớp TỪNG TỆP" ($lech.Count -eq 0) ($lech -join '; ')

        $meta | Out-Null; $oPhuc | Out-Null
    }

    # ---- ③ : nhật ký hai bản khớp về số dòng và trạng thái
    Write-Host ''
    Write-Host '── Nhật ký hai bản' -ForegroundColor Yellow
    $nk = @(Get-ChildItem -LiteralPath $logDir -Filter 'daxoa_*.log' -File -EA SilentlyContinue |
            Where-Object { $_.LastWriteTime -ge $batDau } | Sort-Object LastWriteTime)
    Assert 'sinh ra đúng hai nhật ký xóa' ($nk.Count -eq 2) "tìm thấy $($nk.Count)"
    if ($nk.Count -eq 2) {
        $a = Get-Content -LiteralPath $nk[0].FullName -Encoding UTF8
        $b = Get-Content -LiteralPath $nk[1].FullName -Encoding UTF8
        Assert 'hai nhật ký cùng số dòng' ($a.Count -eq $b.Count) "$($a.Count) so với $($b.Count)"

        $dem = {
            param($lines)
            $h = @{}
            foreach ($l in $lines) {
                if ($l.StartsWith('#')) { continue }
                $tt = ($l -split "`t")[0]
                if ($h.ContainsKey($tt)) { $h[$tt]++ } else { $h[$tt] = 1 }
            }
            return $h
        }
        $da = & $dem $a; $db = & $dem $b
        $moiTrangThai = @($da.Keys) + @($db.Keys) | Sort-Object -Unique
        $khac = @()
        foreach ($t in $moiTrangThai) {
            $x = if ($da.ContainsKey($t)) { $da[$t] } else { 0 }
            $y = if ($db.ContainsKey($t)) { $db[$t] } else { 0 }
            if ($x -ne $y) { $khac += "$t : $x so với $y" }
        }
        Assert 'hai nhật ký cùng trạng thái và cùng số lượng mỗi loại' ($khac.Count -eq 0) ($khac -join '; ')

        foreach ($f in @($nk[0], $nk[1])) {
            $t = (Get-Content -LiteralPath $f.FullName -Encoding UTF8) -join "`n"
            Assert ("nhật ký $($f.Name) có dòng tổng kết hoàn tất=True") ($t -match 'hoàn tất=True') 'thiếu'
        }
    }
} finally {
    if ($null -ne $setBak) { [IO.File]::WriteAllBytes($setFile, $setBak) }
    elseif (Test-Path -LiteralPath $setFile) { Remove-Item -LiteralPath $setFile -Force }
    Get-ChildItem -LiteralPath $logDir -File -EA SilentlyContinue |
        Where-Object { $_.LastWriteTime -ge $batDau } | Remove-Item -Force -EA SilentlyContinue
    if (Test-Path -LiteralPath $sb) { Remove-Item -LiteralPath $sb -Recurse -Force -EA SilentlyContinue }
}

Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host ("  ĐẠT: {0}    HỎNG: {1}" -f $script:Pass, $script:Fail) `
    -ForegroundColor $(if ($script:Fail -eq 0) { 'Green' } else { 'Red' })
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
if ($script:Fail -gt 0) { Write-Host '  Cổng liên thông CHƯA ĐẠT' -ForegroundColor Red; exit 1 }
Write-Host '  Cổng liên thông ĐẠT' -ForegroundColor Green
exit 0
