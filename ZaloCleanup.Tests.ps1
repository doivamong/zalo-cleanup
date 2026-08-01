#Requires -Version 5.1
<#
    Bộ test hồi quy cho công cụ Dọn dẹp Zalo.
    Chạy: powershell -NoProfile -ExecutionPolicy Bypass -File ZaloCleanup.Tests.ps1
    Thêm -Full để chạy cả các phép thử chậm.

    Mỗi test dựng sandbox riêng trong %TEMP%, không bao giờ đụng vào dữ liệu Zalo thật.
    Tệp này phải lưu dạng UTF-8 CÓ BOM vì có chữ tiếng Việt trong chuỗi nhập liệu.
#>
param([switch]$Full)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }
# Bắt buộc: PowerShell 5.1 mặc định dùng ASCII khi truyền dữ liệu sang chương
# trình ngoài, nên chữ có dấu như XÓA sẽ thành X?A trước khi tới công cụ.
$OutputEncoding = [Text.Encoding]::UTF8
$runStart = Get-Date
$tool = Join-Path $PSScriptRoot 'ZaloCleanup.ps1'
$sbRoot = Join-Path $env:TEMP ('zct_' + [Guid]::NewGuid().ToString('N').Substring(0, 8))
$logDir = Join-Path $PSScriptRoot 'logs'
$script:Pass = 0; $script:Fail = 0; $script:Results = @()

function Assert($name, $cond, $detail) {
    if ($cond) { $script:Pass++; $st = 'ĐẠT'; $col = 'Green' }
    else       { $script:Fail++; $st = 'HỎNG'; $col = 'Red' }
    $script:Results += [pscustomobject]@{ Test = $name; KQ = $st; Ghi_chú = $detail }
    Write-Host ("  [{0,-4}] {1}" -f $st, $name) -ForegroundColor $col
    if (-not $cond -and $detail) { Write-Host ("         " + $detail) -ForegroundColor DarkRed }
}

# Đọc số lượng từ đầu ra công cụ mà không phụ thuộc vùng miền.
# Dấu phân cách hàng nghìn đổi theo vùng: 20,000 ở en-US nhưng 20.000 ở vi-VN,
# thậm chí là dấu cách hẹp ở fr-FR. Nên bóc hết ký tự không phải chữ số.
function Get-ReportedCount($output, $label) {
    if ($output -match ($label + '\s*:\s*(.+?)\s*tệp')) {
        $digits = $Matches[1] -replace '[^\d]', ''
        $n = 0
        if ($digits -ne '' -and [int]::TryParse($digits, [ref]$n)) { return $n }
    }
    return -1
}

function Invoke-Tool($root, $keys, $dataRoot) {
    $s = ($keys -join "`r`n") + "`r`n"
    if ($dataRoot) {
        return ($s | powershell.exe -NoProfile -ExecutionPolicy Bypass -File $tool -Root $root -DataRoot $dataRoot 2>&1 | Out-String -Width 200)
    }
    return ($s | powershell.exe -NoProfile -ExecutionPolicy Bypass -File $tool -Root $root 2>&1 | Out-String -Width 200)
}

function New-Sandbox($name) {
    $p = Join-Path $sbRoot "$name\ZaloDownloads"
    New-Item -ItemType Directory -Force $p | Out-Null
    return $p
}

function New-TestFile($path, $bytes, $date) {
    $d = Split-Path $path -Parent
    if (-not (Test-Path -LiteralPath $d)) { New-Item -ItemType Directory -Force $d | Out-Null }
    [IO.File]::WriteAllBytes($path, $bytes)
    (Get-Item -LiteralPath $path).LastWriteTime = $date
}

# Vài phép thử phải tạm đụng vào tệp cấu hình THẬT cạnh script — catalog.json
# và settings.json — rồi trả lại nguyên trạng. Cặp hàm này lo việc đó.
#
# Không dùng Get-Content -Raw rồi Set-Content để cất giữ: Set-Content luôn thêm
# một dòng mới ở cuối, nên mỗi lần chạy test lại làm tệp phình thêm một dòng
# trống. Với catalog.json — tệp có trong git — hậu quả là cây làm việc bẩn ra
# sau mỗi lần chạy dù chẳng ai sửa gì, và lập trình viên đi tìm nguyên nhân
# của một thay đổi không ai tạo ra.
#
# Đọc và ghi thẳng byte thì không phải đoán bảng mã, không đụng tới BOM, và
# tệp trả lại giống hệt tệp lấy đi.
function Backup-RealFile($path) {
    if (-not (Test-Path -LiteralPath $path)) { return $null }
    # Dấu phẩy đầu dòng để PowerShell trả về nguyên mảng byte thay vì rải nó
    # ra thành từng phần tử rời — thiếu nó thì WriteAllBytes không nhận.
    return ,[IO.File]::ReadAllBytes($path)
}

function Restore-RealFile($path, $bytes) {
    if ($null -eq $bytes) {
        Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
        return
    }
    [IO.File]::WriteAllBytes($path, $bytes)
}

# Dọn sandbox. Phải gỡ junction TRƯỚC, đúng bài học của chính công cụ:
# Remove-Item -Recurse gặp reparse point thì hỏng, và -ErrorAction SilentlyContinue
# nuốt mất lỗi — hậu quả là sandbox ở lại %TEMP% kèm tệp sparse 900 GB.
function Remove-Sandbox($path) {
    if ([string]::IsNullOrWhiteSpace($path) -or -not (Test-Path -LiteralPath $path)) { return }
    Get-ChildItem $path -Recurse -Directory -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint } |
        Sort-Object { $_.FullName.Length } -Descending |
        ForEach-Object { try { [IO.Directory]::Delete($_.FullName, $false) } catch { } }
    try { Remove-Item $path -Recurse -Force -ErrorAction Stop } catch { }
    if (Test-Path -LiteralPath $path) {
        Write-Host ("  CẢNH BÁO: không dọn sạch được sandbox " + $path) -ForegroundColor Red
        Write-Host  '  Hãy xóa thủ công — trong đó có tệp sparse rất lớn.' -ForegroundColor Red
    }
}

# Lần chạy trước bị ngắt giữa chừng thì sandbox ở lại. Quét dọn trước khi bắt đầu.
$stale = @(Get-ChildItem $env:TEMP -Directory -Filter 'zct_*' -ErrorAction SilentlyContinue |
           Where-Object { $_.LastWriteTime -lt (Get-Date).AddMinutes(-10) })
if ($stale.Count -gt 0) {
    Write-Host ("  Dọn {0} sandbox còn sót từ lần chạy trước..." -f $stale.Count) -ForegroundColor DarkGray
    $stale | ForEach-Object { Remove-Sandbox $_.FullName }
}

Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host '  BỘ TEST HỒI QUY — Dọn dẹp Zalo' -ForegroundColor Cyan
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host ("  Sandbox: " + $sbRoot) -ForegroundColor DarkGray
Write-Host ''

$old = [datetime]'2025-06-01'
$rnd = New-Object Random 1234

# ---------------------------------------------------------------- cú pháp và mã hóa
Write-Host '── Cú pháp và mã hóa' -ForegroundColor Yellow
$errs = $null; $toks = $null
[System.Management.Automation.Language.Parser]::ParseFile($tool, [ref]$toks, [ref]$errs) | Out-Null
Assert 'Script không có lỗi cú pháp' (-not $errs -or $errs.Count -eq 0) ("Số lỗi: " + @($errs).Count)

