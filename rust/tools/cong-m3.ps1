#Requires -Version 5.1
<#
    Cổng của mốc M3.

    Chạy ĐÚNG MỘT bộ test — ZaloCleanup.Tests.ps1 — hai lần: một lần lái bản
    PowerShell, một lần lái zalo-cli.exe. Không sửa một ký tự nào trong các phép
    thử; chỉ đổi đường dẫn công cụ qua biến môi trường ZALO_TOOL.

    Vì sao không chép bộ test ra làm bản thứ hai: chép là hai bản test trôi khỏi
    nhau, và lúc đó "cả hai đều xanh" chẳng chứng minh được gì về hai công cụ.

    ------------------------------------------------------------------
    PHẠM VI THẬT CỦA CỔNG NÀY, và một chỗ kế hoạch đặt sai

    Kế hoạch viết cổng M3 là "chạy được 69 phép thử E2E hiện có". Đếm lại trên
    mã nguồn thì con số đúng là 67, và quan trọng hơn: 39 trong số đó đòi công
    cụ XÓA, SAO LƯU hoặc KHÔI PHỤC — những việc mà chính kế hoạch xếp vào mốc
    M4. Đòi đủ 67 phép ở M3 là đòi một thứ không thể đạt.

    Đây là lỗi cùng loại với lỗi đã bắt được ở cổng M1, và cách xử lý cũng vậy:
    sửa spec cho đúng phạm vi rồi ghi lại vì sao, chứ không tuyên bố đạt một
    cổng viết sai.

    Cổng M3 đúng phạm vi = các phép thử E2E mà lượt lái công cụ KHÔNG xóa tệp
    nào. Phân loại theo HÀNH VI THẬT chứ không theo phím: lượt dedup và lượt dọn
    cache có xóa tệp dù không hề gõ XÓA, còn phím B lại chỉ là báo cáo.
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

# Các lượt lái công cụ KHÔNG xóa tệp nào, nhận dạng bằng chính chuỗi phím.
# Khóa theo chuỗi phím chứ không theo số dòng: số dòng trôi mỗi lần ai đó thêm
# một phép thử ở trên, còn chuỗi phím thì mô tả đúng thứ đang được lái.
$phimChiDoc = @(
    "@('0')"                                        # thoát ngay ở màn hình chính
    "@('1', '1', '', '', '', '0')"                  # xem mốc tuổi rồi lui
    "@('9', '2', '99', '', '', '0')"                # bộ lọc thư mục, nhập sai
    "@('9', '2', '*', '', '', '0')"                 # bộ lọc thư mục, chọn tất cả
    "@('9', '7', '', 'X', '2', 'c', '', '', '0')"   # vào cửa xóa rồi gõ sai, bị chặn
    "@('1', '2', 'c', 'k', 'k', 'k', '', '0', '0')" # dedup, hủy ở bước cuối
    "@('1', '2', 'c', 'k', 'k', 'c', '', '0', '0')" # dedup, không còn cặp nào
    "@('9', 'B', '', '', '0')"                      # báo cáo vùng bảo vệ
    "@('9', '0', '0')"                              # nhãn menu nâng cao
    "@('1', '1', '0', '0', '0')"                    # mốc tuổi rỗng bị ẩn
    "@('3', '', '', '0')"                           # duyệt bản sao lưu
    "@('3', 'x 1', '', '', '0')"                    # xem trong bản sao lưu
)

# Phép thử "đạt" chỉ vì công cụ CHƯA BIẾT XÓA. Chúng vẫn thuộc phạm vi cổng,
# nhưng phải nêu tên ra: một phép thử xanh vì lý do sai là một phép thử chưa
# chứng minh được gì, và giấu nó đi là tự lừa mình ở đúng chỗ nguy hiểm nhất.
$xanhVoNghia = @(
    'Dữ liệu thật đòi gõ đúng chữ XÓA'
    'Lượt vừa rồi chưa xóa gì'
    'Tệp trong resource sống sót khi bản gốc đã mất'
)

