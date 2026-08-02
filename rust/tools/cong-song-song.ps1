#Requires -Version 5.1
<#
    Cổng đối chiếu song song — mốc M3 và M4.

    Chạy ĐÚNG MỘT bộ test — ZaloCleanup.Tests.ps1 — hai lần: một lần lái bản
    PowerShell, một lần lái zalo-cli.exe. Không sửa một ký tự nào trong các phép
    thử; chỉ đổi công cụ được lái qua biến môi trường ZALO_TOOL.

    Vì sao không chép bộ test ra làm bản thứ hai: chép là hai bản test trôi khỏi
    nhau, và lúc đó "cả hai đều xanh" chẳng chứng minh được gì về hai công cụ.

    ------------------------------------------------------------------
    LỊCH SỬ PHẠM VI, và một lỗi đếm của chính tệp này

    Ở mốc M3, bản Rust chưa biết xóa nên cổng chỉ đòi các phép thử đầu-cuối
    KHÔNG xóa tệp — 28 phép. Phần còn lại được nêu tên tường minh là chờ M4 chứ
    không giấu.

    Từ mốc M4, bản Rust xóa · sao lưu · khôi phục được, nên cổng đòi TOÀN BỘ.

    Bản đầu của tệp này đếm phép thử bằng danh sách có lặp tên trong khi tra kết
    quả bằng bảng băm khử trùng, nên báo "39 phép, xanh 35" và làm người đọc
    tưởng có 4 phép không chạy tới. Thật ra không thiếu phép nào — chỉ là hai vế
    đếm theo hai cách khác nhau. Giờ khử trùng tên ở cả hai vế.
    ------------------------------------------------------------------
#>
param(
    [switch]$BoQuaDung   # bỏ qua bước dựng lại exe, dùng bản đã có
)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }
$OutputEncoding = [Text.Encoding]::UTF8

$goc     = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$boTest  = Join-Path $goc 'ZaloCleanup.Tests.ps1'
$exeDich = Join-Path $goc 'zalo-cli.exe'