$b = [IO.File]::ReadAllBytes($tool)
Assert 'Script lưu dạng UTF-8 có BOM' ($b[0] -eq 0xEF -and $b[1] -eq 0xBB -and $b[2] -eq 0xBF) `
    'Thiếu BOM thì PowerShell 5.1 đọc theo ANSI và chữ tiếng Việt sẽ vỡ'

$txt = [IO.File]::ReadAllText($tool, [Text.Encoding]::UTF8)
Assert 'Chữ tiếng Việt còn nguyên vẹn trong mã nguồn' ($txt -match 'Bạn muốn làm gì' -and $txt -match 'Lấy lại dung lượng') `
    'Chuỗi tiếng Việt bị hỏng'

# ---------------------------------------------------------------- luồng nhập cạn
Write-Host ''
Write-Host '── Luồng nhập cạn phải thoát, không được treo' -ForegroundColor Yellow
$rEof = New-Sandbox 'eof'
New-TestFile (Join-Path $rEof 'video\e1') (New-Object byte[] 512) $old
$job = Start-Job -ScriptBlock { param($t, $r) "1`r`n" | powershell.exe -NoProfile -ExecutionPolicy Bypass -File $t -Root $r 2>&1 | Out-String } -ArgumentList $tool, $rEof
$fin = Wait-Job $job -Timeout 90
Assert 'Công cụ tự thoát khi hết luồng nhập' ($null -ne $fin) 'TREO — vòng lặp vô tận'
if ($null -eq $fin) { Stop-Job $job -EA SilentlyContinue }
Remove-Job $job -Force -EA SilentlyContinue

# ---------------------------------------------------------------- màn hình chính
Write-Host ''
Write-Host '── Màn hình chính chỉ có 5 lựa chọn' -ForegroundColor Yellow
$rHome = New-Sandbox 'home'
New-TestFile (Join-Path $rHome 'video\h1') (New-Object byte[] 512) $old
$o = Invoke-Tool $rHome @('0')
Assert 'Hiện câu hỏi mục đích thay vì danh sách công cụ' ($o -match 'Bạn muốn làm gì') 'Không thấy câu hỏi dẫn dắt'
Assert 'Hiện dung lượng ổ đĩa ngay màn hình đầu' ($o -match 'Ổ C còn trống') 'Thiếu thông tin dung lượng'
Assert 'Không lộ khái niệm quét ở màn hình đầu' (-not ($o -match 'QUÉT theo bộ lọc')) 'Vẫn lộ khái niệm nội bộ'

# ---------------------------------------------------------------- mốc thời gian có dung lượng
Write-Host ''
Write-Host '── Mốc thời gian hiện dung lượng thật' -ForegroundColor Yellow
$rAge = New-Sandbox 'age'
New-TestFile (Join-Path $rAge 'video\cu1') (New-Object byte[] 2048) ([datetime]'2024-01-15')
New-TestFile (Join-Path $rAge 'video\cu2') (New-Object byte[] 4096) ([datetime]'2025-03-10')
$o = Invoke-Tool $rAge @('1', '1', '', '', '', '0')
Assert 'Mỗi mốc thời gian kèm dung lượng đo được' ($o -match 'Cũ hơn 12 tháng\s+→') 'Thiếu dung lượng theo mốc'
Assert 'Mốc thời gian kèm số tệp' ($o -match 'tệp\)') 'Thiếu số tệp'

# ---------------------------------------------------------------- G1 bộ lọc thư mục
Write-Host ''
Write-Host '── G1: bộ lọc thư mục không được tự mở rộng' -ForegroundColor Yellow
$r1 = New-Sandbox 'g1'
New-TestFile (Join-Path $r1 'video\v1') (New-Object byte[] 2048) $old
New-TestFile (Join-Path $r1 'Cache\c1') (New-Object byte[] 2048) $old
$o = Invoke-Tool $r1 @('9', '2', '99', '', '', '0')
Assert 'G1 nhập sai thì báo lỗi' ($o -match 'không hợp lệ') 'Không thấy thông báo lỗi'
Assert 'G1 nhập sai thì giữ nguyên bộ lọc' ($o -match 'giữ nguyên, không đổi gì') 'Không thấy thông báo giữ nguyên'
Assert 'G1 nhập sai không âm thầm chọn tất cả' (-not ($o -match 'Đã đặt: tất cả thư mục')) 'Vẫn âm thầm mở rộng'
$o = Invoke-Tool $r1 @('9', '2', '*', '', '', '0')
Assert 'G1 dấu * chọn tất cả một cách có ý' ($o -match 'Đã đặt: tất cả thư mục') 'Dấu * không hoạt động'

# ---------------------------------------------------------------- G4 G5 xóa dữ liệu thật
Write-Host ''
Write-Host '── G4/G5: đếm đúng và nhật ký đúng' -ForegroundColor Yellow
$r2 = New-Sandbox 'g45'
New-TestFile (Join-Path $r2 'video\a') (New-Object byte[] 1024) $old
New-TestFile (Join-Path $r2 'video\b') (New-Object byte[] 2048) $old
$ro = Join-Path $r2 'video\ro'
New-TestFile $ro (New-Object byte[] 4096) $old
(Get-Item -LiteralPath $ro).Attributes = 'ReadOnly'
# 2 = xóa không sao lưu, XÓA = xác nhận nặng vì đây là dữ liệu thật
$o = Invoke-Tool $r2 @('9', '7', '', 'X', '2', 'XÓA', '', '', '0')
Assert 'G4 xóa hết 3 tệp kể cả tệp chỉ đọc' ((Get-ReportedCount $o 'Đã xóa') -eq 3) `
    ("Công cụ báo " + (Get-ReportedCount $o 'Đã xóa') + " tệp")
Assert 'G4 sandbox rỗng sau khi xóa' (@(Get-ChildItem $r2 -Recurse -File -Force -EA SilentlyContinue).Count -eq 0) 'Còn sót tệp'
Assert 'Báo cáo dung lượng ổ đĩa trước và sau' `
    (($o -match 'Ổ đĩa trước') -and ($o -match 'Thực tế thu được')) 'Thiếu phần đối chiếu dung lượng thật'

# Chỉ xét nhật ký sinh ra TRONG lần chạy này. Lấy tệp mới nhất bất kể thời điểm
# sẽ vớ phải log của lần chạy tay trước đó và làm test tự lừa mình.
$log = Get-ChildItem $logDir -Filter 'daxoa_*.log' -EA SilentlyContinue |
       Where-Object { $_.LastWriteTime -ge $runStart } | Sort-Object LastWriteTime -Desc | Select-Object -First 1
Assert 'G5 có sinh ra nhật ký trong lần chạy này' ($null -ne $log) 'Không tìm thấy nhật ký nào của lần chạy này'
if ($null -ne $log) {
    $lines = Get-Content $log.FullName -Encoding UTF8
    Assert 'G5 nhật ký có 3 dòng ĐÃXÓA' (@($lines | Where-Object { $_ -like 'ĐÃXÓA*' }).Count -eq 3) 'Sai số dòng'
    Assert 'G5 nhật ký ghi rõ không sao lưu' ((($lines -join "`n")) -match 'Sao lưu : không') 'Không ghi trạng thái sao lưu'
    Assert 'G5 tổng kết ghi hoàn tất=True' ((($lines -join "`n")) -match 'hoàn tất=True') 'Thiếu trạng thái hoàn tất'
}