function Get-AssertChiDoc {
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
        $phim = $goi[0].CommandElements[2].Extent.Text
        $nguon += [pscustomobject]@{
            Dong   = $a.Extent.StartLineNumber
            Bien   = ($bien | Sort-Object -Unique)
            Phim   = $phim
            ChiDoc = ($phimChiDoc -contains $phim)
        }
    }
    # Không đếm tổng số lượt khớp: một chuỗi phím có thể xuất hiện nhiều lần
    # trong bộ test một cách hoàn toàn chính đáng. Thứ phải kiểm là mỗi mẫu đã
    # khai báo đều CÒN tìm thấy — mẫu không khớp nữa nghĩa là bộ test đã đổi và
    # việc phân loại phải làm lại bằng tay chứ không được đoán.
    $mat = @($phimChiDoc | Where-Object { $nguon.Phim -notcontains $_ })
    if ($mat.Count -gt 0) {
        $msg = "Danh sách phím chỉ đọc đã lệch khỏi bộ test. Không còn tìm thấy:`n"
        foreach ($m in $mat) { $msg += "   $m`n" }
        $msg += 'Bộ test vừa đổi — phải phân loại lại bằng tay, không được đoán.'
        throw $msg
    }

    $chiDoc = @(); $coXoa = @()
    foreach ($as in $ast.FindAll({ param($n)
        $n -is [Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'Assert' }, $true)) {
        $dong = $as.Extent.StartLineNumber
        $txt = $as.Extent.Text
        $ung = $nguon | Where-Object {
            if ($_.Dong -ge $dong) { return $false }
            foreach ($b in $_.Bien) { if ($txt -match ('\$' + [regex]::Escape($b) + '\b')) { return $true } }
            return $false
        } | Sort-Object Dong -Descending | Select-Object -First 1
        if ($null -eq $ung) { continue }
        $ten = $as.CommandElements[1].Extent.Text.Trim("'")
        if ($ung.ChiDoc) { $chiDoc += $ten } else { $coXoa += $ten }
    }
    return [pscustomobject]@{ ChiDoc = $chiDoc; CoXoa = $coXoa }
}

function Invoke-BoTest($duongDanCongCu) {
    if ($duongDanCongCu) { $env:ZALO_TOOL = $duongDanCongCu }
    else { Remove-Item Env:\ZALO_TOOL -ErrorAction SilentlyContinue }
    # Ở đây thì 2>&1 là cần: bộ test có thể in cảnh báo ra stderr và ta muốn giữ
    # đủ bản ghi. Tạm hạ ErrorActionPreference để lời gọi ngoài không tự thành lỗi.
    $cu = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $out = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $boTest 2>&1 | Out-String -Width 300
    $ErrorActionPreference = $cu
    Remove-Item Env:\ZALO_TOOL -ErrorAction SilentlyContinue
    $kq = @{}
    foreach ($m in [regex]::Matches($out, '\[(ĐẠT|HỎNG)\s*\]\s*(.+?)\r?\n')) {
        $kq[$m.Groups[2].Value.Trim()] = ($m.Groups[1].Value -eq 'ĐẠT')
    }
    return $kq
}

# ---------------------------------------------------------------- dựng
if (-not $BoQuaDung) {
    Write-Host 'Đang dựng zalo-cli.exe...' -ForegroundColor DarkGray
    Push-Location (Join-Path $goc 'rust')
    try {
        # KHÔNG dùng 2>&1 với chương trình ngoài: PowerShell 5.1 bọc từng dòng
        # stderr thành ErrorRecord, và với $ErrorActionPreference = 'Stop' thì
        # một dòng tiến độ bình thường của cargo cũng làm cả script dừng.
        & cargo build --release -p zalo-cli | Out-Null
        if ($LASTEXITCODE -ne 0) { throw 'cargo build hỏng' }
    } finally { Pop-Location }
    Copy-Item (Join-Path $goc 'rust\target\release\zalo-cli.exe') $exeDich -Force
}
if (-not (Test-Path -LiteralPath $exeDich)) { throw "Không thấy $exeDich" }