# Phân loại phép thử: phép nào LÁI CÔNG CỤ, phép nào chỉ soi mã nguồn PowerShell.
#
# Phép soi mã nguồn vẫn phải chạy — chúng canh bản PowerShell — nhưng chúng
# KHÔNG nói gì về bản Rust, nên đừng đếm chúng vào bằng chứng. Gộp chung là tự
# thổi phồng: 135 phép thử luôn xanh bất kể bản Rust đúng hay sai.
function Get-PhanLoai {
    $tok = $null; $err = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile($boTest, [ref]$tok, [ref]$err)
    if ($err.Count) { throw "không phân tích được bộ test: $($err[0].Message)" }

    $nguon = @()
    foreach ($a in $ast.FindAll({ param($n) $n -is [Management.Automation.Language.AssignmentStatementAst] }, $true)) {
        $goi = $a.Right.FindAll({ param($n)
            $n -is [Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'Invoke-Tool' }, $true)
        if ($goi.Count -eq 0) { continue }
        $bien = @($a.Left.Extent.Text.TrimStart('$'))
        foreach ($v in $goi[0].FindAll({ param($n)
            $n -is [Management.Automation.Language.VariableExpressionAst] }, $true)) {
            $bien += $v.VariablePath.UserPath
        }
        $nguon += [pscustomobject]@{ Dong = $a.Extent.StartLineNumber; Bien = ($bien | Sort-Object -Unique) }
    }
    if ($nguon.Count -eq 0) { throw 'Không thấy lời gọi Invoke-Tool nào — bộ test đã đổi hình dạng.' }

    $e2e = New-Object 'Collections.Generic.HashSet[string]'
    $tong = 0
    foreach ($as in $ast.FindAll({ param($n)
        $n -is [Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'Assert' }, $true)) {
        $tong++
        $dong = $as.Extent.StartLineNumber
        $txt = $as.Extent.Text
        $co = $nguon | Where-Object {
            if ($_.Dong -ge $dong) { return $false }
            foreach ($b in $_.Bien) { if ($txt -match ('\$' + [regex]::Escape($b) + '\b')) { return $true } }
            return $false
        } | Select-Object -First 1
        if ($co) { [void]$e2e.Add($as.CommandElements[1].Extent.Text.Trim("'")) }
    }
    return [pscustomobject]@{ E2E = $e2e; TongAssert = $tong }
}

function Invoke-BoTest($duongDanCongCu) {
    if ($duongDanCongCu) { $env:ZALO_TOOL = $duongDanCongCu }
    else { Remove-Item Env:\ZALO_TOOL -ErrorAction SilentlyContinue }
    $cu = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    # -Full là BẮT BUỘC ở cổng này, không phải tùy chọn cho kỹ tính.
    #
    # Bốn phép thử chậm nằm trong khối đó là bốn đường nguy hiểm nhất của mốc M4:
    # tệp bị khóa làm sao lưu hỏng thì bước xóa phải bị chặn, và tệp biến mất
    # giữa lượt xóa thì không được đếm nhầm là đã xóa. Bỏ chúng đi là bỏ đúng
    # phần mà cả mốc M4 sinh ra để canh.
    $out = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $boTest -Full 2>&1 |
           Out-String -Width 300
    $ErrorActionPreference = $cu
    Remove-Item Env:\ZALO_TOOL -ErrorAction SilentlyContinue
    $kq = @{}
    foreach ($m in [regex]::Matches($out, '\[(ĐẠT|HỎNG)\s*\]\s*(.+?)\r?\n')) {
        $ten = $m.Groups[2].Value.Trim()
        $dat = ($m.Groups[1].Value -eq 'ĐẠT')
        # Tên trùng nhau thì lấy kết quả XẤU NHẤT. Một phép thử xanh không được
        # phép che một phép thử cùng tên đã đỏ.
        if ($kq.ContainsKey($ten)) { $kq[$ten] = ($kq[$ten] -and $dat) }
        else { $kq[$ten] = $dat }
    }
    return $kq
}

# ---------------------------------------------------------------- dựng
if (-not $BoQuaDung) {
    Write-Host 'Đang dựng zalo-cli.exe...' -ForegroundColor DarkGray
    Push-Location (Join-Path $goc 'rust')
    try {
        # KHÔNG dùng 2>&1 với chương trình ngoài: PowerShell 5.1 bọc từng dòng
        # stderr thành ErrorRecord, và với ErrorActionPreference = 'Stop' thì một
        # dòng tiến độ bình thường của cargo cũng làm cả script dừng.
        & cargo build --release -p zalo-cli | Out-Null
        if ($LASTEXITCODE -ne 0) { throw 'cargo build hỏng' }
    } finally { Pop-Location }
    Copy-Item (Join-Path $goc 'rust\target\release\zalo-cli.exe') $exeDich -Force
}
if (-not (Test-Path -LiteralPath $exeDich)) { throw "Không thấy $exeDich" }

$pl = Get-PhanLoai
Write-Host ''
Write-Host ("Assert trong bộ test : {0}" -f $pl.TongAssert) -ForegroundColor Cyan
Write-Host ("Phép thử LÁI CÔNG CỤ : {0}   ← đây mới là bằng chứng về bản Rust" -f $pl.E2E.Count) -ForegroundColor Cyan
Write-Host ("Phép thử soi mã nguồn: {0}   (canh bản PowerShell, không nói gì về bản Rust)" -f
            ($pl.TongAssert - $pl.E2E.Count)) -ForegroundColor DarkGray

Write-Host 'Lượt 1/2 · bản PowerShell (mốc đối chiếu)...' -ForegroundColor DarkGray
$ps = Invoke-BoTest $null
Write-Host 'Lượt 2/2 · zalo-cli.exe...' -ForegroundColor DarkGray
$rs = Invoke-BoTest $exeDich

# ---------------------------------------------------------------- đối chiếu
$hong = @()
foreach ($t in $pl.E2E) {
    if (-not $ps.ContainsKey($t)) { $hong += "$t  (không thấy trong lượt PowerShell)"; continue }
    if (-not $ps[$t])             { $hong += "$t  (chính bản PowerShell cũng hỏng)";   continue }
    if (-not $rs.ContainsKey($t)) { $hong += "$t  (không chạy tới ở bản Rust)";        continue }
    if (-not $rs[$t])             { $hong += $t }
}
# Phép soi mã nguồn cũng phải xanh — chúng canh bản PowerShell, mà bản PowerShell
# vẫn là công cụ người dùng đang chạy thật.
$hongNguon = @($ps.Keys | Where-Object { -not $ps[$_] })

Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host ("  Phép thử lái công cụ : {0}" -f $pl.E2E.Count)
Write-Host ("  Đạt với bản Rust     : {0}" -f ($pl.E2E.Count - $hong.Count)) -ForegroundColor Green
Write-Host ("  Hỏng                 : {0}" -f $hong.Count) -ForegroundColor $(if ($hong.Count) { 'Red' } else { 'Green' })
foreach ($h in $hong) { Write-Host ("     • " + $h) -ForegroundColor Red }
if ($hongNguon.Count) {
    Write-Host ''
    Write-Host ("  Bản PowerShell tự hỏng: {0}" -f $hongNguon.Count) -ForegroundColor Red
    foreach ($h in $hongNguon) { Write-Host ("     • " + $h) -ForegroundColor Red }
}
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan

if ($hong.Count -gt 0 -or $hongNguon.Count -gt 0) {
    Write-Host '  Cổng đối chiếu CHƯA ĐẠT' -ForegroundColor Red
    exit 1
}
Write-Host '  Cổng đối chiếu ĐẠT — cùng một bộ test, hai công cụ, cùng kết quả' -ForegroundColor Green
exit 0