# ---------------------------------------------------------------- xác nhận tương xứng rủi ro
Write-Host ''
Write-Host '── Mức xác nhận tương xứng với rủi ro' -ForegroundColor Yellow
$r2b = New-Sandbox 'confirm'
New-TestFile (Join-Path $r2b 'video\x1') (New-Object byte[] 1024) $old
# gõ c thay vì XÓA với dữ liệu thật => phải bị từ chối
$o = Invoke-Tool $r2b @('9', '7', '', 'X', '2', 'c', '', '', '0')
Assert 'Dữ liệu thật đòi gõ đúng chữ XÓA' (@(Get-ChildItem $r2b -Recurse -File -Force -EA SilentlyContinue).Count -eq 1) `
    'Đã xóa dù chỉ gõ c'

# ---------------------------------------------------------------- khử trùng lặp
Write-Host ''
Write-Host '── Khử trùng lặp chỉ xóa bản đã xác minh hash' -ForegroundColor Yellow
$r3 = New-Sandbox 'dedup'
$b1 = New-Object byte[] 300000; $rnd.NextBytes($b1)
$b2 = New-Object byte[] 300000; $rnd.NextBytes($b2)
$b3 = New-Object byte[] 150000; $rnd.NextBytes($b3)
New-TestFile (Join-Path $r3 'video\goc1') $b1 $old
New-TestFile (Join-Path $r3 'resource\c1\video\ban_trung') $b1 $old
New-TestFile (Join-Path $r3 'resource\c1\video\cung_co_khac_noi_dung') $b2 $old
New-TestFile (Join-Path $r3 'resource\c1\video\duy_nhat') $b3 $old
New-TestFile (Join-Path $r3 'resource\c1\Cache\ban_sao_trong_cache') $b1 $old

$o = Invoke-Tool $r3 @('1', '2', 'c', 'k', 'c', '', '', '0')
Assert 'Dedup xác nhận đúng 1 bản trùng' ((Get-ReportedCount $o 'Bản trùng xác nhận') -eq 1) `
    ("Công cụ báo " + (Get-ReportedCount $o 'Bản trùng xác nhận') + " bản trùng")
Assert 'Dedup xóa đúng 1 tệp' ((Get-ReportedCount $o 'Đã xóa') -eq 1) `
    ("Công cụ báo " + (Get-ReportedCount $o 'Đã xóa') + " tệp")
Assert 'Dedup giữ nguyên bản gốc' (Test-Path (Join-Path $r3 'video\goc1')) 'Bản gốc bị xóa'
Assert 'Dedup không đụng tệp duy nhất' (Test-Path (Join-Path $r3 'resource\c1\video\duy_nhat')) 'Tệp duy nhất bị xóa'
Assert 'Dedup không đụng tệp cùng cỡ khác nội dung' (Test-Path (Join-Path $r3 'resource\c1\video\cung_co_khac_noi_dung')) 'Xóa nhầm'
Assert 'Dedup không đụng tệp trong Cache' (Test-Path (Join-Path $r3 'resource\c1\Cache\ban_sao_trong_cache')) 'Xóa nhầm trong Cache'
Assert 'Dedup chỉ cần xác nhận nhẹ' (-not ($o -match 'Gõ đúng chữ  XÓA')) 'Bắt gõ XÓA cho bản trùng lặp'

# ---------------------------------------------------------------- vùng bảo vệ
Write-Host ''
Write-Host '── G9: vùng bảo vệ Database và Partitions' -ForegroundColor Yellow
$dataRoot = Join-Path $sbRoot 'prot'
$r4 = Join-Path $dataRoot 'media\acc\ZaloDownloads'
New-Item -ItemType Directory -Force $r4 | Out-Null
New-TestFile (Join-Path $r4 'video\v1') (New-Object byte[] 1024) $old
New-TestFile (Join-Path $dataRoot 'Database\_production\chat.db') (New-Object byte[] 8192) $old
New-TestFile (Join-Path $dataRoot 'Partitions\session\p1') (New-Object byte[] 4096) $old
New-TestFile (Join-Path $dataRoot 'Cache\appcache1') (New-Object byte[] 2048) $old

$o = Invoke-Tool $r4 @('1', '3', 'c', '', '', '0') $dataRoot
Assert 'Cache ứng dụng được dọn' ((Get-ReportedCount $o 'Đã xóa') -eq 1) `
    ("Công cụ báo " + (Get-ReportedCount $o 'Đã xóa') + " tệp")
Assert 'Database vẫn còn sau khi dọn cache' (Test-Path (Join-Path $dataRoot 'Database\_production\chat.db')) 'Database bị đụng tới'
Assert 'Partitions vẫn còn sau khi dọn cache' (Test-Path (Join-Path $dataRoot 'Partitions\session\p1')) 'Partitions bị đụng tới'

$o = Invoke-Tool $r4 @('9', 'B', '', '', '0') $dataRoot
Assert 'Báo cáo vùng bảo vệ chạy được' ($o -match 'chỉ báo cáo, không bao giờ xóa') 'Không hiện báo cáo'

# ---------------------------------------------------------------- G3 dung lượng ổ đích
Write-Host ''
Write-Host '── G3: sao lưu kiểm tra dung lượng ổ đích' -ForegroundColor Yellow
$r5 = New-Sandbox 'g3'
$huge = Join-Path $r5 'video\sparse_huge'
New-Item -ItemType Directory -Force (Split-Path $huge -Parent) | Out-Null
$fs = [IO.File]::Create($huge); $fs.Close()
& fsutil sparse setflag "$huge" | Out-Null
$fs = [IO.File]::Open($huge, 'Open', 'Write'); $fs.SetLength(900GB); $fs.Close()
(Get-Item -LiteralPath $huge).LastWriteTime = $old

$o = Invoke-Tool $r5 @('9', '7', '', '9', (Join-Path $sbRoot 'bkdest'), '', '', '0')
Assert 'G3 chặn khi không đủ chỗ' ($o -match 'Không đủ chỗ') 'Không chặn'
Assert 'G3 không tạo thư mục đích khi đã chặn' (-not (Test-Path (Join-Path $sbRoot 'bkdest'))) 'Đã tạo thư mục dù bị chặn'
$o = Invoke-Tool $r5 @('9', '7', '', '9', '???:\bad|path', '', '', '0')
Assert 'G3 báo lỗi đường dẫn không hợp lệ' ($o -match 'không hợp lệ') 'Không báo lỗi đường dẫn'

# ---------------------------------------------------------------- kiểm thử đơn vị
Write-Host ''
Write-Host '── Hạ tầng: Get-RelPath, Remove-EmptyDirs, vùng bảo vệ' -ForegroundColor Yellow
$errs2 = $null; $toks2 = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($tool, [ref]$toks2, [ref]$errs2)
foreach ($fn in @('Get-RelPath', 'Test-Protected', 'Build-ProtectedIndex', 'Test-ProtectedRoot',
                  'Initialize-ProtectedAbs',
                  'Test-IsReparsePoint', 'Remove-EmptyDirs', 'Invoke-TruncateLocked',
                  'Test-CatalogEntry', 'Get-CatalogDefs', 'Get-CatalogDefaults',
                  'Get-FreeBytes', 'Get-LongPath', 'Get-DriveLabel')) {
    $node = $ast.Find({ param($x) $x -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $x.Name -eq $fn }, $true)
    if ($node) { Invoke-Expression $node.Extent.Text }
}
$script:ProtectedAbs = @(); $script:ProtectedNames = @('Database', 'Partitions'); $script:DataRoot = ''
$script:ToolDir = Split-Path $tool -Parent
$script:CatalogFile = Join-Path $script:ToolDir 'catalog.json'
$script:SysDrive = $env:SystemDrive
$script:SysRoot = $env:SystemDrive + '\'
$script:LongPathOK = $true
Initialize-ProtectedAbs