$phanLoai = Get-AssertChiDoc
$chiDoc = $phanLoai.ChiDoc
$coXoa  = $phanLoai.CoXoa
Write-Host ''
Write-Host ("Phép thử đầu-cuối : {0}   (chỉ đọc {1} · có xóa {2})" -f
            ($chiDoc.Count + $coXoa.Count), $chiDoc.Count, $coXoa.Count) -ForegroundColor Cyan
Write-Host  "Phạm vi cổng M3   : $($chiDoc.Count) phép chỉ đọc. Phần có xóa thuộc mốc M4." -ForegroundColor Cyan

Write-Host 'Lượt 1/2 · bản PowerShell (mốc đối chiếu)...' -ForegroundColor DarkGray
$ps = Invoke-BoTest $null
Write-Host 'Lượt 2/2 · zalo-cli.exe...' -ForegroundColor DarkGray
$rs = Invoke-BoTest $exeDich

# ---------------------------------------------------------------- đối chiếu
$hong = @(); $vacuous = @()
foreach ($t in $chiDoc) {
    if (-not $ps.ContainsKey($t)) { $hong += "$t  (không thấy trong lượt PowerShell)"; continue }
    if (-not $ps[$t])             { $hong += "$t  (chính bản PowerShell cũng hỏng)";   continue }
    if (-not $rs.ContainsKey($t)) { $hong += "$t  (không chạy tới ở bản Rust)";        continue }
    if (-not $rs[$t])             { $hong += $t;                                       continue }
    if ($xanhVoNghia -contains $t) { $vacuous += $t }
}

$m4Hong  = @($coXoa | Where-Object { $rs.ContainsKey($_) -and -not $rs[$_] })
$m4Xanh  = @($coXoa | Where-Object { $rs.ContainsKey($_) -and $rs[$_] })

Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host ("  Trong phạm vi cổng : {0}" -f $chiDoc.Count)
Write-Host ("  Đạt                : {0}" -f ($chiDoc.Count - $hong.Count)) -ForegroundColor Green
Write-Host ("  Hỏng               : {0}" -f $hong.Count) -ForegroundColor $(if ($hong.Count) { 'Red' } else { 'Green' })
foreach ($h in $hong) { Write-Host ("     • " + $h) -ForegroundColor Red }
if ($vacuous.Count) {
    Write-Host ''
    Write-Host ("  Trong đó XANH VÔ NGHĨA: {0}" -f $vacuous.Count) -ForegroundColor Yellow
    Write-Host '  (đạt chỉ vì bản Rust chưa biết xóa — chưa chứng minh được gì)' -ForegroundColor DarkGray
    foreach ($v in $vacuous) { Write-Host ("     • " + $v) -ForegroundColor Yellow }
}
Write-Host ''
Write-Host ("  Ngoài phạm vi (phần có xóa, chờ M4): {0}" -f $coXoa.Count) -ForegroundColor DarkGray
Write-Host ("     hỏng {0} · xanh {1}" -f $m4Hong.Count, $m4Xanh.Count) -ForegroundColor DarkGray
Write-Host '     Phần xanh ở đây KHÔNG phải bằng chứng: gần hết là những phép thử' -ForegroundColor DarkGray
Write-Host '     kiểu "tệp vẫn còn đó", mà một công cụ chưa biết xóa thì luôn đúng.' -ForegroundColor DarkGray
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan

if ($hong.Count -gt 0) { Write-Host '  Cổng M3 CHƯA ĐẠT' -ForegroundColor Red; exit 1 }
Write-Host '  Cổng M3 ĐẠT' -ForegroundColor Green
exit 0