Assert 'Get-RelPath với gốc là ổ đĩa' ((Get-RelPath 'C:\Users\A\b.txt' 'C:\') -eq 'Users\A\b.txt') ("Nhận được: " + (Get-RelPath 'C:\Users\A\b.txt' 'C:\'))
Assert 'Get-RelPath gốc không có dấu gạch cuối' ((Get-RelPath 'C:\x\y\z' 'C:\x') -eq 'y\z') 'Sai'
Assert 'Get-RelPath không phân biệt hoa thường' ((Get-RelPath 'C:\X\y' 'c:\x') -eq 'y') 'Sai'

$sw = [Diagnostics.Stopwatch]::StartNew()
$r = Remove-EmptyDirs @('C:\') $false
$sw.Stop()
Assert 'Remove-EmptyDirs từ chối gốc ổ đĩa' ($r -eq 0) "Đã xóa $r thư mục"
Assert 'Remove-EmptyDirs từ chối ngay lập tức' ($sw.Elapsed.TotalSeconds -lt 2) ("Mất {0:N1}s — có thể đã duyệt cả ổ đĩa" -f $sw.Elapsed.TotalSeconds)

$ed = Join-Path $sbRoot 'emptydirs'
New-Item -ItemType Directory -Force (Join-Path $ed 'top\sub1\sub2') | Out-Null
New-TestFile (Join-Path $ed 'top\giulai\f') (New-Object byte[] 16) $old
Remove-EmptyDirs @($ed) $false | Out-Null
Assert 'Remove-EmptyDirs xóa thư mục rỗng lồng nhau' (-not (Test-Path (Join-Path $ed 'top\sub1'))) 'Còn sót thư mục rỗng'
Assert 'Remove-EmptyDirs giữ thư mục có tệp' (Test-Path (Join-Path $ed 'top\giulai\f')) 'Xóa nhầm thư mục có tệp'

Assert 'Chặn WinSxS'            (Test-Protected (Join-Path $env:WINDIR 'WinSxS\abc')) 'Không chặn'
Assert 'Chặn Windows\Installer' (Test-Protected (Join-Path $env:WINDIR 'Installer\x.msi')) 'Không chặn'
Assert 'Chặn hiberfil.sys'      (Test-Protected 'C:\hiberfil.sys') 'Không chặn'
Assert 'Chặn pagefile.sys'      (Test-Protected 'C:\pagefile.sys') 'Không chặn'
Assert 'Chặn vm_bundles'        (Test-Protected (Join-Path $env:APPDATA 'Claude\vm_bundles\x.vhdx')) 'Không chặn'
Assert 'Chặn .cargo\bin'        (Test-Protected (Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe')) 'Không chặn'
Assert 'Chặn .rustup'           (Test-Protected (Join-Path $env:USERPROFILE '.rustup\toolchains\x')) 'Không chặn'
Assert 'Công cụ không tự xóa chính mình' (Test-Protected $tool) 'Tự xóa được chính mình'
Assert 'Không chặn nhầm cache thường' (-not (Test-Protected (Join-Path $env:USERPROFILE '.cache\pip\x'))) 'Chặn nhầm'

$fb = Get-FreeBytes 'C:\'
Assert 'Get-FreeBytes đọc được dung lượng ổ C' ($fb -gt 0) ("Nhận được: $fb")
Assert 'Get-FreeBytes trả về -1 khi đường dẫn vô nghĩa' ((Get-FreeBytes '???|bad') -eq -1) 'Không trả về -1'

$defs = Get-CatalogDefs
$badDefs = @()
foreach ($d in $defs) {
    foreach ($p in $d.P) {
        if ($p -match '^[A-Za-z]:\\?$') { $badDefs += $p }
        if (Test-Protected ($p -replace '\*', 'x')) { $badDefs += $p }
    }
}
Assert 'Danh mục không trỏ vào gốc ổ đĩa hay vùng bảo vệ' ($badDefs.Count -eq 0) (($badDefs -join '; '))
Assert 'Danh mục có đủ ba nhóm A/B/C' (@($defs | ForEach-Object { $_.G } | Select-Object -Unique).Count -eq 3) 'Thiếu nhóm'
Assert 'Mọi mục danh mục đều có mô tả' (@($defs | Where-Object { -not $_.Note }).Count -eq 0) 'Có mục thiếu mô tả'
$tempDefs = @($defs | Where-Object { $_.P -contains $env:TEMP -or $_.P -contains (Join-Path $env:WINDIR 'Temp') })
Assert 'Tìm thấy hai mục tệp tạm' ($tempDefs.Count -eq 2) ("Tìm thấy " + $tempDefs.Count)
Assert 'Mục tệp tạm đều có ngưỡng tuổi' (@($tempDefs | Where-Object { -not $_.ContainsKey('Age') -or [int]$_.Age -lt 1 }).Count -eq 0) 'Thiếu ngưỡng tuổi'

# ---------------------------------------------------------------- vùng bảo vệ: depth và chiều ngược
Write-Host ''
Write-Host '── Vùng bảo vệ: mức gốc và chiều ngược' -ForegroundColor Yellow

Assert 'Chặn khi nhắm thẳng vào %LOCALAPPDATA%' (Test-Protected $env:LOCALAPPDATA) 'Không chặn gốc lớn'
Assert 'Chặn khi nhắm thẳng vào %APPDATA%'      (Test-Protected $env:APPDATA)      'Không chặn gốc lớn'
Assert 'Chặn khi nhắm thẳng vào %USERPROFILE%'  (Test-Protected $env:USERPROFILE)  'Không chặn gốc lớn'
# Mức 'gốc' chỉ chặn đúng thư mục đó. Chặn lan xuống con là hỏng cả danh mục cache.
Assert 'Mức gốc không chặn lan xuống con' `
    (-not (Test-Protected (Join-Path $env:LOCALAPPDATA 'npm-cache'))) `
    'Chặn nhầm mục cache hợp lệ'
Assert 'Mức gốc không chặn lan xuống cháu' `
    (-not (Test-Protected (Join-Path $env:LOCALAPPDATA 'npm-cache\_cacache\x'))) `
    'Chặn nhầm tệp trong cache hợp lệ'

# Chiều ngược: nhận một thư mục CHỨA vùng bảo vệ cũng nguy hiểm y như nhận chính nó.
Assert 'Test-ProtectedRoot chặn thư mục cha của vùng bảo vệ' (Test-ProtectedRoot $env:WINDIR) `
    '%WINDIR% chứa WinSxS mà vẫn lọt'
Assert 'Test-ProtectedRoot chặn gốc ổ đĩa' (Test-ProtectedRoot ($env:SystemDrive + '\')) 'Gốc ổ đĩa vẫn lọt'
Assert 'Test-ProtectedRoot vẫn cho qua cache hợp lệ' `
    (-not (Test-ProtectedRoot (Join-Path $env:LOCALAPPDATA 'npm-cache'))) `
    'Chặn nhầm mục cache hợp lệ'
# %APPDATA%\Claude là CHA của vùng bảo vệ %APPDATA%\Claude\vm_bundles, nhưng bản
# thân nó không phải một luật. Đây là chỗ hai hàm phải trả lời khác nhau.
$chaCuaVungBaoVe = Join-Path $env:APPDATA 'Claude'
Assert 'Test-ProtectedRoot chặn cha của vm_bundles' (Test-ProtectedRoot $chaCuaVungBaoVe) `
    'Nhận cả thư mục chứa vùng bảo vệ'
Assert 'Test-Protected không gánh phép kiểm tra chiều ngược' (-not (Test-Protected $chaCuaVungBaoVe)) `
    'Đường nhanh cho từng tệp bị gánh thêm việc, sẽ chậm cả lần quét'

# ---------------------------------------------------------------- junction và reparse point
Write-Host ''
Write-Host '── Junction: không quét xuyên, không xóa xuyên' -ForegroundColor Yellow

$jRoot   = Join-Path $sbRoot 'junction'
$jTarget = Join-Path $jRoot 'dich'
$jLink   = Join-Path $jRoot 'ZaloDownloads\video\lienket'
New-Item -ItemType Directory -Force $jTarget | Out-Null
New-Item -ItemType Directory -Force (Join-Path $jRoot 'ZaloDownloads\video') | Out-Null
New-TestFile (Join-Path $jTarget 'khongduocdung.bin') (New-Object byte[] 4096) $old
$jOK = $true
try { New-Item -ItemType Junction -Path $jLink -Target $jTarget -ErrorAction Stop | Out-Null }
catch { $jOK = $false }

if (-not $jOK) {
    Assert 'Tạo được junction trong sandbox' $false 'Không tạo được junction — bỏ qua nhóm test này'
} else {
    Assert 'Junction có cờ ReparsePoint' (Test-IsReparsePoint (Get-Item -LiteralPath $jLink -Force)) 'Thiếu cờ'

    # Khóa lại hành vi đã kiểm chứng của PowerShell 5.1: -Recurse KHÔNG đi xuyên
    # junction. Nếu một bản Windows sau đổi điều này, test sẽ đỏ ngay.
    $seen = @(Get-ChildItem -LiteralPath (Join-Path $jRoot 'ZaloDownloads') -Recurse -File -Force -ErrorAction SilentlyContinue)
    Assert 'Get-ChildItem -Recurse không đi xuyên junction' `
        (@($seen | Where-Object { $_.Name -eq 'khongduocdung.bin' }).Count -eq 0) `
        'Quét đã chui qua junction — dung lượng sẽ bị đếm trùng và phạm vi vượt ngoài dự tính'

    # Junction trỏ tới thư mục rỗng trông y hệt thư mục rỗng. Xóa đệ quy lên nó
    # có thể xóa xuyên sang đầu bên kia.
    $jEmptyTarget = Join-Path $jRoot 'dichrong'
    $jEmptyLink   = Join-Path $jRoot 'ZaloDownloads\picture\lienketrong'
    New-Item -ItemType Directory -Force $jEmptyTarget | Out-Null
    New-Item -ItemType Directory -Force (Join-Path $jRoot 'ZaloDownloads\picture') | Out-Null
    New-Item -ItemType Junction -Path $jEmptyLink -Target $jEmptyTarget -ErrorAction SilentlyContinue | Out-Null
    Remove-EmptyDirs @((Join-Path $jRoot 'ZaloDownloads')) $false | Out-Null
    Assert 'Dọn thư mục rỗng không xóa junction' (Test-Path -LiteralPath $jEmptyLink) 'Junction đã bị xóa'
    Assert 'Dọn thư mục rỗng không đụng đích của junction' (Test-Path -LiteralPath $jEmptyTarget) 'Đích đã bị xóa'
    Assert 'Đích của junction có tệp vẫn còn nguyên' (Test-Path -LiteralPath (Join-Path $jTarget 'khongduocdung.bin')) 'Tệp bên kia junction đã mất'
}

# ---------------------------------------------------------------- xóa thư mục rỗng không đệ quy
Write-Host ''
Write-Host '── Xóa thư mục rỗng: không đệ quy' -ForegroundColor Yellow

$srcRm = ([System.Management.Automation.Language.Parser]::ParseFile($tool, [ref]$null, [ref]$null)).Find(
    { param($x) $x -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $x.Name -eq 'Remove-EmptyDirs' }, $true)
Assert 'Remove-EmptyDirs không còn dùng Remove-Item -Recurse' `
    ($null -ne $srcRm -and $srcRm.Extent.Text -notmatch 'Remove-Item') `
    'Vẫn còn xóa đệ quy — tệp lọt vào khe hở sẽ bị xóa mà không qua vùng bảo vệ'
Assert 'Remove-EmptyDirs dùng Directory::Delete không đệ quy' `
    ($null -ne $srcRm -and $srcRm.Extent.Text -match '\[IO\.Directory\]::Delete\(.*,\s*\$false\)') `
    'Không thấy lời gọi xóa không đệ quy'

# Thư mục rỗng lúc kiểm tra, có tệp lúc xóa. Xóa không đệ quy phải chịu thua
# chứ không được cuốn theo tệp mới.
$rc = Join-Path $sbRoot 'race\muctieu'
New-Item -ItemType Directory -Force $rc | Out-Null
$rcFile = Join-Path $rc 'chen-vao-phut-chot.bin'
[IO.File]::WriteAllBytes($rcFile, (New-Object byte[] 32))
$rcRemoved = Remove-EmptyDirs @((Join-Path $sbRoot 'race')) $false
Assert 'Không xóa thư mục đã hết rỗng' (Test-Path -LiteralPath $rcFile) 'Tệp chen vào phút chót đã bị xóa mất'
Assert 'Không tính nhầm là đã xóa' ($rcRemoved -eq 0) ("Báo đã xóa $rcRemoved thư mục")

# ---------------------------------------------------------------- catalog.json: kiểm tra hợp lệ
Write-Host ''
Write-Host '── catalog.json: mục sai phải được nêu tên' -ForegroundColor Yellow

Assert 'Bắt lỗi gõ nhầm "path" thành "paths"' `
    (@(Test-CatalogEntry ([pscustomobject]@{ name = 'x'; path = @('%TEMP%') })).Count -gt 0) 'Không bắt được'
Assert 'Bắt lỗi thiếu name' `
    (@(Test-CatalogEntry ([pscustomobject]@{ paths = @('%TEMP%') })).Count -gt 0) 'Không bắt được'
Assert 'Bắt lỗi group sai' `
    (@(Test-CatalogEntry ([pscustomobject]@{ name = 'x'; paths = @('%TEMP%'); group = 'Z' })).Count -gt 0) 'Không bắt được'
Assert 'Bắt lỗi risk sai' `
    (@(Test-CatalogEntry ([pscustomobject]@{ name = 'x'; paths = @('%TEMP%'); risk = 'ĐỎ' })).Count -gt 0) 'Không bắt được'
Assert 'Bắt lỗi ageHours không phải số' `
    (@(Test-CatalogEntry ([pscustomobject]@{ name = 'x'; paths = @('%TEMP%'); ageHours = 'nhiều' })).Count -gt 0) 'Không bắt được'
Assert 'Mục hợp lệ thì không báo lỗi' `
    (@(Test-CatalogEntry ([pscustomobject]@{ name = 'x'; paths = @('%TEMP%'); group = 'C'; risk = 'XANH'; ageHours = 24 })).Count -eq 0) `
    'Báo lỗi oan cho mục đúng'

Assert 'catalog.json thật không có mục nào sai định dạng' `
    (@((Get-Content -LiteralPath (Join-Path $PSScriptRoot 'catalog.json') -Raw -Encoding UTF8 | ConvertFrom-Json).entries |
       Where-Object { @(Test-CatalogEntry $_).Count -gt 0 }).Count -eq 0) `
    'Có mục sai định dạng ngay trong catalog.json đi kèm'

$defsWarn = @($defs | Where-Object { $_.ContainsKey('Warn') -and $_.Warn -ne '' })
Assert 'catalog.json có mục mang cảnh báo' ($defsWarn.Count -ge 3) ("Chỉ có " + $defsWarn.Count)

$srcSel = [IO.File]::ReadAllText($tool, [Text.Encoding]::UTF8)
Assert 'Lệnh * bỏ qua mục có cảnh báo' ($srcSel -match "if \(\`$e\.Warning -eq ''\) \{ \`$sel\[\`$e\.Id\] = \`$true \}") `
    'Chọn tất cả vẫn kéo theo mục có cảnh báo'
Assert 'Chọn theo nhóm bỏ qua mục có cảnh báo' ($srcSel -match "\`$grp \| Where-Object \{ \`$_\.Warning -eq '' \}") `
    'Chọn theo nhóm vẫn kéo theo mục có cảnh báo'

# ---------------------------------------------------------------- cắt cụt tệp bị khóa
Write-Host ''
Write-Host '── Cắt cụt tệp bị khóa' -ForegroundColor Yellow

$tf = Join-Path $sbRoot 'truncate\bikhoa.bin'
New-Item -ItemType Directory -Force (Split-Path $tf -Parent) | Out-Null
[IO.File]::WriteAllBytes($tf, (New-Object byte[] 65536))
# Giữ tệp bằng handle chia sẻ, đúng kiểu trình duyệt giữ tệp cache của nó.
$hold = [IO.File]::Open($tf, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
try {
    $delFailed = $false
    try { [IO.File]::Delete($tf) } catch { $delFailed = $true }
    Assert 'Tệp đang bị giữ thì xóa không được' ($delFailed -or (Test-Path -LiteralPath $tf)) 'Xóa được — sandbox không mô phỏng đúng'
    Assert 'Cắt cụt thành công tệp đang bị giữ' (Invoke-TruncateLocked $tf) 'Không cắt cụt được'
    Assert 'Tệp bị cắt cụt còn 0 byte' ((New-Object IO.FileInfo $tf).Length -eq 0) `
        ("Còn " + (New-Object IO.FileInfo $tf).Length + " byte")
} finally { $hold.Dispose() }

Assert 'Cắt cụt trả về false khi tệp không tồn tại' `
    (-not (Invoke-TruncateLocked (Join-Path $sbRoot 'truncate\khong-ton-tai.bin'))) 'Trả về true cho tệp không có'

# Giới hạn có chủ ý: cắt cụt CHỈ dành cho cache, không bao giờ cho dữ liệu Zalo thật.
Assert 'Cắt cụt chỉ bật cho hai chế độ cache' `
    ($srcSel -match "\`$mayTruncate = \(\`$script:ScanKind -eq 'CACHE ZALO' -or \`$script:ScanKind -eq 'CACHE HỆ THỐNG'\)") `
    'Điều kiện bật cắt cụt đã đổi — kiểm tra lại xem dữ liệu Zalo thật có bị cắt cụt không'
Assert 'Nhật ký có trạng thái CẮTCỤT riêng' ($srcSel -match 'CẮTCỤT`t') 'Không phân biệt được với THẤTBẠI'
Assert 'Dòng tổng kết ghi số tệp bị cắt cụt' ($srcSel -match 'cắt cụt=\$trunc') 'Lịch sử sẽ đếm thiếu'

# ---------------------------------------------------------------- trải nghiệm dùng
Write-Host ''
Write-Host '── Trải nghiệm: màn hình sạch, không lời nhắc thừa' -ForegroundColor Yellow
$srcUx = [IO.File]::ReadAllText($tool, [Text.Encoding]::UTF8)

foreach ($fnName in @('Invoke-WizardReclaim', 'Show-AdvancedMenu', 'Invoke-CatalogScan', 'Invoke-WizardZaloOld')) {
    $node = ([System.Management.Automation.Language.Parser]::ParseFile($tool, [ref]$null, [ref]$null)).Find(
        { param($x) $x -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $x.Name -eq $fnName }, $true)
    Assert ("Màn hình $fnName làm sạch màn hình") ($null -ne $node -and $node.Extent.Text -match 'Clear-Host') `
        'Vẽ chồng lên màn hình cũ'
}

Assert 'Lời nhắc chỉ hiện khi có kết quả cần đọc' ($srcUx -match 'if \(\$coKetQua\) \{ Read-Line') `
    'Vẫn nhắc vô điều kiện, sẽ nuốt mất phím lệnh kế tiếp'
Assert 'Phím quay lại thống nhất là 0' ($srcUx -match "\`$c -eq '' -or \`$c -eq '0'") 'Không nhận phím 0'

# Mốc thời gian rỗng phải được ẩn
$rBucket = New-Sandbox 'bucket'
1..2 | ForEach-Object { New-TestFile (Join-Path $rBucket "video\b$_") (New-Object byte[] 4096) ([datetime]'2026-01-15') }
$oB = Invoke-Tool $rBucket @('1', '1', '0', '0', '0')
Assert 'Không hiện mốc thời gian rỗng' (-not ($oB -match '0 B\s+\(0 tệp\)')) 'Vẫn hiện lựa chọn 0 byte vô nghĩa'
Assert 'Giải thích vì sao thiếu mốc' ($oB -match 'không hiện vì không còn dữ liệu') 'Người dùng không hiểu chỗ trống'

# Không còn mốc nào thì nói rõ thay vì hiện danh sách rỗng
$rEmpty = New-Sandbox 'bucketrong'
New-TestFile (Join-Path $rEmpty 'video\moi') (New-Object byte[] 1024) (Get-Date)
$oE = Invoke-Tool $rEmpty @('1', '1', '0', '0', '0')
Assert 'Không còn dữ liệu cũ thì báo rõ ràng' ($oE -match 'Không còn dữ liệu cũ ở các mốc thường dùng') `
    'Hiện danh sách rỗng khó hiểu'

# ---------------------------------------------------------------- tính di động
Write-Host ''
Write-Host '── Tính di động: chạy được trên mọi máy Windows 11' -ForegroundColor Yellow

# P1 không còn ký tự ổ đĩa ghi cứng trong mã thực thi
$srcTxt = [IO.File]::ReadAllText($tool, [Text.Encoding]::UTF8)
$codeOnly = ($srcTxt -split "`r?`n" | Where-Object { $_.Trim() -notmatch '^#' }) -join "`n"
$hard = @([regex]::Matches($codeOnly, "(?<![\w%])[A-Za-z]:\\\\?(?![\\/])") | ForEach-Object {
    $codeOnly.Substring([Math]::Max(0, $_.Index - 40), 60) })
$hardReal = @($hard | Where-Object { $_ -notmatch 'ví dụ|Nhập thư mục đích' })
Assert 'Không còn ký tự ổ đĩa ghi cứng trong mã thực thi' ($hardReal.Count -eq 0) (($hardReal -join ' ||| '))

# P1 dùng biến môi trường cho ổ hệ thống
Assert 'Dùng $env:SystemDrive thay cho C: cố định' ($srcTxt -match '\$env:SystemDrive') 'Không thấy dùng SystemDrive'
Assert 'Lệnh vssadmin nhắm theo ổ hệ thống' ($srcTxt -match '/for=\$\(\$script:SysDrive\)') 'vssadmin vẫn ghi cứng ổ C'

# P2 chạy được khi không có Zalo
Assert 'Không thoát khi máy chưa cài Zalo' (-not ($srcTxt -match 'Không xác định được thư mục ZaloDownloads. Thoát')) `
    'Vẫn thoát khi không tìm thấy Zalo'
Assert 'Có cờ HasZalo để ẩn các mục cần Zalo' ($srcTxt -match '\$script:HasZalo') 'Thiếu cờ HasZalo'

# P3 không lọc đầu ra vssadmin theo từ khóa tiếng Anh
Assert 'Không lọc vssadmin theo từ khóa tiếng Anh' (-not ($srcTxt -match "match 'volume\|space'")) `
    'Vẫn lọc theo từ tiếng Anh, sẽ trắng màn hình trên Windows ngôn ngữ khác'

# P4 P5 P6 nhận biết môi trường
Assert 'Đặt bảng mã console về UTF-8' ($srcTxt -match 'chcp 65001') 'Không đặt bảng mã'
Assert 'Phát hiện hỗ trợ đường dẫn dài' ($srcTxt -match 'LongPathsEnabled') 'Không phát hiện long path'
Assert 'Phát hiện Controlled Folder Access' ($srcTxt -match 'EnableControlledFolderAccess') 'Không phát hiện CFA'

# P5 hàm dựng đường dẫn dài
$script:LongPathOK = $false
$longName = 'C:\' + ('x' * 250) + '\a.txt'
Assert 'Get-LongPath thêm tiền tố khi đường dẫn quá dài' ((Get-LongPath $longName) -eq ('\\?\' + $longName)) `
    ("Nhận được: " + (Get-LongPath $longName).Substring(0, 12))
Assert 'Get-LongPath không đụng đường dẫn ngắn' ((Get-LongPath 'C:\a\b.txt') -eq 'C:\a\b.txt') 'Sai'
$script:LongPathOK = $true
Assert 'Get-LongPath không thêm gì khi Windows đã bật long path' ((Get-LongPath $longName) -eq $longName) 'Sai'

# P7 danh mục nạp từ tệp ngoài
Assert 'Có tệp catalog.json cạnh script' (Test-Path $script:CatalogFile) 'Thiếu catalog.json'
$catFromFile = Get-CatalogDefs
$catBuiltin = Get-CatalogDefaults
Assert 'Danh mục nạp được từ catalog.json' ($catFromFile.Count -gt $catBuiltin.Count) `
    ("Từ tệp {0} mục, dựng sẵn {1} mục" -f $catFromFile.Count, $catBuiltin.Count)
Assert 'Danh mục ngoài có đánh dấu mục chưa kiểm chứng' (@($catFromFile | Where-Object { $_.ContainsKey('V') -and -not $_.V }).Count -gt 0) `
    'Không mục nào được đánh dấu chưa kiểm chứng'
$badPath = @()
foreach ($d in $catFromFile) {
    foreach ($p in $d.P) {
        if ($p -match '%') { $badPath += $p }
        if ($p -match '^[A-Za-z]:\\?$') { $badPath += $p }
    }
}
Assert 'Mọi đường dẫn trong catalog.json đã khai triển biến môi trường' ($badPath.Count -eq 0) (($badPath -join '; '))

# Màn hình danh mục phải có đủ lệnh chọn tất cả và bỏ chọn tất cả.
# Từ v5, dấu * chọn mọi mục TRỪ mục mang cảnh báo — chi tiết ở nhóm test catalog.json.
Assert 'Danh mục có lệnh chọn tất cả' ($srcTxt -match "if \(\`$raw -eq '\*'\) \{") 'Thiếu xử lý dấu *'
Assert 'Danh mục gợi ý lệnh chọn tất cả trên màn hình' ($srcTxt -match 'Gõ  \*   chọn tất cả') 'Không gợi ý cho người dùng'
Assert 'Danh mục vẫn có lệnh bỏ chọn tất cả' ($srcTxt -match 'Gõ  -   bỏ chọn tất cả') 'Mất lệnh bỏ chọn'
Assert 'Cảnh báo khi chọn mục chưa kiểm chứng' ($srcTxt -match 'chưa kiểm chứng tận nơi:') 'Không cảnh báo'

# Bộ đọc số của chính bộ test phải độc lập vùng miền
Assert 'Đọc được số kiểu en-US (20,000)'  ((Get-ReportedCount 'Đã xóa       : 20,000 tệp' 'Đã xóa') -eq 20000) 'Sai'
Assert 'Đọc được số kiểu vi-VN (20.000)'  ((Get-ReportedCount 'Đã xóa       : 20.000 tệp' 'Đã xóa') -eq 20000) 'Sai'
Assert 'Đọc được số có dấu cách (20 000)' ((Get-ReportedCount 'Đã xóa       : 20 000 tệp' 'Đã xóa') -eq 20000) 'Sai'
Assert 'Đọc được số không phân cách'      ((Get-ReportedCount 'Đã xóa       : 7 tệp' 'Đã xóa') -eq 7) 'Sai'
Assert 'Trả về -1 khi không có số'        ((Get-ReportedCount 'không có gì ở đây' 'Đã xóa') -eq -1) 'Sai'

# Chạy thật dưới vùng miền vi-VN để chắc công cụ lẫn bộ test đều không phụ thuộc
$rCul = New-Sandbox 'culture'
1..3 | ForEach-Object { New-TestFile (Join-Path $rCul "video\c$_") (New-Object byte[] 1024) $old }
$runner = Join-Path $sbRoot 'chay_vi_vn.ps1'
$runnerSrc = @'
param($toolPath, $rootPath)
[Threading.Thread]::CurrentThread.CurrentCulture = New-Object Globalization.CultureInfo 'vi-VN'
[Threading.Thread]::CurrentThread.CurrentUICulture = New-Object Globalization.CultureInfo 'vi-VN'
& $toolPath -Root $rootPath
'@
[IO.File]::WriteAllText($runner, $runnerSrc, (New-Object Text.UTF8Encoding $true))
$keysCul = (@('9', '7', '', 'X', '2', 'XÓA', '', '', '0') -join "`r`n") + "`r`n"
$oCul = ($keysCul | powershell.exe -NoProfile -ExecutionPolicy Bypass -File $runner $tool $rCul 2>&1 | Out-String -Width 200)
Assert 'Công cụ chạy đúng dưới vùng miền vi-VN' ((Get-ReportedCount $oCul 'Đã xóa') -eq 3) `
    ("Công cụ báo " + (Get-ReportedCount $oCul 'Đã xóa') + " tệp")
Assert 'Sandbox rỗng sau khi chạy dưới vi-VN' (@(Get-ChildItem $rCul -Recurse -File -Force -EA SilentlyContinue).Count -eq 0) 'Còn sót tệp'

# P7 hỏng tệp thì quay về danh mục dựng sẵn.
# Phép thử này cố tình làm hỏng catalog.json thật, nên phải trả lại bằng
# finally: đứt gánh giữa chừng mà không trả thì người dùng ở lại với một
# catalog.json hỏng do chính bộ test gây ra.
$catBak = Backup-RealFile $script:CatalogFile
try {
    [IO.File]::WriteAllText($script:CatalogFile, '{ khong phai json hop le',
                            (New-Object Text.UTF8Encoding $true))
    $catFallback = Get-CatalogDefs
    Assert 'catalog.json hỏng thì quay về danh mục dựng sẵn' ($catFallback.Count -eq $catBuiltin.Count) `
        ("Nhận được " + $catFallback.Count + " mục")
} finally {
    Restore-RealFile $script:CatalogFile $catBak
}

# ---------------------------------------------------------------- khôi phục tự tìm
Write-Host ''
Write-Host '── Khôi phục tự tìm bản sao lưu và mô tả nội dung' -ForegroundColor Yellow
$setFile0 = Join-Path $PSScriptRoot 'settings.json'
$setBak0 = Backup-RealFile $setFile0

$bkParent = Join-Path $sbRoot 'kholuu'
$bkSet = Join-Path $bkParent '20260801_090000'
New-Item -ItemType Directory -Force (Join-Path $bkSet 'video') | Out-Null
1..2 | ForEach-Object { New-TestFile (Join-Path $bkSet "video\v$_") (New-Object byte[] 200000) ([datetime]'2025-04-01') }
New-TestFile (Join-Path $bkSet 'picture\p1.jxl') (New-Object byte[] 50000) ([datetime]'2025-11-20')
$rTarget = New-Sandbox 'khoiphuc'
@{ Tool='ZaloCleanup'; Version=4; Created='01/08/2026 09:00:00'; SourceRoot=$rTarget
   ScanKind='DỮ LIỆU ZALO'; Count=3; Bytes=450000; FullVerify=$false; Verified=3; VerifyFail=0; CopyFail=0 } |
  ConvertTo-Json | Set-Content (Join-Path $bkSet '_zalocleanup_backup.json') -Encoding UTF8

# ghi nhớ thư mục sao lưu vào settings để công cụ tự tìm ra
(@{ BackupPolicy = 'HOI'; BackupRoots = @($bkParent) } | ConvertTo-Json) | Set-Content $setFile0 -Encoding UTF8

$o = Invoke-Tool $rTarget @('3', '', '', '0')
Assert 'Khôi phục tự tìm ra bản sao lưu, không hỏi đường dẫn' ($o -match 'Tìm thấy 1 bản sao lưu') 'Không tự tìm được'
Assert 'Hiện nội dung gồm những thư mục nào' ($o -match 'Gồm\s+:.*video') 'Thiếu mô tả nội dung'
Assert 'Hiện loại tệp bên trong' ($o -match 'Loại tệp\s+:') 'Thiếu loại tệp'
Assert 'Hiện khoảng thời gian của tệp' ($o -match 'Tệp từ\s+:.*2025') 'Thiếu khoảng thời gian'
Assert 'Hiện nơi bản sao lưu đang nằm' ($o -match 'Nằm ở\s+:') 'Thiếu vị trí'

$o = Invoke-Tool $rTarget @('3', 'x 1', '', '', '0')
Assert 'Xem được danh sách tệp lớn nhất bên trong' ($o -match 'Ba tệp lớn nhất') 'Không xem được chi tiết'

Restore-RealFile $setFile0 $setBak0

# ---------------------------------------------------------------- chính sách sao lưu
Write-Host ''
Write-Host '── Chính sách sao lưu: sao lưu là tùy chọn' -ForegroundColor Yellow
$setFile = Join-Path $PSScriptRoot 'settings.json'
$setBak = Backup-RealFile $setFile

function New-PolicySandbox($name) {
    $p = New-Sandbox $name
    1..2 | ForEach-Object { New-TestFile (Join-Path $p "video\p$_") (New-Object byte[] 1024) $old }
    return $p
}

'{"BackupPolicy":"HOI"}' | Set-Content $setFile -Encoding UTF8
$rp = New-PolicySandbox 'polask'
$o = Invoke-Tool $rp @('9', '7', '', 'X', '2', 'XÓA', '', '', '0')
Assert 'HOI hỏi trước khi xóa' ($o -match 'chưa được sao lưu') 'Không hỏi về sao lưu'
Assert 'HOI cho phép xóa không sao lưu' ((Get-ReportedCount $o 'Đã xóa') -eq 2) `
    ("Công cụ báo " + (Get-ReportedCount $o 'Đã xóa') + " tệp")

$rp2 = New-PolicySandbox 'polcancel'
$o = Invoke-Tool $rp2 @('9', '7', '', 'X', '', '', '', '0')
Assert 'HOI hủy thì không xóa gì' (@(Get-ChildItem $rp2 -Recurse -File -Force -EA SilentlyContinue).Count -eq 2) 'Đã xóa dù chọn hủy'

'{"BackupPolicy":"BATBUOC"}' | Set-Content $setFile -Encoding UTF8
$rp3 = New-PolicySandbox 'polreq'
$o = Invoke-Tool $rp3 @('9', '7', '', 'X', 'XÓA', '', '', '0')
Assert 'BATBUOC chặn xóa khi chưa sao lưu' ($o -match 'bắt buộc sao lưu') 'Không chặn'
Assert 'BATBUOC không xóa tệp nào' (@(Get-ChildItem $rp3 -Recurse -File -Force -EA SilentlyContinue).Count -eq 2) 'Đã xóa dù bị chặn'

'{"BackupPolicy":"KHONG"}' | Set-Content $setFile -Encoding UTF8
$rp4 = New-PolicySandbox 'polnever'
$o = Invoke-Tool $rp4 @('9', '7', '', 'X', 'XÓA', '', '', '0')
Assert 'KHONG không hỏi về sao lưu' (-not ($o -match 'chưa được sao lưu')) 'Vẫn hỏi'
Assert 'KHONG xóa thẳng sau khi gõ XÓA' (@(Get-ChildItem $rp4 -Recurse -File -Force -EA SilentlyContinue).Count -eq 0) 'Không xóa được'

Restore-RealFile $setFile $setBak

# ---------------------------------------------------------------- phép thử chậm
if ($Full) {
    Write-Host ''
    Write-Host '── G2: sao lưu lỗi thì chặn xóa (chậm)' -ForegroundColor Yellow
    $r6 = New-Sandbox 'g2'
    1..4 | ForEach-Object { New-TestFile (Join-Path $r6 "video\f$_") (New-Object byte[] 1024) $old }
    $lock = Join-Path $r6 'video\f3'
    $job = Start-Job -ScriptBlock { param($p) $s = [IO.File]::Open($p, 'Open', 'Read', 'None'); Start-Sleep -Seconds 60; $s.Close() } -ArgumentList $lock
    Start-Sleep -Seconds 3
    $o = Invoke-Tool $r6 @('9', '7', '', '9', (Join-Path $sbRoot 'bk2'), '', '', 'X', 'XÓA', '', '', '0')
    Stop-Job $job -EA SilentlyContinue; Remove-Job $job -Force -EA SilentlyContinue
    Assert 'G2 sao lưu ghi nhận thất bại' ($o -match 'Chép lỗi') 'Không ghi nhận lỗi chép'
    Assert 'G2 chặn bước xóa' ($o -match 'sao lưu chưa sạch') 'Không chặn xóa'
    Assert 'G2 không tệp nào bị xóa' (@(Get-ChildItem $r6 -Recurse -File -Force -EA SilentlyContinue).Count -eq 4) 'Đã xóa dù bị chặn'

    Write-Host ''
    Write-Host '── G4: tệp biến mất giữa chừng (chậm)' -ForegroundColor Yellow
    $r7 = New-Sandbox 'race'
    $dir = Join-Path $r7 'video'
    New-Item -ItemType Directory -Force $dir | Out-Null
    $buf = New-Object byte[] 512
    for ($i = 0; $i -lt 20000; $i++) { [IO.File]::WriteAllBytes((Join-Path $dir ('f{0:D5}' -f $i)), $buf) }
    Get-ChildItem $dir -File | ForEach-Object { $_.LastWriteTime = $old }
    $sab = Start-Job -ScriptBlock {
        param($d) Start-Sleep -Seconds 5
        for ($i = 16000; $i -lt 20000; $i++) { try { [IO.File]::Delete((Join-Path $d ('f{0:D5}' -f $i))) } catch { } }
    } -ArgumentList $dir
    $o = Invoke-Tool $r7 @('9', '7', '', 'X', '2', 'XÓA', '', '', '0')
    Wait-Job $sab | Out-Null; Remove-Job $sab -Force
    Assert 'G4 báo cáo số tệp biến mất trước' ($o -match 'Biến mất trước khi xóa') 'Không báo cáo'
    $daXoa = Get-ReportedCount $o 'Đã xóa'
    Assert 'G4 không đếm nhầm tệp biến mất là đã xóa' ($daXoa -gt 0 -and $daXoa -lt 20000) `
        ("Công cụ báo $daXoa tệp, đáng lẽ phải nhỏ hơn 20000")
}

# ---------------------------------------------------------------- kết quả
Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
$script:Results | Format-Table -AutoSize | Out-String -Width 120 | Write-Host
Write-Host ("  ĐẠT: {0}    HỎNG: {1}" -f $script:Pass, $script:Fail) -ForegroundColor $(if ($script:Fail -eq 0) { 'Green' } else { 'Red' })
if (-not $Full) { Write-Host '  (Chạy lại với -Full để thêm các phép thử chậm)' -ForegroundColor DarkGray }
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan

Remove-Sandbox $sbRoot
Get-ChildItem $logDir -File -ErrorAction SilentlyContinue |
    Where-Object { $_.LastWriteTime -gt (Get-Date).AddMinutes(-30) } |
    Remove-Item -Force -ErrorAction SilentlyContinue

if ($script:Fail -gt 0) { exit 1 }
exit 0
