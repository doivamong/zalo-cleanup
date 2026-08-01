#Requires -Version 5.1
<#
    DỌN DẸP ZALO  v5
    ================
    Công cụ dọn dẹp dữ liệu thủ công cho Windows.

    Công cụ này KHÔNG tự chạy. Không đăng ký Scheduled Task, không hook,
    không chạy nền. Nó chỉ làm việc khi bạn mở và bấm lệnh.

    Năm nguyên tắc bất biến:
      1. Không quét thì không thể xóa.
      2. Đổi bộ lọc là kết quả quét cũ bị hủy.
      3. Nhập sai thì giữ nguyên, không bao giờ tự mở rộng phạm vi.
      4. Vùng bảo vệ bị chặn cứng ở tầng code.
      5. Sao lưu chưa sạch thì không cho xóa.

    LƯU Ý CHO NGƯỜI SỬA FILE NÀY: phải lưu dạng UTF-8 CÓ BOM.
    Thiếu BOM thì PowerShell 5.1 đọc theo bảng mã ANSI và mọi chữ có dấu sẽ vỡ.

    Xem README.md để biết chi tiết.
#>

param(
    [string]$Root = '',
    [string]$DataRoot = ''
)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }
try { $Host.UI.RawUI.WindowTitle = 'Dọn dẹp Zalo v5' } catch { }

# ================================================================ trạng thái
$script:FromDate    = $null
# Mặc định không giới hạn hai đầu: công cụ không được tự thu hẹp phạm vi quét
# thay bạn. Muốn nhắm tệp cũ thì chọn mốc trong Set-DateRange hoặc dùng luồng
# hướng dẫn ở màn hình chính.
$script:ToDate      = $null
$script:Folders     = @()
$script:Exts        = @()
$script:ExFolders   = @()
$script:ExExts      = @()
$script:MinSizeKB   = 0
$script:KeepRescache = $true

$script:Scan        = $null
$script:ScanStamp   = ''
$script:ScanKind    = ''
$script:ScanRoot    = ''
$script:LastBackup  = $null

$script:TreeCache   = $null
$script:TreeScanSec = $null      # lần đo trước mất bao lâu, để quyết định có tự đo lại không
$script:LastScanErrors = 0

$script:Root        = $Root
$script:DataRoot    = $DataRoot

$script:ProtectedNames = @('Database', 'Partitions')
# Mỗi luật là @{ Path = <đường dẫn tuyệt đối>; Depth = 'any' | 0 }
#   'any' — chặn chính nó và mọi thứ bên dưới. Đây là mức của toàn bộ luật cũ.
#   0     — chỉ chặn khi nhắm thẳng vào chính thư mục đó; con vẫn cho phép.
#           Dùng cho các gốc lớn như %LOCALAPPDATA%, nơi mà %LOCALAPPDATA%\npm-cache
#           là mục hợp lệ nhưng bản thân %LOCALAPPDATA% thì không bao giờ được nhắm tới.
$script:ProtectedRules = @()
$script:ProtectedAbs   = @()     # chỉ các luật Depth='any', tách sẵn cho đường nhanh
$script:CleanupRoots   = @()

$script:AppCacheRel = @('Cache', 'Code Cache', 'GPUCache', 'DawnGraphiteCache',
                        'DawnWebGPUCache', 'ShaderCache', 'media\update', 'media\temp')
$script:StandaloneDirs = @('video', 'picture', 'voice', 'file')

# Số luồng đọc đĩa cho bước băm nội dung.
# Đo trên máy thật (SSD SATA, tệp chưa nằm trong bộ nhớ đệm): một luồng được
# 41 MB/s, tám luồng được 53 MB/s. Đĩa mới là trần chứ không phải CPU — SHA256
# của .NET chạy 840 MB/s khi dữ liệu đã ở trong RAM. Song song ở đây là để vắt
# thêm hàng đợi của SSD, không phải để bắt CPU tính nhanh hơn. Quá tám luồng
# không đo được cải thiện nào nữa.
$script:HashThreads = [Math]::Min(8, [Math]::Max(1, [Environment]::ProcessorCount))

# Chính sách sao lưu. Sao lưu KHÔNG bắt buộc.
#   HOI     = mỗi lần xóa dữ liệu thật sẽ hỏi (mặc định)
#   KHONG   = không hỏi
#   BATBUOC = phải có bản sao lưu sạch mới cho xóa
$script:BackupPolicy = 'HOI'

# Các thư mục đã từng dùng để sao lưu. Ghi nhớ để lần sau khỏi phải nhớ đường dẫn.
$script:BackupRoots = @()

$script:ToolDir      = $PSScriptRoot
$script:LogDir       = Join-Path $script:ToolDir 'logs'
$script:ProfileFile  = Join-Path $script:ToolDir 'profiles.json'
$script:SettingsFile = Join-Path $script:ToolDir 'settings.json'
$script:CatalogFile  = Join-Path $script:ToolDir 'catalog.json'
if (-not (Test-Path -LiteralPath $script:LogDir)) {
    New-Item -ItemType Directory -Force -Path $script:LogDir | Out-Null
}

# ---------------------------------------------------------------- môi trường máy
# Không giả định Windows nằm ở ổ C. Không giả định máy có cài Zalo.
$script:SysDrive  = $env:SystemDrive                 # ví dụ 'C:'
if ([string]::IsNullOrWhiteSpace($script:SysDrive)) { $script:SysDrive = 'C:' }
$script:SysRoot   = $script:SysDrive + '\'           # ví dụ 'C:\'
$script:HasZalo   = $false                           # đặt lại sau khi dò tìm
$script:LongPathOK = $true                           # đặt lại trong Initialize-Environment
$script:CfaOn      = $false
$script:OldCodePage = $null

# ================================================================ tiện ích
function Write-Title($text) {
    Write-Host ''
    Write-Host ('─' * 62) -ForegroundColor DarkCyan
    Write-Host "  $text" -ForegroundColor Cyan
    Write-Host ('─' * 62) -ForegroundColor DarkCyan
}

function Show-DateLabel($d) {
    if ($null -eq $d) { return 'không giới hạn' }
    return $d.ToString('dd/MM/yyyy')
}

# Nhãn gộp cho cả khoảng. Không có mốc nào thì nói thẳng là mọi thời điểm,
# đọc dễ hơn nhiều so với 'không giới hạn → không giới hạn'.
function Show-DateRangeLabel($from, $to) {
    if ($null -eq $from -and $null -eq $to) { return 'mọi thời điểm' }
    if ($null -eq $from) { return 'đến ' + (Show-DateLabel $to) }
    if ($null -eq $to)   { return 'từ ' + (Show-DateLabel $from) }
    return (Show-DateLabel $from) + ' → ' + (Show-DateLabel $to)
}

function Show-Size([long]$bytes) {
    if ($bytes -ge 1GB) { return ('{0:N2} GB' -f ($bytes / 1GB)) }
    if ($bytes -ge 1MB) { return ('{0:N1} MB' -f ($bytes / 1MB)) }
    if ($bytes -ge 1KB) { return ('{0:N0} KB' -f ($bytes / 1KB)) }
    return "$bytes B"
}

function Get-RelPath($full, $base) {
    if ([string]::IsNullOrWhiteSpace($base)) { return $full }
    $b = $base
    if (-not $b.EndsWith('\')) { $b = $b + '\' }
    if ($full.StartsWith($b, [StringComparison]::OrdinalIgnoreCase)) {
        return $full.Substring($b.Length)
    }
    return $full
}

# Read-Host an toàn.
# Người dùng bấm Enter -> trả về chuỗi rỗng. Luồng nhập đã cạn -> trả về $null.
# Phải phân biệt, nếu không vòng lặp menu sẽ quay vô tận khi chạy phi tương tác.
$script:EofStreak = 0
function Read-Line($prompt) {
    $v = Read-Host $prompt
    if ($null -eq $v) {
        $script:EofStreak++
        if ($script:EofStreak -ge 5) {
            Write-Host ''
            Write-Host '  Luồng nhập đã cạn. Thoát.' -ForegroundColor DarkGray
            try { Restore-Environment } catch { }
            exit 0
        }
        return ''
    }
    $script:EofStreak = 0
    return $v.Trim()
}

function Read-DateValue($prompt) {
    while ($true) {
        $raw = Read-Line $prompt
        if ($raw -eq '') { return $null }
        foreach ($f in @('dd/MM/yyyy', 'd/M/yyyy', 'yyyy-MM-dd', 'ddMMyyyy')) {
            $out = [datetime]::MinValue
            $ok = [datetime]::TryParseExact($raw, $f, [Globalization.CultureInfo]::InvariantCulture,
                                            [Globalization.DateTimeStyles]::None, [ref]$out)
            if ($ok) { return $out }
        }
        Write-Host '  Không hiểu định dạng. Ví dụ hợp lệ: 31/12/2025 hoặc 2025-12-31' -ForegroundColor Yellow
    }
}

function Read-YesNo($prompt) {
    $v = (Read-Line $prompt).ToLower()
    return ($v -eq 'c' -or $v -eq 'có' -or $v -eq 'co' -or $v -eq 'y')
}

# Câu xác nhận chấp nhận cả bản có dấu lẫn không dấu. Người dùng có thể gõ
# thiếu dấu, và một số môi trường dòng lệnh làm hỏng ký tự có dấu khi truyền
# qua đường ống.
# Bỏ dấu thanh khỏi chuỗi, giữ nguyên chữ hoa chữ thường.
# Chữ Đ và đ KHÔNG phải chữ có dấu thanh mà là chữ cái riêng, nên không bị đụng
# tới — đúng như mong muốn, vì không cụm xác nhận nào của công cụ chứa nó.
function Remove-ToneMarks($s) {
    if ([string]::IsNullOrEmpty($s)) { return $s }
    $d = $s.Normalize([Text.NormalizationForm]::FormD)
    $sb = New-Object Text.StringBuilder
    foreach ($ch in $d.ToCharArray()) {
        if ([Globalization.CharUnicodeInfo]::GetUnicodeCategory($ch) -ne
            [Globalization.UnicodeCategory]::NonSpacingMark) { [void]$sb.Append($ch) }
    }
    return $sb.ToString().Normalize([Text.NormalizationForm]::FormC)
}

# Tiếng Việt có hai kiểu đặt dấu đều đúng chính tả cho cùng một chữ: XÓA và XOÁ.
# Bộ gõ đặt dấu kiểu nào là do người dùng chọn chứ không phải do họ gõ sai. Bản
# cũ chỉ nhận đúng XÓA, nên người dùng bộ gõ đặt dấu kiểu mới gõ XOÁ thì bị từ
# chối, và kết luận hợp lý nhất của họ là công cụ hỏng.
#
# Cách sửa: bỏ dấu rồi so với dạng không dấu. Nhận mọi kiểu đặt dấu, nhưng VẪN
# so phân biệt hoa thường, nên chữ thường vẫn bị từ chối — ma sát của bước xác
# nhận không hề bị nới ra, chỉ hết bắt bẻ chuyện đặt dấu ở đâu.
function Test-ConfirmPhrase($answer, $withMarks, $plain) {
    if ($answer -ceq $withMarks -or $answer -ceq $plain) { return $true }
    return ((Remove-ToneMarks $answer) -ceq $plain)
}

function Reset-Scan {
    $script:Scan = $null
    $script:ScanStamp = ''
    $script:ScanKind = ''
    $script:ScanRoot = ''
    $script:LastBackup = $null
}

function Reset-TreeCache { $script:TreeCache = $null }

# Dung lượng trống THẬT SỰ của ổ đĩa. Đây mới là con số quan trọng:
# số byte file bị xóa chỉ là kế toán trên giấy tờ, nếu Shadow Copy hấp thụ
# thì ổ đĩa không hề rộng ra.
function Get-FreeBytes($path) {
    try {
        $qual = [IO.Path]::GetPathRoot([IO.Path]::GetFullPath($path))
        $dl = $qual.TrimEnd('\', '/').TrimEnd(':')
        return (Get-PSDrive -Name $dl -ErrorAction Stop).Free
    } catch { return -1 }
}

# Tên ổ đĩa để hiển thị, ví dụ 'C'.
function Get-DriveLabel($path) {
    try { return ([IO.Path]::GetPathRoot([IO.Path]::GetFullPath($path))).TrimEnd('\', '/').TrimEnd(':') }
    catch { return $script:SysDrive.TrimEnd(':') }
}

# Đường dẫn dài quá 260 ký tự cần tiền tố đặc biệt, trừ khi Windows đã bật
# hỗ trợ long path. Mặc định của Windows là TẮT.
function Get-LongPath($path) {
    if ($script:LongPathOK) { return $path }
    if ($path.Length -lt 240) { return $path }
    if ($path.StartsWith('\\?\')) { return $path }
    if ($path.StartsWith('\\')) { return '\\?\UNC\' + $path.Substring(2) }
    return '\\?\' + $path
}

function Initialize-Environment {
    # Bảng mã console: tiếng Việt cần UTF-8. Ghi lại bảng mã cũ để trả về khi thoát.
    try {
        $cur = (& chcp) -replace '[^\d]', ''
        if ($cur -ne '65001') {
            $script:OldCodePage = $cur
            & chcp 65001 | Out-Null
        }
    } catch { }
    try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

    # Hỗ trợ đường dẫn dài
    try {
        $lp = Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem' -Name LongPathsEnabled -ErrorAction Stop
        $script:LongPathOK = ([int]$lp.LongPathsEnabled -eq 1)
    } catch { $script:LongPathOK = $false }

    # Controlled Folder Access của Windows Defender có thể chặn xóa
    try {
        $mp = Get-MpPreference -ErrorAction Stop
        $script:CfaOn = ([int]$mp.EnableControlledFolderAccess -ne 0)
    } catch { $script:CfaOn = $false }
}

function Restore-Environment {
    if ($null -ne $script:OldCodePage) {
        try { & chcp $script:OldCodePage | Out-Null } catch { }
    }
}

function Show-EnvironmentWarnings {
    if (-not $script:LongPathOK) {
        Write-Host '  Lưu ý: Windows đang tắt hỗ trợ đường dẫn dài. Công cụ tự dùng tiền tố' -ForegroundColor DarkGray
        Write-Host '  đặc biệt cho đường dẫn trên 240 ký tự nên vẫn xóa được.' -ForegroundColor DarkGray
    }
    if ($script:CfaOn) {
        Write-Host '  Cảnh báo: Controlled Folder Access của Windows Defender đang bật.' -ForegroundColor Yellow
        Write-Host '  Một số thao tác xóa có thể bị chặn và báo thất bại.' -ForegroundColor Yellow
    }
}

function Read-Settings {
    if (-not (Test-Path -LiteralPath $script:SettingsFile)) { return }
    try {
        $s = Get-Content -LiteralPath $script:SettingsFile -Raw | ConvertFrom-Json
        if ($s.BackupPolicy -in @('HOI', 'KHONG', 'BATBUOC')) { $script:BackupPolicy = $s.BackupPolicy }
        if ($s.PSObject.Properties['BackupRoots']) { $script:BackupRoots = @($s.BackupRoots) }
    } catch { }
}

function Save-Settings {
    try {
        [pscustomobject]@{ BackupPolicy = $script:BackupPolicy; BackupRoots = @($script:BackupRoots) } |
            ConvertTo-Json | Set-Content -LiteralPath $script:SettingsFile -Encoding UTF8
    } catch { }
}

function Register-BackupRoot($path) {
    if ([string]::IsNullOrWhiteSpace($path)) { return }
    if ($script:BackupRoots -notcontains $path) {
        $script:BackupRoots = @($script:BackupRoots) + $path
        Save-Settings
    }
}

function Show-PolicyLabel {
    switch ($script:BackupPolicy) {
        'HOI'     { return 'Hỏi mỗi lần xóa dữ liệu thật' }
        'KHONG'   { return 'Không hỏi' }
        'BATBUOC' { return 'Bắt buộc sao lưu sạch mới cho xóa' }
    }
    return $script:BackupPolicy
}

# ================================================================ vùng bảo vệ
# Hàm này chạy ba lần cho MỖI tệp — 149 nghìn tệp ở lần chạy thật — nên chỉ
# hỏi đúng một câu rẻ nhất: đường dẫn có nằm TRONG vùng bảo vệ không.
# Chiều ngược lại (đường dẫn là CHA của vùng bảo vệ) do Test-ProtectedRoot lo,
# và chỉ được hỏi cho vài chục thư mục gốc.
# Chỉ mục tra cứu dựng sẵn cho Test-Protected. Ngữ nghĩa chặn KHÔNG đổi một ly,
# chỉ đổi cách tra:
#   ProtectedExact  — đường dẫn mà chính nó bị chặn (phép so bằng của bản cũ)
#   ProtectedPrefix — thư mục mà mọi thứ nằm dưới đều bị chặn (phép so tiền tố)
# Luật Depth=0 chỉ vào ProtectedExact, đúng như bản cũ chỉ so bằng cho chúng
# và không bao giờ so tiền tố.
#
# Bắt buộc dùng StringComparer::OrdinalIgnoreCase cho cả hai. Bảng băm mặc định
# của PowerShell so theo vùng miền hiện hành, mà công cụ có chạy dưới vi-VN —
# so theo vùng miền là cách để một đường dẫn lọt lưới ở máy này mà không lọt ở
# máy khác.
$script:ProtectedExact      = $null
$script:ProtectedPrefix     = $null
$script:ProtectedDirCache   = $null
$script:ProtectedIndexFor   = $null   # DataRoot tại lúc dựng chỉ mục
$script:ProtectedIndexRules = $null   # tham chiếu mảng luật tại lúc dựng chỉ mục

function Build-ProtectedIndex {
    $exact  = New-Object 'Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)
    $prefix = New-Object 'Collections.Generic.List[string]'

    foreach ($p in $script:ProtectedAbs) {
        [void]$exact.Add($p)
        $prefix.Add($p)
    }
    foreach ($r in $script:ProtectedRules) {
        if ($r.Depth -eq 'any') { continue }   # đã nạp ở vòng trên
        [void]$exact.Add($r.Path)
    }
    if (-not [string]::IsNullOrWhiteSpace($script:DataRoot)) {
        foreach ($n in $script:ProtectedNames) {
            $p = (Join-Path $script:DataRoot $n)
            [void]$exact.Add($p)
            $prefix.Add($p)
        }
    }

    $script:ProtectedExact      = $exact
    $script:ProtectedPrefix     = $prefix
    $script:ProtectedDirCache   = New-Object 'Collections.Generic.Dictionary[string,bool]' ([StringComparer]::OrdinalIgnoreCase)
    $script:ProtectedIndexFor   = $script:DataRoot
    $script:ProtectedIndexRules = $script:ProtectedRules
}

function Test-Protected($path) {
    # Tự dựng lại khi luật hoặc DataRoot đổi — đổi tài khoản bằng phím T là đổi
    # DataRoot. Không trông vào việc nhớ gọi hàm dựng lại ở đúng chỗ: quên một
    # chỗ ở đây nghĩa là mất một lớp chặn, nên hàm tự kiểm lấy.
    if ($null -eq $script:ProtectedExact -or
        $script:ProtectedIndexFor -ne $script:DataRoot -or
        -not [object]::ReferenceEquals($script:ProtectedIndexRules, $script:ProtectedRules)) {
        Build-ProtectedIndex
    }

    if ($script:ProtectedExact.Contains($path)) { return $true }

    # Nằm dưới vùng bảo vệ hay không chỉ phụ thuộc thư mục chứa nó, mà hàng vạn
    # tệp dùng chung một thư mục — nên nhớ kết quả theo thư mục thay vì hỏi lại
    # từng tệp.
    #
    # Cắt bằng LastIndexOf chứ không dùng [IO.Path]::GetDirectoryName: hàm kia
    # ném lỗi khi đường dẫn có ký tự lạ, mà một hàm canh cửa thì không được phép
    # ném — ném ở đây là bỏ trống cửa.
    $i = $path.LastIndexOf('\')
    if ($i -lt 0) { return $false }
    $dir = $path.Substring(0, $i)

    if ($script:ProtectedDirCache.ContainsKey($dir)) { return $script:ProtectedDirCache[$dir] }

    $hit = $false
    foreach ($p in $script:ProtectedPrefix) {
        if ($dir.Equals($p, [StringComparison]::OrdinalIgnoreCase) -or
            $dir.StartsWith($p + '\', [StringComparison]::OrdinalIgnoreCase)) {
            $hit = $true
            break
        }
    }
    $script:ProtectedDirCache[$dir] = $hit
    return $hit
}

# Dùng cho THƯ MỤC GỐC: gốc quét, gốc dọn thư mục rỗng, đường dẫn trong catalog.json.
# Ngoài việc hỏi như Test-Protected, nó chặn thêm chiều ngược — nhận một thư mục
# CHỨA vùng bảo vệ cũng nguy hiểm y như nhận chính vùng bảo vệ. catalog.json là
# tệp người dùng sửa được, nên một mục ghi "%WINDIR%" phải bị chặn ngay ở đây.
function Test-ProtectedRoot($path) {
    if (Test-Protected $path) { return $true }
    $p = $path.TrimEnd('\')
    foreach ($r in $script:ProtectedRules) {
        if ($r.Path.StartsWith($p + '\', [StringComparison]::OrdinalIgnoreCase)) { return $true }
    }
    if ([string]::IsNullOrWhiteSpace($script:DataRoot)) { return $false }
    foreach ($n in $script:ProtectedNames) {
        $q = (Join-Path $script:DataRoot $n)
        if ($q.StartsWith($p + '\', [StringComparison]::OrdinalIgnoreCase)) { return $true }
    }
    return $false
}

function Initialize-ProtectedAbs {
    $w = $env:WINDIR; $u = $env:USERPROFILE; $l = $env:LOCALAPPDATA; $a = $env:APPDATA
    $pf = ${env:ProgramFiles}; $px = ${env:ProgramFiles(x86)}; $pd = $env:ProgramData

    # Mức 'any' — giữ nguyên toàn bộ luật đã có từ v4, hành vi không đổi.
    $anyPaths = @(
        (Join-Path $w 'WinSxS'), (Join-Path $w 'Installer'), (Join-Path $w 'System32'),
        (Join-Path $w 'SysWOW64'), (Join-Path $w 'servicing'), (Join-Path $w 'assembly'),
        (Join-Path $script:SysRoot 'hiberfil.sys'),
        (Join-Path $script:SysRoot 'pagefile.sys'),
        (Join-Path $script:SysRoot 'swapfile.sys'),
        (Join-Path $a 'Claude\vm_bundles'),
        (Join-Path $u '.cargo\bin'), (Join-Path $u '.rustup'),
        (Join-Path $l 'Programs'), (Join-Path $l 'Packages'),
        $script:ToolDir
    ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    # Mức 0 — chỉ chặn khi nhắm thẳng vào chính thư mục. Đây là lưới chắn cho
    # catalog.json bị sửa sai: "%LOCALAPPDATA%" bị chặn, "%LOCALAPPDATA%\npm-cache" thì không.
    $rootPaths = @($w, $u, $l, $a, $pf, $px, $pd, $script:SysRoot) |
                 Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    $rules = @()
    foreach ($p in $anyPaths)  { $rules += @{ Path = $p.TrimEnd('\'); Depth = 'any' } }
    foreach ($p in $rootPaths) {
        $t = $p.TrimEnd('\')
        if ($t -match '^[A-Za-z]:$') { $t = $t + '\' }   # gốc ổ đĩa giữ dấu gạch chéo
        if (@($rules | Where-Object { $_.Path -eq $t }).Count -gt 0) { continue }
        $rules += @{ Path = $t; Depth = 0 }
    }
    $script:ProtectedRules = $rules
    $script:ProtectedAbs   = @($rules | Where-Object { $_.Depth -eq 'any' } | ForEach-Object { $_.Path })
}

# Dọn thư mục rỗng, giới hạn trong đúng các gốc được chỉ định.
# KHÔNG BAO GIỜ nhận gốc ổ đĩa, nếu không sẽ duyệt cả ổ C hai mươi lần.
#
# Hai điều tuyệt đối không làm ở đây:
#
#   1. Không dùng Remove-Item -Recurse. Giữa lúc kết luận "thư mục này rỗng" và
#      lúc ra lệnh xóa có một khe hở; tiến trình khác kịp ghi tệp vào đó thì
#      -Recurse xóa luôn tệp ấy mà không qua Test-Protected. [IO.Directory]::Delete
#      với recursive=$false ném lỗi khi thư mục hết rỗng — đúng thứ ta muốn.
#
#   2. Không đụng tới reparse point. Một junction bị chặn quyền đọc sẽ trả về
#      danh sách con rỗng, trông y hệt thư mục rỗng; xóa đệ quy lên nó có thể
#      xóa xuyên sang thư mục đích ở đầu bên kia.
function Test-IsReparsePoint($item) {
    if ($null -eq $item) { return $false }
    try { return [bool]($item.Attributes -band [IO.FileAttributes]::ReparsePoint) }
    catch { return $false }
}

# Tệp đang bị tiến trình khác giữ thì không xóa được, nhưng nếu tiến trình ấy mở
# ở chế độ chia sẻ thì vẫn ghi đè được — cắt về 0 byte là thu lại đủ dung lượng,
# chỉ còn sót cái tên rỗng mà chủ của nó sẽ tự dọn hoặc tự ghi lại.
#
# CHỈ dùng cho cache. Cắt cụt một tệp đang được dùng có thể làm hỏng ứng dụng
# đang giữ nó theo kiểu rất khó chẩn đoán, trong khi xóa thất bại thì vô hại.
# Với cache thì đánh đổi đó đáng; với dữ liệu Zalo thật thì không, huống hồ công
# cụ đã bắt đóng Zalo trước khi xóa nên tệp Zalo bị khóa là dấu hiệu bất thường
# đáng được báo THẤTBẠI để bạn nhìn thấy.
function Invoke-TruncateLocked($longPath) {
    try {
        $fs = [IO.File]::Open($longPath, [IO.FileMode]::Open, [IO.FileAccess]::Write, [IO.FileShare]::ReadWrite)
        try { $fs.SetLength(0) } finally { $fs.Dispose() }
        return $true
    } catch { return $false }
}

function Remove-EmptyDirs($roots, $preserveTopLevel) {
    $removed = 0
    foreach ($r in @($roots)) {
        if ([string]::IsNullOrWhiteSpace($r)) { continue }
        if ($r -match '^[A-Za-z]:\\?$') { continue }
        if (-not (Test-Path -LiteralPath $r)) { continue }
        if (Test-ProtectedRoot $r) { continue }
        if (Test-IsReparsePoint (Get-Item -LiteralPath $r -Force -ErrorAction SilentlyContinue)) { continue }
        $keep = @($r)
        if ($preserveTopLevel) {
            $keep += @(Get-ChildItem -LiteralPath $r -Directory -Force -ErrorAction SilentlyContinue |
                       Select-Object -ExpandProperty FullName)
        }
        for ($pass = 1; $pass -le 20; $pass++) {
            $empty = @(Get-ChildItem -LiteralPath $r -Recurse -Directory -Force -ErrorAction SilentlyContinue |
                       Where-Object { $keep -notcontains $_.FullName -and -not (Test-IsReparsePoint $_) -and
                                      -not (Test-Protected $_.FullName) -and
                                      -not (Get-ChildItem -LiteralPath $_.FullName -Force -ErrorAction SilentlyContinue) })
            if ($empty.Count -eq 0) { break }
            $gone = 0
            foreach ($d in $empty) {
                # recursive=$false — hết rỗng thì ném lỗi chứ không xóa mù.
                try { [IO.Directory]::Delete((Get-LongPath $d.FullName), $false); $removed++; $gone++ } catch { }
            }
            if ($gone -eq 0) { break }   # lượt này không xóa được gì thì lượt sau cũng vậy
        }
    }
    return $removed
}

# Duyệt cây thư mục bằng .NET thay vì Get-ChildItem -Recurse. Đo trên 56.914
# tệp thật: 3,40 giây xuống 1,12 giây. Phần lớn chỗ tiết kiệm là do không phải
# dựng đối tượng bọc của PowerShell cho từng tệp.
#
# Hai tính chất của bản cũ BẮT BUỘC phải giữ, vì mất một trong hai là hỏng:
#
#   1. Đếm được lỗi truy cập chứ không nuốt. Bản cũ đếm bằng -ErrorVariable;
#      ở đây bắt lỗi theo từng thư mục, lại còn đúng chỗ hơn.
#
#   2. KHÔNG đi xuyên reparse point. Đã đo tận nơi: Get-ChildItem -Recurse của
#      PowerShell 5.1 không đi xuyên junction. EnumerateDirectories thì CÓ, nên
#      phải tự chặn. Đi xuyên là mở đường cho một lệnh xóa lan sang thư mục ở
#      đầu bên kia của junction — đúng thứ mà Remove-EmptyDirs đã phải đề phòng.
#
# Duyệt bằng ngăn xếp chứ không đệ quy: cây sâu bất thường thì đệ quy tràn ngăn
# xếp, mà tràn ở giữa một lượt quét thì người dùng nhận được danh sách cụt.
function Get-FilesSafe($path) {
    $out   = New-Object 'Collections.Generic.List[IO.FileInfo]'
    $stack = New-Object 'Collections.Generic.Stack[IO.DirectoryInfo]'
    try { $stack.Push((New-Object IO.DirectoryInfo $path)) }
    catch { $script:LastScanErrors++; return $out }

    while ($stack.Count -gt 0) {
        $d = $stack.Pop()
        try { foreach ($f in $d.EnumerateFiles()) { $out.Add($f) } }
        catch { $script:LastScanErrors++ }
        try {
            foreach ($s in $d.EnumerateDirectories()) {
                if ($s.Attributes -band [IO.FileAttributes]::ReparsePoint) { continue }
                $stack.Push($s)
            }
        } catch { $script:LastScanErrors++ }
    }
    return $out
}

function Show-ScanErrors {
    if ($script:LastScanErrors -gt 0) {
        Write-Host ''
        Write-Host ('  Cảnh báo: {0} mục không đọc được và đã bị bỏ qua.' -f $script:LastScanErrors) -ForegroundColor Yellow
        Write-Host  '  Kết quả có thể chưa đầy đủ. Thử đóng Zalo rồi quét lại.' -ForegroundColor Yellow
    }
}

# ================================================================ thư mục gốc
function Find-ZaloRoots {
    $base = Join-Path $env:APPDATA 'ZaloData\media'
    if (-not (Test-Path -LiteralPath $base)) { return @() }
    $out = @()
    Get-ChildItem -LiteralPath $base -Directory -ErrorAction SilentlyContinue | ForEach-Object {
        $p = Join-Path $_.FullName 'ZaloDownloads'
        if (Test-Path -LiteralPath $p) { $out += $p }
    }
    return $out
}

function Resolve-DataRoot($root) {
    try {
        $p = Split-Path $root -Parent
        $p = Split-Path $p -Parent
        $p = Split-Path $p -Parent
        if (Test-Path -LiteralPath $p) { return $p }
    } catch { }
    return ''
}

function Select-Root {
    param([switch]$Quiet)
    $roots = @(Find-ZaloRoots)
    if ($roots.Count -eq 0) {
        # Lúc khởi động thì im lặng: máy không có Zalo vẫn dùng được các chức năng khác.
        if ($Quiet) { return }
        Write-Host 'Không tìm thấy thư mục ZaloDownloads nào trong AppData.' -ForegroundColor Red
        $manual = Read-Line 'Nhập đường dẫn thủ công (Enter để bỏ qua)'
        if ($manual -ne '' -and (Test-Path -LiteralPath $manual)) { $script:Root = $manual }
    }
    elseif ($roots.Count -eq 1) { $script:Root = $roots[0] }
    else {
        Write-Title 'Chọn tài khoản Zalo'
        for ($i = 0; $i -lt $roots.Count; $i++) {
            $g = @(Get-ChildItem -LiteralPath $roots[$i] -Recurse -File -Force -ErrorAction SilentlyContinue)
            $sum = ($g | Measure-Object Length -Sum).Sum
            if ($null -eq $sum) { $sum = 0 }
            Write-Host ("   {0}  {1}" -f ($i + 1), (Split-Path (Split-Path $roots[$i] -Parent) -Leaf))
            Write-Host ("      {0:N0} tệp · {1}" -f $g.Count, (Show-Size $sum)) -ForegroundColor DarkGray
        }
        $pick = Read-Line 'Chọn số'
        $n = 0
        if ([int]::TryParse($pick, [ref]$n) -and $n -ge 1 -and $n -le $roots.Count) {
            $script:Root = $roots[$n - 1]
        }
    }
    if ($script:DataRoot -eq '' -and $script:Root -ne '') {
        $script:DataRoot = Resolve-DataRoot $script:Root
    }
    Reset-TreeCache
}

function Get-TopFolders {
    if (-not (Test-Path -LiteralPath $script:Root)) { return @() }
    return @(Get-ChildItem -LiteralPath $script:Root -Directory -Force -ErrorAction SilentlyContinue |
             Select-Object -ExpandProperty Name)
}

function Get-TreeSize([switch]$Force) {
    if ($null -ne $script:TreeCache -and -not $Force) { return $script:TreeCache }
    $swTree = [Diagnostics.Stopwatch]::StartNew()
    $script:LastScanErrors = 0
    $g = @(Get-FilesSafe $script:Root)
    $sum = ($g | Measure-Object Length -Sum).Sum
    if ($null -eq $sum) { $sum = 0 }
    $per = @{}
    foreach ($f in $g) {
        $rel = Get-RelPath $f.FullName $script:Root
        $i = $rel.IndexOf('\')
        $top = if ($i -ge 0) { $rel.Substring(0, $i) } else { '(gốc)' }
        if (-not $per.ContainsKey($top)) { $per[$top] = [pscustomobject]@{ Files = 0; Bytes = [long]0 } }
        $per[$top].Files++
        $per[$top].Bytes += $f.Length
    }
    $swTree.Stop()
    $script:TreeScanSec = $swTree.Elapsed.TotalSeconds
    $script:TreeCache = [pscustomobject]@{
        Stamp = (Get-Date).ToString('dd/MM/yyyy HH:mm:ss')
        Files = $g.Count; Bytes = $sum; Per = $per
    }
    return $script:TreeCache
}

# ================================================================ quét theo bộ lọc
# Lọc theo đuôi tệp, thư mục loại trừ và .rescache.
#
# KHÔNG kiểm vùng bảo vệ — chữ "Unguarded" nằm trong tên để chỗ gọi tự nói ra
# điều đó, chứ không bắt người đọc phải mở hàm ra mới biết. Người gọi có bổn
# phận tự chạy Test-Protected TRƯỚC.
#
# Vì sao tách ra: Invoke-Scan đã kiểm vùng bảo vệ ngay đầu vòng lặp để còn đếm
# số tệp bị chặn, nên kiểm lại ở đây là hỏi hai lần cùng một câu về cùng một
# tệp — đo được 0,78 giây trên 56.914 tệp. Thêm người gọi mới thì phải tự đặt
# Test-Protected trước, không có ngoại lệ.
function Test-PassFilterUnguarded($file) {
    if ($script:KeepRescache -and $file.Extension -eq '.rescache') { return $false }
    $e = $file.Extension.ToLower()
    if ($e -eq '') { $e = '(không đuôi)' }
    if ($script:Exts.Count -gt 0 -and $script:Exts -notcontains $e) { return $false }
    if ($script:ExExts.Count -gt 0 -and $script:ExExts -contains $e) { return $false }
    if ($script:ExFolders.Count -gt 0) {
        $rel = Get-RelPath $file.FullName $script:Root
        $i = $rel.IndexOf('\')
        $top = if ($i -ge 0) { $rel.Substring(0, $i) } else { '(gốc)' }
        if ($script:ExFolders -contains $top) { return $false }
    }
    return $true
}

function Invoke-Scan([switch]$Quiet) {
    if (-not (Test-Path -LiteralPath $script:Root)) {
        Write-Host 'Thư mục gốc không hợp lệ.' -ForegroundColor Red
        return
    }
    if (-not $Quiet) {
        Write-Title 'Đang quét theo bộ lọc'
        Write-Host ('  Từ ngày  : ' + (Show-DateLabel $script:FromDate))
        Write-Host ('  Đến ngày : ' + (Show-DateLabel $script:ToDate))
        Write-Host  '  Bước này chỉ đọc, không xóa gì.' -ForegroundColor DarkGray
    }

    $lo = [datetime]::MinValue
    if ($null -ne $script:FromDate) { $lo = $script:FromDate.Date }
    $hi = [datetime]::MaxValue
    if ($null -ne $script:ToDate) { $hi = $script:ToDate.Date.AddDays(1).AddTicks(-1) }
    $minBytes = [long]$script:MinSizeKB * 1KB

    $scanDirs = @()
    if ($script:Folders.Count -gt 0) {
        foreach ($f in $script:Folders) {
            $p = Join-Path $script:Root $f
            if (Test-Path -LiteralPath $p) { $scanDirs += $p }
        }
    } else { $scanDirs = @($script:Root) }

    $hits = New-Object Collections.Generic.List[object]
    $blocked = 0
    $script:LastScanErrors = 0
    $sw = [Diagnostics.Stopwatch]::StartNew()

    # foreach chứ không ForEach-Object: đường ống của PowerShell tốn thêm một
    # nhịp cho mỗi tệp, đo được 0,60 giây trên 56.914 tệp.
    # LƯU Ý cho người sửa sau: trong ForEach-Object thì `return` nghĩa là bỏ qua
    # phần tử này, còn trong foreach thì `return` THOÁT HẲN khỏi hàm. Đổi vòng
    # lặp mà quên đổi `return` thành `continue` là lượt quét dừng ở tệp đầu tiên
    # bị loại, và người dùng nhận một danh sách cụt mà không hay biết.
    foreach ($dir in $scanDirs) {
        foreach ($f in (Get-FilesSafe $dir)) {
            if (Test-Protected $f.FullName) { $blocked++; continue }
            $t = $f.LastWriteTime
            if ($t -lt $lo -or $t -gt $hi) { continue }
            if ($f.Length -lt $minBytes) { continue }
            # An toàn vì vùng bảo vệ đã được kiểm ở ngay đầu vòng lặp này.
            if (-not (Test-PassFilterUnguarded $f)) { continue }
            $hits.Add([pscustomobject]@{ Path = $f.FullName; Size = $f.Length; Date = $t; Keeper = '' })
        }
    }
    $sw.Stop()

    $script:Scan = $hits
    $script:ScanStamp = (Get-Date).ToString('dd/MM/yyyy HH:mm:ss')
    $script:ScanKind = 'DỮ LIỆU ZALO'
    $script:ScanRoot = $script:Root
    $script:CleanupRoots = @($script:Root)
    $script:LastBackup = $null

    $total = ($hits | Measure-Object Size -Sum).Sum
    if ($null -eq $total) { $total = 0 }
    Write-Host ''
    Write-Host ('  Tìm thấy   : {0:N0} tệp' -f $hits.Count) -ForegroundColor Green
    Write-Host ('  Dung lượng : {0}' -f (Show-Size $total)) -ForegroundColor Green
    if (-not $Quiet) { Write-Host ('  Mất {0:N1} giây' -f $sw.Elapsed.TotalSeconds) -ForegroundColor DarkGray }
    if ($blocked -gt 0) {
        Write-Host ('  Đã chặn {0:N0} tệp thuộc vùng bảo vệ.' -f $blocked) -ForegroundColor DarkGray
    }
    Show-ScanErrors
}

# ================================================================ quét cache ứng dụng Zalo
function Invoke-AppCacheScan {
    if ([string]::IsNullOrWhiteSpace($script:DataRoot) -or -not (Test-Path -LiteralPath $script:DataRoot)) {
        Write-Host 'Không xác định được thư mục ZaloData.' -ForegroundColor Red
        return
    }
    Write-Title 'Cache của ứng dụng Zalo'
    Write-Host '  Các thư mục cache nằm ngoài ZaloDownloads. Zalo tự tạo lại được.'
    Write-Host '  Không chứa tin nhắn hay ảnh video đã nhận.' -ForegroundColor Green
    Write-Host ''

    $hits = New-Object Collections.Generic.List[object]
    $script:LastScanErrors = 0
    foreach ($rel in $script:AppCacheRel) {
        $p = Join-Path $script:DataRoot $rel
        if (-not (Test-Path -LiteralPath $p)) { continue }
        if (Test-ProtectedRoot $p) { continue }
        $n = 0; $b = [long]0
        Get-FilesSafe $p | ForEach-Object {
            if (Test-Protected $_.FullName) { return }
            if ($_.Extension -eq '.rescache') { return }
            $hits.Add([pscustomobject]@{ Path = $_.FullName; Size = $_.Length; Date = $_.LastWriteTime; Keeper = ''; Group = $rel })
            $n++; $b += $_.Length
        }
        if ($n -gt 0) { Write-Host ('   {0,-22} {1,7:N0} tệp · {2}' -f $rel, $n, (Show-Size $b)) }
    }
    if ($hits.Count -eq 0) { Write-Host ''; Write-Host '  Không có gì để dọn.' -ForegroundColor Green; return }

    $script:Scan = $hits
    $script:ScanStamp = (Get-Date).ToString('dd/MM/yyyy HH:mm:ss')
    $script:ScanKind = 'CACHE ZALO'
    $script:ScanRoot = $script:DataRoot
    $script:CleanupRoots = @($script:AppCacheRel | ForEach-Object { Join-Path $script:DataRoot $_ } |
                             Where-Object { Test-Path -LiteralPath $_ })
    $script:LastBackup = $null

    $total = ($hits | Measure-Object Size -Sum).Sum
    Write-Host ''
    Write-Host ('  Tổng: {0:N0} tệp · {1}' -f $hits.Count, (Show-Size $total)) -ForegroundColor Green
    Show-ScanErrors
}

# ================================================================ khử trùng lặp
function Get-Sha256Full($path, $sha) {
    $fs = [IO.File]::OpenRead($path)
    try { return [BitConverter]::ToString($sha.ComputeHash($fs)).Replace('-', '') }
    finally { $fs.Dispose() }
}

function Get-QuickSig($path, $size, $sha) {
    $chunk = 65536
    if ($size -le ($chunk * 2)) { return 'FULL:' + (Get-Sha256Full $path $sha) }
    $buf = New-Object byte[] ($chunk * 2)
    $fs = [IO.File]::OpenRead($path)
    try {
        [void]$fs.Read($buf, 0, $chunk)
        $fs.Seek(-$chunk, [IO.SeekOrigin]::End) | Out-Null
        [void]$fs.Read($buf, $chunk, $chunk)
    } finally { $fs.Dispose() }
    return 'Q:' + [BitConverter]::ToString($sha.ComputeHash($buf)).Replace('-', '')
}

# Thân của một luồng băm. Phải tự chứa mọi thứ vì runspace không thừa hưởng
# hàm của script chính. Trả về hashtable đường dẫn -> chữ ký, $null nếu đọc lỗi.
#   $mode = 'FULL'  băm toàn bộ nội dung, chữ ký mang tiền tố 'FULL:'
#   $mode = 'QUICK' băm 64 KB đầu và 64 KB cuối, tiền tố 'Q:'
#                   tệp từ 128 KB trở xuống thì đằng nào cũng đọc hết cả tệp,
#                   nên trả thẳng 'FULL:' — chữ ký đó chính là SHA256 toàn tệp.
# Hai tiền tố khác nhau để chữ ký nhanh không bao giờ bị đem so với chữ ký đầy đủ.
$script:HashWorker = {
    param($paths, $mode)
    $out = @{}
    $sha = [Security.Cryptography.SHA256]::Create()
    $chunk = 65536
    try {
        foreach ($p in $paths) {
            try {
                $fs = [IO.File]::OpenRead($p)
                try {
                    if ($mode -eq 'FULL' -or $fs.Length -le ($chunk * 2)) {
                        $out[$p] = 'FULL:' + [BitConverter]::ToString($sha.ComputeHash($fs)).Replace('-', '')
                    } else {
                        # Đọc cho bằng đủ chứ không tin một lần Read trả về đúng
                        # số byte đã xin. Read thiếu thì phần đuôi của bộ đệm còn
                        # rác, hai tệp giống hệt nhau có thể ra hai chữ ký khác
                        # nhau và bản trùng bị bỏ sót.
                        $buf = New-Object byte[] ($chunk * 2)
                        $got = 0
                        while ($got -lt $chunk) {
                            $n = $fs.Read($buf, $got, $chunk - $got)
                            if ($n -le 0) { break }
                            $got += $n
                        }
                        [void]$fs.Seek(-$chunk, [IO.SeekOrigin]::End)
                        $got = 0
                        while ($got -lt $chunk) {
                            $n = $fs.Read($buf, $chunk + $got, $chunk - $got)
                            if ($n -le 0) { break }
                            $got += $n
                        }
                        $out[$p] = 'Q:' + [BitConverter]::ToString($sha.ComputeHash($buf)).Replace('-', '')
                    }
                } finally { $fs.Dispose() }
            } catch { $out[$p] = $null }
        }
    } finally { $sha.Dispose() }
    return $out
}

# Băm một mẻ tệp, chia cho nhiều luồng cùng đọc đĩa.
#
# Chỉ đúng phép băm được chạy song song. Băm là việc thuần túy tính toán: không
# ghi, không xóa, không quyết định gì. Toàn bộ khâu so sánh, loại bỏ và xóa vẫn
# nằm ở luồng chính và đọc được tuần tự từ trên xuống. Không bao giờ để luồng
# phụ chạm tới việc xóa.
function Get-HashSet($paths, $mode, $activity) {
    $res = @{}
    $list = @($paths)
    if ($list.Count -eq 0) { return $res }

    # Ít tệp thì dựng runspace còn đắt hơn là làm thẳng một mạch.
    $n = [Math]::Min($script:HashThreads, $list.Count)
    if ($n -le 1 -or $list.Count -lt 16) {
        $r = & $script:HashWorker $list $mode
        foreach ($k in $r.Keys) { $res[$k] = $r[$k] }
        return $res
    }

    # Chia kiểu chia bài để mỗi luồng nhận lẫn lộn tệp to với tệp nhỏ, tránh
    # cảnh một luồng ôm hết phần nặng còn các luồng khác ngồi không.
    $chunks = New-Object 'Collections.Generic.List[object]'
    for ($i = 0; $i -lt $n; $i++) { $chunks.Add((New-Object Collections.Generic.List[string])) }
    for ($i = 0; $i -lt $list.Count; $i++) { $chunks[$i % $n].Add([string]$list[$i]) }

    $pool = [RunspaceFactory]::CreateRunspacePool(1, $n)
    $pool.Open()
    $jobs = @()
    try {
        foreach ($c in $chunks) {
            if ($c.Count -eq 0) { continue }
            $ps = [PowerShell]::Create()
            $ps.RunspacePool = $pool
            [void]$ps.AddScript($script:HashWorker.ToString()).AddArgument($c.ToArray()).AddArgument($mode)
            $jobs += [pscustomobject]@{ PS = $ps; Handle = $ps.BeginInvoke() }
        }
        $done = 0
        foreach ($j in $jobs) {
            foreach ($h in $j.PS.EndInvoke($j.Handle)) {
                if ($h -isnot [hashtable]) { continue }
                foreach ($k in $h.Keys) { $res[$k] = $h[$k] }
            }
            $done++
            Write-Progress -Activity $activity -Status ("luồng {0}/{1}" -f $done, $jobs.Count) `
                           -PercentComplete (($done / $jobs.Count) * 100)
        }
    } finally {
        foreach ($j in $jobs) { try { $j.PS.Dispose() } catch { } }
        try { $pool.Close(); $pool.Dispose() } catch { }
        Write-Progress -Activity $activity -Completed
    }
    return $res
}

function Invoke-DedupScan {
    if (-not (Test-Path -LiteralPath $script:Root)) { Write-Host 'Thư mục gốc không hợp lệ.' -ForegroundColor Red; return }
    $resRoot = Join-Path $script:Root 'resource'
    if (-not (Test-Path -LiteralPath $resRoot)) {
        Write-Host 'Không có thư mục resource để đối chiếu.' -ForegroundColor Yellow; return
    }

    Write-Title 'Tìm bản trùng lặp'
    Write-Host '  Zalo lưu mỗi tấm ảnh và mỗi video hai bản: một bản độc lập, một bản'
    Write-Host '  trong thư mục theo hội thoại. Công cụ tìm bản thừa và luôn giữ bản gốc.'
    Write-Host ''
    Write-Host '  Kết luận chỉ dựa trên đối chiếu nội dung bằng SHA256 toàn tệp.' -ForegroundColor Green
    Write-Host ''

    $script:LastScanErrors = 0
    Write-Host '  Bước 1/4 · lập chỉ mục bản giữ lại...' -ForegroundColor DarkGray
    $keepBySize = @{}; $keepCount = 0
    foreach ($d in $script:StandaloneDirs) {
        $p = Join-Path $script:Root $d
        if (-not (Test-Path -LiteralPath $p)) { continue }
        Get-FilesSafe $p | ForEach-Object {
            if ($_.Extension -eq '.rescache' -or $_.Length -le 0) { return }
            if (Test-Protected $_.FullName) { return }
            if (-not $keepBySize.ContainsKey($_.Length)) {
                $keepBySize[$_.Length] = New-Object Collections.Generic.List[string]
            }
            $keepBySize[$_.Length].Add($_.FullName); $keepCount++
        }
    }
    Write-Host ("          {0:N0} tệp độc lập, {1:N0} nhóm kích thước" -f $keepCount, $keepBySize.Count) -ForegroundColor DarkGray

    if ($keepCount -eq 0) {
        Write-Host ''
        Write-Host '  Không còn bản độc lập nào để đối chiếu.' -ForegroundColor Red
        Write-Host '  Mọi tệp trong resource lúc này đều là bản duy nhất — không xóa gì cả.' -ForegroundColor Red
        return
    }

    Write-Host '  Bước 2/4 · lọc ứng viên theo kích thước...' -ForegroundColor DarkGray
    $cands = New-Object Collections.Generic.List[object]
    $candBytes = [long]0
    Get-FilesSafe $resRoot | ForEach-Object {
        if ($_.Extension -eq '.rescache' -or $_.Length -le 0) { return }
        if ($_.FullName -like '*\Cache\*') { return }
        if (Test-Protected $_.FullName) { return }
        if ($keepBySize.ContainsKey($_.Length)) {
            $cands.Add([pscustomobject]@{ Path = $_.FullName; Size = $_.Length; Date = $_.LastWriteTime })
            $candBytes += $_.Length
        }
    }
    Write-Host ("          {0:N0} ứng viên · {1}" -f $cands.Count, (Show-Size $candBytes)) -ForegroundColor DarkGray

    if ($cands.Count -eq 0) { Write-Host ''; Write-Host '  Không tìm thấy bản trùng nào.' -ForegroundColor Green; return }

    Write-Host ''
    Write-Host '  Bước tiếp theo đọc đĩa để băm nội dung, có thể mất vài phút.' -ForegroundColor DarkGray
    if (-not (Read-YesNo '  Tiếp tục? (c/k)')) { Write-Host '  Đã hủy.' -ForegroundColor DarkGray; return }

    $dups = New-Object Collections.Generic.List[object]
    $quickReject = 0; $fullReject = 0; $err = 0
    $sw = [Diagnostics.Stopwatch]::StartNew()

    try {
        Write-Host ''
        Write-Host '  Bước 3/4 · lọc nhanh bằng chữ ký đầu và cuối tệp...' -ForegroundColor DarkGray

        # Chỉ đụng tới bản giữ lại thuộc nhóm kích thước thật sự có ứng viên.
        # Nhóm không có ứng viên nào thì không đọc đến, y như bản cũ.
        # Khác bản cũ ở chỗ: trong một nhóm đã bị đụng thì băm hết bản giữ lại
        # chứ không dừng ở bản khớp đầu tiên. Đổi lại được cả mẻ để chia luồng.
        # Trên dữ liệu thật mỗi nhóm chỉ có trung bình 1,8 bản nên phần đọc thừa
        # là không đáng kể.
        $needSizes = @{}
        foreach ($c in $cands) { $needSizes[$c.Size] = $true }
        $batch = New-Object Collections.Generic.List[string]
        foreach ($c in $cands) { $batch.Add($c.Path) }
        foreach ($s in $needSizes.Keys) {
            foreach ($kp in $keepBySize[$s]) {
                if ([IO.File]::Exists($kp)) { $batch.Add($kp) }
            }
        }
        $quick = Get-HashSet $batch 'QUICK' 'Lọc nhanh'

        $pairs = New-Object Collections.Generic.List[object]
        foreach ($c in $cands) {
            $cq = $quick[$c.Path]
            if ($null -eq $cq) { $err++; continue }
            foreach ($kp in $keepBySize[$c.Size]) {
                $kq = $quick[$kp]
                if ($null -eq $kq) { continue }
                if ($kq -eq $cq) { $pairs.Add([pscustomobject]@{ Cand = $c; Keep = $kp; Sig = $cq }); break }
                else { $quickReject++ }
            }
        }
        Write-Host ("          {0:N0} cặp qua vòng lọc nhanh" -f $pairs.Count) -ForegroundColor DarkGray

        Write-Host '  Bước 4/4 · xác minh SHA256 toàn tệp...' -ForegroundColor DarkGray

        # Tệp từ 128 KB trở xuống đã bị đọc trọn vẹn ở bước 3, nên chữ ký
        # 'FULL:' của nó CHÍNH LÀ SHA256 toàn tệp — cặp nào khớp chữ ký đó thì
        # đã xác minh xong, không phải đọc lại. Bản cũ vẫn đọc lại lần nữa ở
        # bước này; với dữ liệu Zalo thật thì gần như mọi ứng viên đều nằm dưới
        # ngưỡng đó, tức là đọc thừa nguyên một lượt toàn bộ dữ liệu.
        # Kết luận không hề nới lỏng: vẫn là đối chiếu SHA256 toàn tệp.
        $needFull = @{}
        foreach ($pr in $pairs) {
            if ([string]$pr.Sig -like 'FULL:*') { continue }
            $needFull[$pr.Cand.Path] = $true
            $needFull[$pr.Keep] = $true
        }
        $full = Get-HashSet @($needFull.Keys) 'FULL' 'Xác minh toàn tệp'

        foreach ($pr in $pairs) {
            if ([string]$pr.Sig -like 'FULL:*') {
                $dups.Add([pscustomobject]@{ Path = $pr.Cand.Path; Size = $pr.Cand.Size
                                             Date = $pr.Cand.Date; Keeper = $pr.Keep })
                continue
            }
            $ch = $full[$pr.Cand.Path]; $kh = $full[$pr.Keep]
            if ($null -eq $ch -or $null -eq $kh) { $err++; continue }
            if ($ch -eq $kh) {
                $dups.Add([pscustomobject]@{ Path = $pr.Cand.Path; Size = $pr.Cand.Size
                                             Date = $pr.Cand.Date; Keeper = $pr.Keep })
            } else { $fullReject++ }
        }
    } finally {
        $sw.Stop()
    }

    $dupBytes = ($dups | Measure-Object Size -Sum).Sum
    if ($null -eq $dupBytes) { $dupBytes = 0 }

    Write-Host ''
    Write-Host ('  Bản trùng xác nhận : {0:N0} tệp' -f $dups.Count) -ForegroundColor Green
    Write-Host ('  Có thể thu hồi     : {0}' -f (Show-Size $dupBytes)) -ForegroundColor Green
    Write-Host ('  Loại ở vòng lọc nhanh : {0:N0}' -f $quickReject) -ForegroundColor DarkGray
    Write-Host ('  Loại ở vòng toàn tệp  : {0:N0}' -f $fullReject) -ForegroundColor DarkGray
    Write-Host ('  Thời gian          : {0:N1} giây' -f $sw.Elapsed.TotalSeconds) -ForegroundColor DarkGray
    if ($err -gt 0) { Write-Host ('  Lỗi đọc tệp        : {0:N0}' -f $err) -ForegroundColor Yellow }
    Show-ScanErrors

    if ($dups.Count -eq 0) { return }

    $script:Scan = $dups
    $script:ScanStamp = (Get-Date).ToString('dd/MM/yyyy HH:mm:ss')
    $script:ScanKind = 'BẢN TRÙNG LẶP'
    $script:ScanRoot = $script:Root
    $script:CleanupRoots = @($script:Root)
    $script:LastBackup = $null
}

# ================================================================ danh mục cache hệ thống
# Danh mục có thể nạp từ catalog.json cạnh script để mở rộng cho máy khác
# mà không phải sửa mã nguồn. Không có tệp đó thì dùng danh sách dựng sẵn.
# Kiểm tra một mục trong catalog.json, trả về danh sách lý do nó không dùng được.
# Mảng rỗng nghĩa là mục hợp lệ.
function Test-CatalogEntry($e) {
    $bad = @()
    if ([string]::IsNullOrWhiteSpace($e.name)) { $bad += 'thiếu "name"' }
    if (-not $e.PSObject.Properties['paths']) { $bad += 'thiếu "paths" (có phải bạn gõ "path"?)' }
    elseif (@($e.paths).Count -eq 0)          { $bad += '"paths" rỗng' }
    elseif (@($e.paths | Where-Object { -not ($_ -is [string]) }).Count -gt 0) {
        $bad += '"paths" phải là mảng chuỗi'
    }
    if ($e.PSObject.Properties['group'] -and $e.group -notin @('A', 'B', 'C')) {
        $bad += ('"group" phải là A, B hoặc C — đang là "' + $e.group + '"')
    }
    if ($e.PSObject.Properties['risk'] -and $e.risk -notin @('XANH', 'VÀNG')) {
        $bad += ('"risk" phải là XANH hoặc VÀNG — đang là "' + $e.risk + '"')
    }
    if ($e.PSObject.Properties['ageHours']) {
        $n = 0
        if (-not [int]::TryParse([string]$e.ageHours, [ref]$n) -or $n -lt 0) {
            $bad += '"ageHours" phải là số nguyên không âm'
        }
    }
    if ($e.PSObject.Properties['procs'] -and $null -ne $e.procs -and
        @($e.procs | Where-Object { -not ($_ -is [string]) }).Count -gt 0) {
        $bad += '"procs" phải là mảng chuỗi'
    }
    return $bad
}

function Get-CatalogDefs {
    if (Test-Path -LiteralPath $script:CatalogFile) {
        try {
            $j = Get-Content -LiteralPath $script:CatalogFile -Raw -Encoding UTF8 | ConvertFrom-Json
            $out = @()
            $rejected = @()
            $idx = 0
            foreach ($e in @($j.entries)) {
                $idx++
                # Mục sai phải được nêu tên. Bỏ qua im lặng nghĩa là bạn nhìn màn
                # hình thấy thiếu một mục mà không tài nào biết tại sao.
                $bad = Test-CatalogEntry $e
                if ($bad.Count -gt 0) {
                    $who = if ([string]::IsNullOrWhiteSpace($e.name)) { "mục thứ $idx" } else { '"' + $e.name + '"' }
                    $rejected += ('   ' + $who + ' — ' + ($bad -join '; '))
                    continue
                }
                $paths = @()
                foreach ($p in @($e.paths)) {
                    $x = [Environment]::ExpandEnvironmentVariables($p)
                    if (-not [string]::IsNullOrWhiteSpace($x)) { $paths += $x }
                }
                if ($paths.Count -eq 0) {
                    $rejected += ('   "' + $e.name + '" — "paths" chỉ chứa chuỗi rỗng sau khi thay biến môi trường')
                    continue
                }
                $h = @{ G = $e.group; N = $e.name; P = $paths
                        R = $(if ($e.risk) { $e.risk } else { 'XANH' })
                        Note = $(if ($e.note) { $e.note } else { '' })
                        Warn = $(if ($e.warning) { [string]$e.warning } else { '' })
                        Pr = @($e.procs)
                        V = $(if ($e.PSObject.Properties['verified']) { [bool]$e.verified } else { $false }) }
                if ($e.PSObject.Properties['ageHours'] -and [int]$e.ageHours -gt 0) { $h.Age = [int]$e.ageHours }
                $out += $h
            }
            if ($rejected.Count -gt 0) {
                Write-Host ''
                Write-Host ('  {0} mục trong catalog.json bị bỏ qua vì sai định dạng:' -f $rejected.Count) -ForegroundColor Yellow
                $rejected | ForEach-Object { Write-Host $_ -ForegroundColor DarkYellow }
                Write-Host ''
            }
            if ($out.Count -gt 0) { return $out }
            Write-Host '  Không mục nào trong catalog.json dùng được, quay về danh mục dựng sẵn.' -ForegroundColor Yellow
        } catch {
            Write-Host ('  Không đọc được catalog.json, dùng danh mục dựng sẵn: ' + $_.Exception.Message) -ForegroundColor Yellow
        }
    }
    return (Get-CatalogDefaults)
}

function Get-CatalogDefaults {
    $u = $env:USERPROFILE; $l = $env:LOCALAPPDATA; $w = $env:WINDIR; $t = $env:TEMP
    return @(
        @{G='A'; N='pre-commit';              P=@("$u\.cache\pre-commit");        R='VÀNG'; Pr=@();                   Note='Môi trường hook, tự dựng lại nhưng phải tải từ mạng'}
        @{G='A'; N='npm';                     P=@("$l\npm-cache");                R='VÀNG'; Pr=@('node');             Note='Gói npm tải lại khi cài lại'}
        @{G='A'; N='codex runtimes';          P=@("$u\.cache\codex-runtimes");    R='VÀNG'; Pr=@('node');             Note='Runtime tải lại khi cần'}
        @{G='A'; N='Playwright';              P=@("$l\ms-playwright");            R='VÀNG'; Pr=@('node','chrome');    Note='Trình duyệt test, tải lại bằng lệnh playwright install'}
        @{G='A'; N='HuggingFace';             P=@("$u\.cache\huggingface");       R='VÀNG'; Pr=@('python','pythonw'); Note='Mô hình AI, tải lại có thể rất nặng mạng'}
        @{G='A'; N='uv (Python)';             P=@("$l\uv\cache");                 R='VÀNG'; Pr=@('python','pythonw'); Note='Gói Python tải lại từ mạng'}
        @{G='A'; N='Puppeteer';               P=@("$u\.cache\puppeteer");         R='VÀNG'; Pr=@('node','chrome');    Note='Chromium tải lại khi cần'}
        @{G='A'; N='pip (Python)';            P=@("$l\pip\Cache");                R='VÀNG'; Pr=@('python','pythonw'); Note='Gói Python tải lại từ mạng'}
        @{G='A'; N='cargo registry (Rust)';   P=@("$u\.cargo\registry");          R='VÀNG'; Pr=@('cargo','rustc');    Note='Crate tải lại từ mạng. Không đụng .cargo\bin và .rustup'}

        @{G='B'; N='Ollama — bộ cài cũ';      P=@("$l\Ollama\updates_v2");        R='XANH'; Pr=@('ollama');           Note='Tệp cài đặt còn sót, không còn tác dụng'}
        @{G='B'; N='LM Studio — bộ cài cũ';   P=@("$l\lm-studio-updater");        R='XANH'; Pr=@();                   Note='Tệp cài đặt còn sót'}
        @{G='B'; N='Intel Package Cache';     P=@((Join-Path $env:ProgramData 'Intel Package Cache*')); R='XANH'; Pr=@();      Note='Bộ cài driver Intel đã dùng xong'}
        @{G='B'; N='GIGABYTE đã tải';         P=@((Join-Path $env:ProgramFiles 'GIGABYTE\Control Center\Lib\Download')); R='XANH'; Pr=@(); Note='Bộ cài driver đã tải, không còn cần'}

        @{G='C'; N='Tệp tạm của bạn';         P=@("$t");                          R='XANH'; Pr=@(); Age=24;           Note='Chỉ tệp cũ hơn 24 giờ, để không phá ứng dụng đang mở'}
        @{G='C'; N='Tệp tạm của Windows';     P=@("$w\Temp");                     R='XANH'; Pr=@(); Age=24;           Note='Chỉ tệp cũ hơn 24 giờ'}
        @{G='C'; N='Cache Chrome';            P=@("$l\Google\Chrome\User Data\*\Cache"); R='XANH'; Pr=@('chrome');    Note='Trang web tải lại khi truy cập'}
        @{G='C'; N='Ảnh thu nhỏ';             P=@("$l\Microsoft\Windows\Explorer"); R='XANH'; Pr=@();                 Note='Windows tự dựng lại'}
        @{G='C'; N='INetCache';               P=@("$l\Microsoft\Windows\INetCache"); R='XANH'; Pr=@();                Note='Cache Internet cũ'}
        @{G='C'; N='Báo cáo lỗi ứng dụng';    P=@("$l\CrashDumps");               R='XANH'; Pr=@();                   Note='Crash dump'}
        @{G='C'; N='Windows Update đã tải';   P=@("$w\SoftwareDistribution\Download"); R='XANH'; Pr=@();              Note='Gói cập nhật đã cài xong'}
    )
}

function Test-Writable($path) {
    $t = Join-Path $path ('zc_' + [Guid]::NewGuid().ToString('N').Substring(0, 6) + '.tmp')
    try { [IO.File]::WriteAllText($t, 'x'); Remove-Item -LiteralPath $t -Force -ErrorAction SilentlyContinue; return $true }
    catch { return $false }
}

function Test-IsAdmin {
    try {
        $id = [Security.Principal.WindowsIdentity]::GetCurrent()
        return (New-Object Security.Principal.WindowsPrincipal($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    } catch { return $false }
}

function Restart-Elevated {
    Write-Host ''
    Write-Host '  Sẽ mở lại công cụ trong cửa sổ mới với quyền quản trị.' -ForegroundColor Yellow
    Write-Host '  Windows sẽ hiện hộp thoại xác nhận.' -ForegroundColor DarkGray
    if (-not (Read-YesNo '  Tiếp tục? (c/k)')) { Write-Host '  Đã hủy.' -ForegroundColor DarkGray; return $false }
    try {
        Start-Process powershell.exe -Verb RunAs -ArgumentList @(
            '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$PSCommandPath`""
        ) -ErrorAction Stop
        Write-Host '  Đã mở cửa sổ mới. Cửa sổ này sẽ đóng.' -ForegroundColor Green
        Start-Sleep -Seconds 2
        exit 0
    } catch {
        Write-Host ('  Không mở được: ' + $_.Exception.Message) -ForegroundColor Red
        return $false
    }
}

function Build-Catalog {
    $defs = Get-CatalogDefs
    $out = New-Object Collections.Generic.List[object]
    $id = 0; $i = 0
    foreach ($d in $defs) {
        $i++
        Write-Progress -Activity 'Đo dung lượng' -Status $d.N -PercentComplete (($i / $defs.Count) * 100)
        $paths = @()
        foreach ($pat in $d.P) {
            if ($pat -match '[\*\?]') {
                $paths += @(Get-Item -Path $pat -Force -ErrorAction SilentlyContinue |
                            Where-Object { $_.PSIsContainer } | Select-Object -ExpandProperty FullName)
            } elseif (Test-Path -LiteralPath $pat) { $paths += $pat }
        }
        $paths = @($paths | Where-Object { -not (Test-ProtectedRoot $_) } | Select-Object -Unique)
        if ($paths.Count -eq 0) { continue }

        $minAge = 0
        if ($d.ContainsKey('Age')) { $minAge = [int]$d.Age }
        $cut = [datetime]::MaxValue
        if ($minAge -gt 0) { $cut = (Get-Date).AddHours(-$minAge) }

        $files = 0; $bytes = [long]0
        foreach ($p in $paths) {
            $g = @(Get-ChildItem -LiteralPath $p -Recurse -File -Force -ErrorAction SilentlyContinue |
                   Where-Object { $_.LastWriteTime -lt $cut })
            $files += $g.Count
            $b = ($g | Measure-Object Length -Sum).Sum
            if ($null -ne $b) { $bytes += $b }
        }
        if ($files -eq 0) { continue }

        $verified = $true
        if ($d.ContainsKey('V')) { $verified = [bool]$d.V }

        $id++
        $warn = ''
        if ($d.ContainsKey('Warn')) { $warn = [string]$d.Warn }

        $out.Add([pscustomobject]@{
            Id = $id; Group = $d.G; Name = $d.N; Paths = $paths
            Risk = $d.R; Note = $d.Note; Procs = $d.Pr
            Files = $files; Bytes = $bytes; MinAge = $minAge
            Verified = $verified; Warning = $warn
            Writable = (Test-Writable $paths[0])
        })
    }
    Write-Progress -Activity 'Đo dung lượng' -Completed
    return $out
}

function Invoke-CatalogScan {
    Write-Title 'Cache hệ thống ngoài Zalo'
    Write-Host '  Công cụ chỉ đụng vào các vị trí đã được kiểm chứng, không dò tìm theo'
    Write-Host '  mẫu tên. Rất nhiều ứng dụng đặt tên thư mục là Cache nhưng bên trong'
    Write-Host '  là dữ liệu thật, nên dò theo tên là cách làm hỏng máy.' -ForegroundColor Green
    Write-Host ''
    Write-Host '  Đang đo dung lượng từng mục...' -ForegroundColor DarkGray

    $cat = Build-Catalog
    if ($cat.Count -eq 0) { Write-Host '  Không có mục nào có dữ liệu.' -ForegroundColor Green; return }

    $running = @(Get-Process -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name -Unique)
    $sel = @{}
    $notice = ''

    while ($true) {
        Clear-Host
        Write-Title 'Cache hệ thống ngoài Zalo'
        $lastG = ''
        foreach ($e in $cat) {
            if ($e.Group -ne $lastG) {
                $lastG = $e.Group
                $gname = switch ($e.Group) { 'A' { 'Cache công cụ lập trình' }
                                             'B' { 'Bộ cài đặt thừa' }
                                             'C' { 'Tệp tạm và trình duyệt' } }
                Write-Host ''
                Write-Host ("  ── {0} {1}" -f $e.Group, $gname) -ForegroundColor Cyan
            }
            $mark = if ($sel.ContainsKey($e.Id)) { 'x' } else { ' ' }
            $col  = if ($e.Risk -eq 'VÀNG') { 'Yellow' } else { 'Gray' }
            $warn = ''
            if (-not $e.Writable) { $warn += '  · cần quyền quản trị' }
            if (-not $e.Verified) { $warn += '  · chưa kiểm chứng tận nơi' }
            if ($e.MinAge -gt 0) { $warn += ('  · chỉ tệp cũ hơn ' + $e.MinAge + ' giờ') }
            $busy = @($e.Procs | Where-Object { $running -contains $_ })
            if ($busy.Count -gt 0) { $warn += ('  · đang chạy: ' + ($busy -join ',')) }
            if ($e.Warning -ne '') { $warn += '  · có cảnh báo' }
            Write-Host ("   [{0}] {1,2}. {2,-24} {3,9}  {4,7:N0} tệp{5}" -f `
                        $mark, $e.Id, $e.Name, (Show-Size $e.Bytes), $e.Files, $warn) -ForegroundColor $col
            Write-Host ("           " + $e.Note) -ForegroundColor DarkGray
            if ($e.Warning -ne '') { Write-Host ("           ! " + $e.Warning) -ForegroundColor Yellow }
        }

        $chosen = @($cat | Where-Object { $sel.ContainsKey($_.Id) })
        $selBytes = ($chosen | Measure-Object Bytes -Sum).Sum
        if ($null -eq $selBytes) { $selBytes = 0 }
        Write-Host ''
        Write-Host ('  Đang chọn {0} mục · {1}' -f $chosen.Count, (Show-Size $selBytes)) -ForegroundColor Green
        Write-Host ''
        Write-Host '   Gõ số hoặc chữ nhóm, cách nhau bởi dấu phẩy. Ví dụ: A,12,15'
        Write-Host '   Gõ  *   chọn tất cả trừ mục có cảnh báo    Gõ  -   bỏ chọn tất cả'
        Write-Host '   Gõ  ok  để dọn các mục đã chọn  Gõ  0   quay lại'
        if (@($cat | Where-Object { -not $_.Writable }).Count -gt 0 -and -not (Test-IsAdmin)) {
            Write-Host '   Gõ  admin  mở lại với quyền quản trị' -ForegroundColor Yellow
        }
        if ($notice -ne '') { Write-Host ''; Write-Host ('   ' + $notice) -ForegroundColor Yellow; $notice = '' }
        $raw = Read-Line '  Lệnh'

        if ($raw -eq '' -or $raw -eq '0') { return }
        if ($raw.ToLower() -eq 'admin') { Restart-Elevated | Out-Null; continue }
        if ($raw -eq '-') { $sel = @{}; continue }
        # Chọn hàng loạt không được kéo theo mục có cảnh báo — thứ đó phải cố ý,
        # tức là gõ đúng số của nó.
        if ($raw -eq '*') {
            $skipped = @($cat | Where-Object { $_.Warning -ne '' })
            foreach ($e in $cat) { if ($e.Warning -eq '') { $sel[$e.Id] = $true } }
            if ($skipped.Count -gt 0) {
                $notice = 'Đã bỏ qua ' + $skipped.Count + ' mục có cảnh báo: ' +
                          (($skipped | Select-Object -Expand Name) -join ', ') + '. Gõ đúng số nếu vẫn muốn chọn.'
            }
            continue
        }
        if ($raw.ToLower() -eq 'ok') {
            if ($chosen.Count -eq 0) { $notice = 'Chưa chọn mục nào. Gõ số, chữ nhóm, hoặc * để chọn tất cả.'; continue }
            $chuaKiem = @($chosen | Where-Object { -not $_.Verified })
            if ($chuaKiem.Count -gt 0) {
                Write-Host ''
                Write-Host ('  Trong lựa chọn có {0} mục chưa kiểm chứng tận nơi: {1}' -f `
                            $chuaKiem.Count, (($chuaKiem | Select-Object -Expand Name) -join ', ')) -ForegroundColor Yellow
                if (-not (Read-YesNo '  Vẫn tiếp tục? (c/k)')) { continue }
            }

            # Đọc lại danh sách tiến trình ngay lúc này, không dùng ảnh chụp lúc
            # mở màn hình — bạn có thể vừa đóng ứng dụng xong.
            $running = @(Get-Process -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name -Unique)
            $busy = @($chosen | Where-Object { @($_.Procs | Where-Object { $running -contains $_ }).Count -gt 0 })
            if ($busy.Count -gt 0) {
                Write-Host ''
                Write-Host ('  {0} mục đang được ứng dụng khác sử dụng:' -f $busy.Count) -ForegroundColor Yellow
                foreach ($e in $busy) {
                    $who = @($e.Procs | Where-Object { $running -contains $_ }) -join ', '
                    Write-Host ('   · {0,-24} đang chạy: {1}' -f $e.Name, $who) -ForegroundColor Yellow
                }
                Write-Host ''
                Write-Host '  Ứng dụng đang chạy thường tải lại ngay thứ vừa bị xóa. Lần trước,' -ForegroundColor DarkGray
                Write-Host '  Ollama tải lại đúng 1.491 MB chỉ mười một phút sau khi dọn.' -ForegroundColor DarkGray
                Write-Host ''
                Write-Host '   1  Bỏ các mục đó ra, dọn phần còn lại'
                Write-Host '   2  Cứ dọn hết'
                Write-Host '   Enter để quay lại, tự đóng ứng dụng rồi thử lại'
                $bc = Read-Line '  Chọn'
                if ($bc -eq '1') {
                    foreach ($e in $busy) { $sel.Remove($e.Id) }
                    $chosen = @($cat | Where-Object { $sel.ContainsKey($_.Id) })
                    if ($chosen.Count -eq 0) {
                        $notice = 'Bỏ xong thì không còn mục nào. Hãy chọn lại.'
                        continue
                    }
                }
                elseif ($bc -ne '2') { continue }
            }
            break
        }

        $add = @(); $bad = @(); $skippedWarn = @()
        foreach ($tok in $raw.Split(',')) {
            $tk = $tok.Trim()
            if ($tk -eq '') { continue }
            if ($tk.ToUpper() -in @('A', 'B', 'C')) {
                $grp = @($cat | Where-Object { $_.Group -eq $tk.ToUpper() })
                $skippedWarn += @($grp | Where-Object { $_.Warning -ne '' })
                $add += @($grp | Where-Object { $_.Warning -eq '' } | Select-Object -ExpandProperty Id)
                continue
            }
            $n = 0
            if ([int]::TryParse($tk, [ref]$n) -and @($cat | Where-Object { $_.Id -eq $n }).Count -eq 1) { $add += $n }
            else { $bad += $tk }
        }
        if ($bad.Count -gt 0) {
            $notice = 'Giá trị không hợp lệ: ' + ($bad -join ', ') + '. Lựa chọn giữ nguyên để tránh xóa nhầm.'
            continue
        }
        foreach ($n in $add) { $sel[$n] = $true }
        if ($skippedWarn.Count -gt 0) {
            $notice = 'Chọn theo nhóm đã bỏ qua ' + $skippedWarn.Count + ' mục có cảnh báo: ' +
                      (($skippedWarn | Select-Object -Expand Name -Unique) -join ', ') + '. Gõ đúng số nếu vẫn muốn chọn.'
        }
    }

    $chosen = @($cat | Where-Object { $sel.ContainsKey($_.Id) })
    $skipAdmin = @($chosen | Where-Object { -not $_.Writable })
    if ($skipAdmin.Count -gt 0) {
        Write-Host ''
        Write-Host ('  {0} mục cần quyền quản trị sẽ bị bỏ qua: {1}' -f $skipAdmin.Count, (($skipAdmin | Select-Object -Expand Name) -join ', ')) -ForegroundColor Yellow
        $chosen = @($chosen | Where-Object { $_.Writable })
        if ($chosen.Count -eq 0) { Write-Host '  Không còn mục nào để dọn.' -ForegroundColor Red; return }
    }

    Write-Host ''
    Write-Host '  Đang thu thập danh sách tệp...' -ForegroundColor DarkGray
    $hits = New-Object Collections.Generic.List[object]
    $roots = @()
    $script:LastScanErrors = 0
    foreach ($e in $chosen) {
        $cut = [datetime]::MaxValue
        if ($e.MinAge -gt 0) { $cut = (Get-Date).AddHours(-$e.MinAge) }
        foreach ($p in $e.Paths) {
            if (Test-ProtectedRoot $p) { continue }
            $roots += $p
            Get-FilesSafe $p | ForEach-Object {
                if (Test-Protected $_.FullName) { return }
                if ($_.LastWriteTime -ge $cut) { return }
                $hits.Add([pscustomobject]@{ Path = $_.FullName; Size = $_.Length; Date = $_.LastWriteTime
                                             Keeper = ''; Group = $e.Name })
            }
        }
    }
    if ($hits.Count -eq 0) { Write-Host '  Không có tệp nào.' -ForegroundColor Green; return }

    $script:Scan = $hits
    $script:ScanStamp = (Get-Date).ToString('dd/MM/yyyy HH:mm:ss')
    $script:ScanKind = 'CACHE HỆ THỐNG'
    $script:ScanRoot = $script:SysRoot
    $script:CleanupRoots = @($roots | Select-Object -Unique)
    $script:LastBackup = $null

    $total = ($hits | Measure-Object Size -Sum).Sum
    Write-Host ''
    Write-Host ('  Tìm thấy   : {0:N0} tệp' -f $hits.Count) -ForegroundColor Green
    Write-Host ('  Dung lượng : {0}' -f (Show-Size $total)) -ForegroundColor Green
    Show-ScanErrors
}

# ================================================================ chi tiết
function Show-ScanDetail {
    if ($null -eq $script:Scan -or $script:Scan.Count -eq 0) {
        Write-Host 'Chưa có kết quả quét.' -ForegroundColor Yellow; return
    }
    $base = $script:ScanRoot
    if ([string]::IsNullOrWhiteSpace($base)) { $base = $script:Root }

    Write-Title ('Chi tiết · ' + $script:ScanKind)
    $hasGroup = @($script:Scan | Where-Object { $_.PSObject.Properties['Group'] -and $_.Group }).Count -gt 0

    $script:Scan | Group-Object {
            if ($hasGroup) { $_.Group }
            else {
                $rel = Get-RelPath $_.Path $base
                $i = $rel.IndexOf('\')
                if ($i -ge 0) { $rel.Substring(0, $i) } else { '(gốc)' }
            }
        } |
        Sort-Object { ($_.Group | Measure-Object Size -Sum).Sum } -Descending |
        Format-Table @{n='Nhóm'; e={$_.Name}},
                     @{n='Tệp'; e={'{0:N0}' -f $_.Count}},
                     @{n='Dung lượng'; e={Show-Size (($_.Group | Measure-Object Size -Sum).Sum)}} -AutoSize |
        Out-String -Width 90 | Write-Host

    $script:Scan | Group-Object { $_.Date.Year } | Sort-Object Name |
        Format-Table @{n='Năm'; e={$_.Name}},
                     @{n='Tệp'; e={'{0:N0}' -f $_.Count}},
                     @{n='Dung lượng'; e={Show-Size (($_.Group | Measure-Object Size -Sum).Sum)}} -AutoSize |
        Out-String -Width 90 | Write-Host

    if ($script:ScanKind -eq 'BẢN TRÙNG LẶP') {
        Write-Host '  10 cặp trùng lớn nhất — xóa bên trên, giữ bên dưới' -ForegroundColor Cyan
        $script:Scan | Sort-Object Size -Descending | Select-Object -First 10 | ForEach-Object {
            Write-Host ('   xóa  ' + (Get-RelPath $_.Path $base)) -ForegroundColor DarkYellow
            Write-Host ('   giữ  ' + (Get-RelPath $_.Keeper $base) + '   [' + (Show-Size $_.Size) + ']') -ForegroundColor DarkGreen
            Write-Host ''
        }
    } else {
        Write-Host '  15 tệp lớn nhất' -ForegroundColor Cyan
        $script:Scan | Sort-Object Size -Descending | Select-Object -First 15 |
            Format-Table @{n='Ngày'; e={$_.Date.ToString('dd/MM/yyyy')}},
                         @{n='Kích thước'; e={Show-Size $_.Size}},
                         @{n='Đường dẫn'; e={Get-RelPath $_.Path $base}} -AutoSize |
            Out-String -Width 150 | Write-Host
    }

    if (Read-YesNo '  Xuất toàn bộ danh sách ra tệp CSV? (c/k)') {
        $f = Join-Path $script:LogDir ('quet_' + (Get-Date -Format 'yyyyMMdd_HHmmss') + '.csv')
        $script:Scan | Select-Object @{n='Ngay';e={$_.Date.ToString('yyyy-MM-dd HH:mm:ss')}},
                                     @{n='Bytes';e={$_.Size}}, Path, Keeper |
            Export-Csv -LiteralPath $f -NoTypeInformation -Encoding UTF8
        Write-Host ('  Đã xuất: ' + $f) -ForegroundColor Green
    }
}

# ================================================================ Zalo
function Test-ZaloRunning { return @(Get-Process -Name 'Zalo*' -ErrorAction SilentlyContinue) }

function Test-TouchesZalo {
    $base = Join-Path $env:APPDATA 'ZaloData'
    $paths = @($script:CleanupRoots)
    if ($paths.Count -eq 0) { $paths = @($script:ScanRoot) }
    foreach ($p in $paths) {
        if ([string]::IsNullOrWhiteSpace($p)) { continue }
        if ($p.StartsWith($base, [StringComparison]::OrdinalIgnoreCase)) { return $true }
    }
    return $false
}

function Close-Zalo {
    $p = Test-ZaloRunning
    if ($p.Count -eq 0) { return $true }
    Write-Host ''
    Write-Host ("  Zalo đang chạy ({0} tiến trình). Cần đóng trước khi thao tác." -f $p.Count) -ForegroundColor Yellow
    if (-not (Read-YesNo '  Đóng Zalo bây giờ? (c/k)')) { return $false }
    $p | ForEach-Object { try { $_.CloseMainWindow() | Out-Null } catch { } }
    Start-Sleep -Seconds 4
    $still = Test-ZaloRunning
    if ($still.Count -gt 0) { $still | Stop-Process -Force -ErrorAction SilentlyContinue; Start-Sleep -Seconds 3 }
    if ((Test-ZaloRunning).Count -gt 0) {
        Write-Host '  Vẫn còn tiến trình Zalo. Hãy đóng tay rồi thử lại.' -ForegroundColor Red
        return $false
    }
    Write-Host '  Đã đóng Zalo.' -ForegroundColor Green
    return $true
}

# ================================================================ sao lưu
function Invoke-Backup {
    if ($null -eq $script:Scan -or $script:Scan.Count -eq 0) { Write-Host 'Chưa có kết quả quét.' -ForegroundColor Yellow; return }
    $total = ($script:Scan | Measure-Object Size -Sum).Sum
    if ($null -eq $total) { $total = 0 }
    $base = $script:ScanRoot
    if ([string]::IsNullOrWhiteSpace($base)) { $base = $script:Root }

    Write-Title 'Sao lưu và xác minh'
    Get-PSDrive -PSProvider FileSystem | Where-Object { $null -ne $_.Free } |
        Format-Table @{n='Ổ'; e={$_.Name}}, @{n='Trống'; e={Show-Size $_.Free}} -AutoSize | Out-String | Write-Host
    Write-Host ('  Cần ít nhất: {0}' -f (Show-Size $total)) -ForegroundColor Yellow

    $dest = Read-Line '  Nhập thư mục đích (ví dụ D:\SaoLuuZalo)'
    if ($dest -eq '') { Write-Host '  Đã hủy.' -ForegroundColor DarkGray; return }

    try {
        $full = [IO.Path]::GetFullPath($dest)
        $qual = [IO.Path]::GetPathRoot($full)
        if ([string]::IsNullOrWhiteSpace($qual)) { throw 'không xác định được ổ đĩa' }
        $drv = Get-PSDrive -Name ($qual.TrimEnd('\', '/').TrimEnd(':')) -ErrorAction Stop
    } catch {
        Write-Host ('  Đường dẫn đích không hợp lệ: ' + $_.Exception.Message) -ForegroundColor Red
        return
    }
    $need = [long]($total * 1.02) + 100MB
    Write-Host ''
    Write-Host ('  Ổ {0} trống {1}, cần {2}' -f $drv.Name, (Show-Size $drv.Free), (Show-Size $need))
    if ($drv.Free -lt $need) {
        Write-Host ''
        Write-Host '  Không đủ chỗ. Đã dừng, chưa chép tệp nào.' -ForegroundColor Red
        Write-Host ('  Thiếu khoảng {0}. Chọn ổ khác hoặc thu hẹp phạm vi.' -f (Show-Size ($need - $drv.Free))) -ForegroundColor Red
        return
    }
    Write-Host '  Đủ chỗ.' -ForegroundColor Green

    Write-Host ''
    Write-Host '  Mức xác minh sau khi chép:'
    Write-Host '   1  Kích thước toàn bộ, cộng SHA256 mẫu 50 tệp  (nhanh, mặc định)'
    Write-Host '   2  SHA256 toàn bộ                              (chậm nhưng chắc chắn tuyệt đối)'
    $fullVerify = ((Read-Line '  Chọn (Enter = 1)') -eq '2')

    $stamp = Get-Date -Format 'yyyyMMdd_HHmmss'
    $destRun = Join-Path $dest $stamp
    New-Item -ItemType Directory -Force -Path $destRun | Out-Null
    Register-BackupRoot $dest

    $ok = 0; $fail = 0; $i = 0; $diskFull = $false
    $copied = New-Object Collections.Generic.List[object]
    $failLog = New-Object Collections.Generic.List[string]
    # Nhớ thư mục đã tạo để khỏi hỏi đĩa lại cho từng tệp. Sao lưu Zalo thường
    # đổ hàng vạn tệp vào vài trăm thư mục nên gần như lần nào cũng trúng.
    $madeDirs = New-Object 'Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)

    foreach ($f in $script:Scan) {
        $i++
        if ($i % 200 -eq 0) {
            Write-Progress -Activity 'Đang sao lưu' -Status ("{0:N0}/{1:N0} · lỗi {2:N0}" -f $i, $script:Scan.Count, $fail) `
                           -PercentComplete (($i / $script:Scan.Count) * 100)
        }
        try {
            $rel = Get-RelPath $f.Path $base

            # Get-RelPath trả về NGUYÊN đường dẫn tuyệt đối khi tệp không nằm
            # dưới $base. Ghép kiểu .NET với một đường dẫn tuyệt đối thì phần gốc
            # bị vứt đi và kết quả là chính đường dẫn tuyệt đối đó — bản sao lưu
            # sẽ ghi ra NGOÀI thư mục sao lưu, tệ nhất là đè lên chính tệp nguồn.
            # Bản cũ thoát nạn một cách tình cờ, nhờ Join-Path dựng ra đường dẫn
            # hỏng rồi mới ném lỗi. Ở đây chặn thẳng và nói rõ lý do.
            if ([IO.Path]::IsPathRooted($rel)) { throw 'tệp không nằm dưới gốc quét' }

            $target = $destRun + '\' + $rel
            $dir = [IO.Path]::GetDirectoryName($target)
            if (-not $madeDirs.Contains($dir)) {
                # Chỉ ghi nhớ sau khi tạo được thật, để một lần tạo hỏng không
                # làm mọi tệp sau đó tưởng nhầm là thư mục đã có.
                [void][IO.Directory]::CreateDirectory($dir)
                [void]$madeDirs.Add($dir)
            }
            [IO.File]::Copy($f.Path, $target, $true)
            $ok++
            $copied.Add([pscustomobject]@{ Src = $f.Path; Dst = $target; Rel = $rel; Size = $f.Size })
        } catch {
            $msg = $_.Exception.Message

            # Ổ đích hết chỗ thì DỪNG NGAY, đừng thử nốt hàng vạn tệp còn lại:
            # mỗi tệp sau đó cũng hỏng y hệt, chỉ tổ mất thời gian và làm nhật ký
            # ngập một loại lỗi duy nhất, che mất những lỗi thật sự khác nhau.
            # Invoke-Restore đã làm đúng như vậy từ đầu — đây là đưa Invoke-Backup
            # về ngang hàng.
            #
            # IO.File::Copy ném IOException thẳng nên mã lỗi nằm ngay ở Exception,
            # còn Copy-Item thì bọc thêm một lớp. Xét cả hai cho chắc.
            # 112 = ERROR_DISK_FULL, 39 = ERROR_HANDLE_DISK_FULL.
            $hr = 0
            try { $hr = $_.Exception.HResult -band 0xFFFF } catch { }
            if ($hr -ne 112 -and $hr -ne 39) {
                try { $hr = $_.Exception.InnerException.HResult -band 0xFFFF } catch { }
            }
            if ($hr -eq 112 -or $hr -eq 39 -or $msg -match 'not enough space|disk is full') {
                $diskFull = $true
                $failLog.Add($f.Path + ' => ổ đích hết chỗ, dừng tại đây')
                break
            }

            $fail++
            # Không chặn trần số dòng nhật ký lỗi. Sao lưu hỏng là chặn luôn bước
            # xóa, nên danh sách tệp hỏng chính là thứ người dùng cần đọc để quyết
            # định làm gì tiếp — cắt bớt nó là giấu đúng thứ họ cần nhất.
            $failLog.Add($f.Path + ' => ' + $msg)
        }
    }
    Write-Progress -Activity 'Đang sao lưu' -Completed

    if ($diskFull) {
        Write-Host ''
        Write-Host '  Ổ đích hết chỗ giữa chừng. Đã dừng ngay tại tệp đó.' -ForegroundColor Red
        Write-Host ('  Chép được {0:N0}/{1:N0} tệp trước khi hết chỗ.' -f $ok, $script:Scan.Count) -ForegroundColor Yellow
        Write-Host '  Bản sao lưu này KHÔNG trọn vẹn nên không mở đường xóa.' -ForegroundColor Red
    }

    Write-Host ''
    Write-Host '  Đang xác minh...' -ForegroundColor DarkGray
    $vFail = 0; $vChecked = 0
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        foreach ($c in $copied) {
            if (-not [IO.File]::Exists($c.Dst)) { $vFail++; $failLog.Add($c.Dst + ' => thiếu ở đích'); continue }
            if ([IO.FileInfo]::new($c.Dst).Length -ne $c.Size) {
                $vFail++; $failLog.Add($c.Dst + ' => lệch kích thước'); continue
            }
        }
        # $copied là List[object]. Trong PowerShell 5.1, toán tử @() áp lên
        # List[object] ném "Argument types do not match"; chỉ .Count gọi thẳng
        # mới chạy. Bẫy này từng làm hỏng nguyên nhánh xác minh SHA256 toàn bộ:
        # ai chọn mức chắc chắn nhất thì lại rơi đúng vào mức duy nhất bị lỗi,
        # còn mức mặc định thì thoát vì Get-Random đã trả ra mảng thật.
        # Nên đổi sang mảng bằng ToArray() rồi mới dùng, và đếm đúng một lần.
        $sample = $copied.ToArray()
        if (-not $fullVerify -and $sample.Count -gt 50) {
            $sample = @($sample | Get-Random -Count 50)
        }
        $sampleN = $sample.Count
        $j = 0
        foreach ($c in $sample) {
            $j++
            if ($j % 20 -eq 0) {
                Write-Progress -Activity 'Xác minh' -Status ("{0:N0}/{1:N0}" -f $j, $sampleN) `
                               -PercentComplete (($j / $sampleN) * 100)
            }
            try {
                if ((Get-Sha256Full $c.Src $sha) -ne (Get-Sha256Full $c.Dst $sha)) {
                    $vFail++; $failLog.Add($c.Dst + ' => hash không khớp')
                }
                $vChecked++
            } catch { $vFail++; $failLog.Add($c.Dst + ' => lỗi đọc khi xác minh') }
        }
    } finally {
        Write-Progress -Activity 'Xác minh' -Completed
        $sha.Dispose()
    }

    [pscustomobject]@{
        Tool = 'ZaloCleanup'; Version = 4
        Created = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss')
        SourceRoot = $base; ScanKind = $script:ScanKind
        Count = $ok; Bytes = ($copied | Measure-Object Size -Sum).Sum
        FullVerify = $fullVerify; Verified = $vChecked; VerifyFail = $vFail; CopyFail = $fail
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $destRun '_zalocleanup_backup.json') -Encoding UTF8

    # DiskFull và phép so Ok với Total là BẮT BUỘC, không phải để hiển thị.
    # Khi ổ đích hết chỗ, vòng chép dừng bằng break TRƯỚC lúc tăng $fail, nên
    # Fail vẫn bằng 0 dù bản sao lưu dở dang. Chỉ xét Fail thôi là mở khóa bước
    # xóa cho một bản sao lưu thiếu tệp — mất dữ liệu mà người dùng tin là đã
    # có đường lui. Ghi thẳng trạng thái trọn vẹn vào đây để Invoke-Delete xét.
    $script:LastBackup = [pscustomobject]@{
        ScanStamp = $script:ScanStamp; Dest = $destRun
        Total = $script:Scan.Count; Ok = $ok; Fail = $fail; VerifyFail = $vFail
        DiskFull = $diskFull
    }

    Write-Host ''
    Write-Host ("  Đã chép   : {0:N0} tệp" -f $ok) -ForegroundColor Green
    Write-Host ("  Xác minh  : {0:N0} tệp bằng SHA256" -f $vChecked) -ForegroundColor Green
    Write-Host ("  Vị trí    : {0}" -f $destRun) -ForegroundColor Green
    if ($fail -gt 0 -or $vFail -gt 0 -or $diskFull -or $ok -ne $script:Scan.Count) {
        $lf = Join-Path $script:LogDir ('saoluu_loi_' + $stamp + '.txt')
        $failLog | Set-Content -LiteralPath $lf -Encoding UTF8
        Write-Host ''
        if ($fail -gt 0)  { Write-Host ("  Chép lỗi     : {0:N0} tệp" -f $fail) -ForegroundColor Red }
        if ($vFail -gt 0) { Write-Host ("  Xác minh lỗi : {0:N0} tệp" -f $vFail) -ForegroundColor Red }
        if ($ok -ne $script:Scan.Count) {
            Write-Host ("  Thiếu        : {0:N0}/{1:N0} tệp chưa được chép" `
                        -f ($script:Scan.Count - $ok), $script:Scan.Count) -ForegroundColor Red
        }
        Write-Host  '  Bước xóa sẽ bị chặn cho đến khi sao lưu lại sạch.' -ForegroundColor Red
        Write-Host ("  Chi tiết: {0}" -f $lf) -ForegroundColor DarkGray
    } else {
        Write-Host '  Sao lưu sạch. Đã mở khóa bước xóa.' -ForegroundColor Green
    }
}

# ================================================================ khôi phục
# Đọc một thư mục bản sao lưu và trả về đối tượng mô tả, hoặc $null.
function Read-BackupSet($dir) {
    $idx = Join-Path $dir '_zalocleanup_backup.json'
    if (-not (Test-Path -LiteralPath $idx)) { return $null }
    try { return [pscustomobject]@{ Dir = $dir; Meta = (Get-Content -LiteralPath $idx -Raw | ConvertFrom-Json) } }
    catch { return $null }
}

# Tự đi tìm bản sao lưu thay vì bắt người dùng nhớ đường dẫn.
# Ưu tiên các thư mục đã từng dùng, sau đó quét nông các ổ đĩa.
function Find-Backups {
    $sets = @()
    $seen = @{}

    $candidates = @()
    foreach ($r in @($script:BackupRoots)) { if (Test-Path -LiteralPath $r) { $candidates += $r } }
    foreach ($d in @(Get-PSDrive -PSProvider FileSystem -ErrorAction SilentlyContinue | Where-Object { $null -ne $_.Free })) {
        $root = $d.Name + ':\'
        $candidates += $root
        foreach ($sub in @(Get-ChildItem -LiteralPath $root -Directory -Force -ErrorAction SilentlyContinue)) {
            $candidates += $sub.FullName
        }
    }

    foreach ($c in @($candidates | Select-Object -Unique)) {
        # bản thân thư mục có thể là một bản sao lưu
        $s = Read-BackupSet $c
        if ($null -ne $s -and -not $seen.ContainsKey($c)) { $seen[$c] = $true; $sets += $s; continue }
        foreach ($sub in @(Get-ChildItem -LiteralPath $c -Directory -Force -ErrorAction SilentlyContinue)) {
            if ($seen.ContainsKey($sub.FullName)) { continue }
            $s = Read-BackupSet $sub.FullName
            if ($null -ne $s) { $seen[$sub.FullName] = $true; $sets += $s }
        }
    }
    return @($sets | Sort-Object { $_.Meta.Created } -Descending)
}

# Mô tả bên trong bản sao lưu có gì, để người dùng biết mình đang chọn cái gì.
function Get-BackupSummary($set) {
    $files = @(Get-ChildItem -LiteralPath $set.Dir -Recurse -File -Force -ErrorAction SilentlyContinue |
               Where-Object { $_.Name -ne '_zalocleanup_backup.json' })
    if ($files.Count -eq 0) { return $null }
    $byTop = $files | Group-Object {
        $rel = Get-RelPath $_.FullName $set.Dir
        $i = $rel.IndexOf('\')
        if ($i -ge 0) { $rel.Substring(0, $i) } else { '(gốc)' }
    } | Sort-Object { ($_.Group | Measure-Object Length -Sum).Sum } -Descending
    $byExt = $files | Group-Object {
        $e = [IO.Path]::GetExtension($_.Name).ToLower()
        if ($e -eq '') { 'không đuôi' } else { $e }
    } | Sort-Object { ($_.Group | Measure-Object Length -Sum).Sum } -Descending
    return [pscustomobject]@{
        Files = $files.Count
        Bytes = ($files | Measure-Object Length -Sum).Sum
        Top   = @($byTop | Select-Object -First 3 | ForEach-Object { '{0} {1}' -f $_.Name, (Show-Size (($_.Group | Measure-Object Length -Sum).Sum)) })
        Ext   = @($byExt | Select-Object -First 3 | ForEach-Object { '{0} ({1:N0})' -f $_.Name, $_.Count })
        Oldest = ($files | Measure-Object LastWriteTime -Minimum).Minimum
        Newest = ($files | Measure-Object LastWriteTime -Maximum).Maximum
        Sample = @($files | Sort-Object Length -Descending | Select-Object -First 3 |
                   ForEach-Object { '{0}  [{1}]' -f (Get-RelPath $_.FullName $set.Dir), (Show-Size $_.Length) })
    }
}

function Invoke-Restore {
    Write-Title 'Khôi phục dữ liệu đã sao lưu'
    Write-Host '  Đang tìm các bản sao lưu trên máy...' -ForegroundColor DarkGray
    $sets = @(Find-Backups)

    if ($sets.Count -eq 0) {
        Write-Host ''
        Write-Host '  Không tìm thấy bản sao lưu nào do công cụ này tạo ra.' -ForegroundColor Yellow
        Write-Host '  Công cụ đã tìm trong các thư mục từng dùng và các ổ đĩa.' -ForegroundColor DarkGray
        Write-Host ''
        $man = Read-Line '  Nhập đường dẫn thủ công nếu bạn biết (Enter để quay lại)'
        if ($man -eq '' -or -not (Test-Path -LiteralPath $man)) { return }
        $sets = @()
        $s = Read-BackupSet $man
        if ($null -ne $s) { $sets += $s }
        foreach ($sub in @(Get-ChildItem -LiteralPath $man -Directory -Force -ErrorAction SilentlyContinue)) {
            $s = Read-BackupSet $sub.FullName
            if ($null -ne $s) { $sets += $s }
        }
        if ($sets.Count -eq 0) { Write-Host '  Vẫn không thấy bản sao lưu nào ở đó.' -ForegroundColor Red; return }
    }

    Write-Host ''
    Write-Host ('  Tìm thấy {0} bản sao lưu:' -f $sets.Count) -ForegroundColor Green
    Write-Host ''
    for ($i = 0; $i -lt $sets.Count; $i++) {
        $m = $sets[$i].Meta
        Write-Host ("   {0}  {1}   {2:N0} tệp · {3}" -f ($i + 1), $m.Created, $m.Count, (Show-Size ([long]$m.Bytes))) -ForegroundColor Cyan
        Write-Host ("      Nội dung : {0}" -f $m.ScanKind) -ForegroundColor DarkGray
        $sm = Get-BackupSummary $sets[$i]
        if ($null -ne $sm) {
            Write-Host ("      Gồm      : {0}" -f (($sm.Top) -join ' · ')) -ForegroundColor DarkGray
            Write-Host ("      Loại tệp : {0}" -f (($sm.Ext) -join ' · ')) -ForegroundColor DarkGray
            Write-Host ("      Tệp từ   : {0} đến {1}" -f $sm.Oldest.ToString('dd/MM/yyyy'), $sm.Newest.ToString('dd/MM/yyyy')) -ForegroundColor DarkGray
        }
        Write-Host ("      Nằm ở    : {0}" -f $sets[$i].Dir) -ForegroundColor DarkGray
        Write-Host ("      Trả về   : {0}" -f $m.SourceRoot) -ForegroundColor DarkGray
        if ($m.CopyFail -gt 0 -or $m.VerifyFail -gt 0) {
            Write-Host ("      Cảnh báo : bản này từng lỗi (chép {0}, xác minh {1})" -f $m.CopyFail, $m.VerifyFail) -ForegroundColor Red
        }
        Write-Host ''
    }
    Write-Host '   Gõ số để khôi phục, hoặc  x <số>  để xem danh sách tệp bên trong'
    Write-Host '   Enter để quay lại'
    $ans = Read-Line '  Chọn'
    if ($ans -eq '') { Write-Host '  Đã hủy.' -ForegroundColor DarkGray; return }

    if ($ans -match '^[xX]\s*(\d+)$') {
        $k = [int]$Matches[1]
        if ($k -lt 1 -or $k -gt $sets.Count) { Write-Host '  Số không hợp lệ.' -ForegroundColor Red; return }
        $sm = Get-BackupSummary $sets[$k - 1]
        Write-Host ''
        Write-Host ('  Ba tệp lớn nhất trong bản {0}:' -f $k) -ForegroundColor Cyan
        $sm.Sample | ForEach-Object { Write-Host ('   ' + $_) -ForegroundColor DarkGray }
        Write-Host ''
        Write-Host ('  Toàn bộ nằm tại: ' + $sets[$k - 1].Dir) -ForegroundColor DarkGray
        Write-Host  '  Mở thư mục đó bằng File Explorer để xem đầy đủ.' -ForegroundColor DarkGray
        return
    }

    $n = 0
    if (-not ([int]::TryParse($ans, [ref]$n)) -or $n -lt 1 -or $n -gt $sets.Count) {
        Write-Host '  Số không hợp lệ. Đã hủy.' -ForegroundColor Red; return
    }
    $set = $sets[$n - 1]
    $target = $set.Meta.SourceRoot

    Write-Host ''
    Write-Host '  Nếu tệp đích đã tồn tại:'
    Write-Host '   1  Bỏ qua, giữ tệp hiện có   (mặc định, an toàn nhất)'
    Write-Host '   2  Ghi đè bằng bản sao lưu'
    $overwrite = ((Read-Line '  Chọn (Enter = 1)') -eq '2')

    $files = @(Get-ChildItem -LiteralPath $set.Dir -Recurse -File -Force -ErrorAction SilentlyContinue |
               Where-Object { $_.Name -ne '_zalocleanup_backup.json' })

    while ($true) {
        Write-Host ''
        Write-Host ('  Sẽ khôi phục về: ' + $target) -ForegroundColor Yellow
        if (-not (Test-Path -LiteralPath $target)) { Write-Host '  Thư mục đích không còn tồn tại. Sẽ được tạo lại.' -ForegroundColor Yellow }

        Write-Host '  Đang tính dung lượng cần thiết...' -ForegroundColor DarkGray
        $need = [long]0; $willWrite = 0; $willSkip = 0
        foreach ($f in $files) {
            $dst = Join-Path $target (Get-RelPath $f.FullName $set.Dir)
            if (Test-Path -LiteralPath $dst) {
                if (-not $overwrite) { $willSkip++; continue }
                $cur = 0
                try { $cur = (New-Object IO.FileInfo $dst).Length } catch { }
                if (($f.Length - $cur) -gt 0) { $need += ($f.Length - $cur) }
                $willWrite++
            } else { $need += $f.Length; $willWrite++ }
        }
        $need = [long]($need * 1.02) + 50MB

        try {
            $qual = [IO.Path]::GetPathRoot([IO.Path]::GetFullPath($target))
            $drv = Get-PSDrive -Name ($qual.TrimEnd('\', '/').TrimEnd(':')) -ErrorAction Stop
        } catch {
            Write-Host ('  Không đọc được dung lượng ổ đích: ' + $_.Exception.Message) -ForegroundColor Red; return
        }

        Write-Host ''
        Write-Host ('  Sẽ ghi  : {0:N0} tệp' -f $willWrite)
        if ($willSkip -gt 0) { Write-Host ('  Bỏ qua  : {0:N0} tệp đã tồn tại' -f $willSkip) -ForegroundColor DarkGray }
        Write-Host ('  Cần chỗ : {0}' -f (Show-Size $need))
        Write-Host ('  Ổ {0} trống: {1}' -f $drv.Name, (Show-Size $drv.Free))

        if ($willWrite -eq 0) { Write-Host ''; Write-Host '  Không có gì để khôi phục.' -ForegroundColor Green; return }
        if ($drv.Free -ge $need) { Write-Host '  Đủ chỗ.' -ForegroundColor Green; break }

        Write-Host ''
        Write-Host ('  Không đủ chỗ trên ổ {0}. Thiếu khoảng {1}.' -f $drv.Name, (Show-Size ($need - $drv.Free))) -ForegroundColor Red
        Write-Host  '  Chưa ghi tệp nào. Khôi phục nửa chừng rồi hết chỗ sẽ để lại trạng thái' -ForegroundColor Red
        Write-Host  '  dở dang, nên công cụ dừng hẳn thay vì làm liều.' -ForegroundColor Red
        Write-Host ''
        Write-Host '   1  Khôi phục sang thư mục khác rồi tự chép về sau'
        Write-Host '   2  Nhập lại đường dẫn đích khác'
        Write-Host '   3  Tôi đã giải phóng thêm chỗ, đo lại'
        Write-Host '   Enter để hủy'
        $opt = Read-Line '  Chọn'
        if ($opt -eq '1' -or $opt -eq '2') {
            Get-PSDrive -PSProvider FileSystem | Where-Object { $null -ne $_.Free } |
                Format-Table @{n='Ổ'; e={$_.Name}}, @{n='Trống'; e={Show-Size $_.Free}} -AutoSize | Out-String | Write-Host
            $alt = Read-Line '  Đường dẫn đích mới'
            if ($alt -eq '') { Write-Host '  Đã hủy.' -ForegroundColor DarkGray; return }
            $target = $alt
            Write-Host ''
            Write-Host '  Lưu ý: khôi phục ra ngoài vị trí gốc thì Zalo sẽ không thấy dữ liệu này.' -ForegroundColor Yellow
            continue
        }
        elseif ($opt -eq '3') { continue }
        else { Write-Host '  Đã hủy. Không ghi gì.' -ForegroundColor DarkGray; return }
    }

    if ($target.StartsWith((Join-Path $env:APPDATA 'ZaloData'), [StringComparison]::OrdinalIgnoreCase)) {
        if (-not (Close-Zalo)) { Write-Host '  Dừng lại, chưa khôi phục gì.' -ForegroundColor Yellow; return }
    }

    $ok = 0; $skip = 0; $fail = 0; $i = 0; $diskFull = $false
    $logFile = Join-Path $script:LogDir ('khoiphuc_' + (Get-Date -Format 'yyyyMMdd_HHmmss') + '.log')
    $w = New-Object IO.StreamWriter($logFile, $false, [Text.Encoding]::UTF8)
    $w.WriteLine('# Nhật ký khôi phục')
    $w.WriteLine('# Từ      : ' + $set.Dir)
    $w.WriteLine('# Về      : ' + $target)
    $w.WriteLine('# Bắt đầu : ' + (Get-Date).ToString('dd/MM/yyyy HH:mm:ss'))

    try {
        foreach ($f in $files) {
            $i++
            if ($i % 200 -eq 0) {
                Write-Progress -Activity 'Đang khôi phục' -Status ("{0:N0}/{1:N0}" -f $i, $files.Count) `
                               -PercentComplete (($i / $files.Count) * 100)
                $w.Flush()
            }
            try {
                $rel = Get-RelPath $f.FullName $set.Dir
                $dst = Join-Path $target $rel
                if ((Test-Path -LiteralPath $dst) -and -not $overwrite) { $skip++; $w.WriteLine("BỎQUA`t" + $rel); continue }
                $dir = Split-Path $dst -Parent
                if (-not (Test-Path -LiteralPath $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
                Copy-Item -LiteralPath $f.FullName -Destination $dst -Force -ErrorAction Stop
                $ok++; $w.WriteLine("KHÔIPHỤC`t" + $rel)
            } catch {
                $msg = $_.Exception.Message
                $hr = 0
                try { $hr = $_.Exception.InnerException.HResult -band 0xFFFF } catch { }
                if ($hr -eq 112 -or $hr -eq 39 -or $msg -match 'not enough space|disk is full') {
                    $diskFull = $true; $w.WriteLine("HẾTCHỖ`t" + $f.FullName); break
                }
                $fail++; $w.WriteLine("THẤTBẠI`t" + $f.FullName + "`t" + $msg)
            }
        }
    } finally {
        Write-Progress -Activity 'Đang khôi phục' -Completed
        if ($diskFull) { $w.WriteLine('# Dừng sớm: ổ đích hết chỗ') }
        $w.WriteLine("# Tổng kết: khôi phục=$ok bỏ qua=$skip thất bại=$fail hết chỗ=$diskFull")
        $w.Flush(); $w.Dispose()
    }

    Reset-TreeCache
    Write-Host ''
    if ($diskFull) {
        Write-Host '  Dừng sớm: ổ đích hết chỗ giữa chừng.' -ForegroundColor Red
        Write-Host ('  Đã khôi phục {0:N0}/{1:N0} tệp trước khi hết chỗ.' -f $ok, $files.Count) -ForegroundColor Yellow
        Write-Host  '  Bản sao lưu vẫn nguyên vẹn. Giải phóng thêm chỗ rồi chạy lại là an toàn.' -ForegroundColor Green
    } else {
        Write-Host ('  Đã khôi phục: {0:N0} tệp' -f $ok) -ForegroundColor Green
    }
    if ($skip -gt 0) { Write-Host ('  Bỏ qua (đã có): {0:N0} tệp' -f $skip) -ForegroundColor Yellow }
    if ($fail -gt 0) { Write-Host ('  Thất bại      : {0:N0} tệp' -f $fail) -ForegroundColor Red }
    Write-Host ('  Nhật ký: ' + $logFile) -ForegroundColor DarkGray
}

# ================================================================ xóa
# Bản sao lưu chỉ được coi là SẠCH khi vừa không lỗi vừa TRỌN VẸN. Hai vế, không
# phải một — và vế thứ hai mới là vế dễ mất.
#
# Khi ổ đích hết chỗ giữa chừng, vòng chép trong Invoke-Backup thoát bằng break
# TRƯỚC lúc tăng $fail. Nên nếu chỉ xét Fail thì một bản sao lưu thiếu tệp vẫn
# được chấm là sạch, và bước xóa được mở khóa cho một đường lui không tồn tại.
#
# Tách ra thành hàm riêng để phép thử gọi thẳng được. Ca ổ đích hết chỗ không
# dựng lại được trong sandbox nếu không tạo ổ đĩa ảo; để điều kiện này nằm chìm
# trong Invoke-Delete thì nó vĩnh viễn không có phép thử nào canh — mà bài học
# đắt nhất của công cụ này là một nhánh không có test là một nhánh chưa từng chạy.
function Test-BackupClean($bk, $scanStamp) {
    if ($null -eq $bk) { return $false }
    if ($bk.ScanStamp -ne $scanStamp) { return $false }
    if ($bk.Fail -ne 0 -or $bk.VerifyFail -ne 0) { return $false }
    if ($bk.DiskFull) { return $false }
    return ($bk.Ok -eq $bk.Total)
}

function Invoke-Delete {
    if ($null -eq $script:Scan -or $script:Scan.Count -eq 0) { Write-Host 'Chưa có kết quả quét.' -ForegroundColor Yellow; return }
    $total = ($script:Scan | Measure-Object Size -Sum).Sum
    if ($null -eq $total) { $total = 0 }

    # Dữ liệu thật thì cần xác nhận nặng; bản trùng lặp và cache thì nhẹ hơn.
    $isRealData = ($script:ScanKind -eq 'DỮ LIỆU ZALO')

    Write-Title ('Xóa · ' + $script:ScanKind)
    Write-Host ('  Số tệp     : {0:N0}' -f $script:Scan.Count) -ForegroundColor Yellow
    Write-Host ('  Dung lượng : {0}' -f (Show-Size $total)) -ForegroundColor Yellow

    if ($script:ScanKind -eq 'BẢN TRÙNG LẶP') {
        Write-Host ''
        Write-Host '  Mỗi tệp đã được xác minh bằng SHA256 là giống hệt một bản khác đang' -ForegroundColor Green
        Write-Host '  được giữ lại. Bạn sẽ không mất tấm ảnh hay đoạn video nào.' -ForegroundColor Green
    } elseif ($script:ScanKind -eq 'CACHE ZALO') {
        Write-Host ''
        Write-Host '  Cache của ứng dụng. Zalo tự tạo lại khi cần.' -ForegroundColor Green
    } elseif ($script:ScanKind -eq 'CACHE HỆ THỐNG') {
        Write-Host ''
        Write-Host '  Cache hệ thống từ danh sách đã kiểm chứng. Ứng dụng tự tạo lại.' -ForegroundColor Green
        Write-Host '  Mục màu vàng phải tải lại từ mạng, có thể tốn băng thông.' -ForegroundColor Yellow
    } else {
        Write-Host ('  Khoảng thời gian : {0}' -f (Show-DateRangeLabel $script:FromDate $script:ToDate))
        Write-Host ''
        Write-Host '  Đây là dữ liệu thật. Tệp sẽ bị xóa hẳn, không qua Thùng rác,' -ForegroundColor Red
        Write-Host '  không khôi phục được. Ảnh và video quá hạn lưu trên máy chủ Zalo' -ForegroundColor Red
        Write-Host '  sẽ mất vĩnh viễn.' -ForegroundColor Red
    }

    # Chính sách sao lưu — chỉ áp dụng cho dữ liệu thật
    $hasBackup = ($null -ne $script:LastBackup -and $script:LastBackup.ScanStamp -eq $script:ScanStamp)
    $cleanBackup = Test-BackupClean $script:LastBackup $script:ScanStamp

    if ($isRealData -and $script:BackupPolicy -eq 'BATBUOC' -and -not $cleanBackup) {
        Write-Host ''
        Write-Host '  Đã chặn: chính sách hiện tại là bắt buộc sao lưu.' -ForegroundColor Red
        Write-Host  '  Hãy sao lưu trong Tùy chọn nâng cao rồi quay lại xóa.' -ForegroundColor Yellow
        return
    }

    if ($isRealData -and $script:BackupPolicy -eq 'HOI' -and -not $hasBackup) {
        Write-Host ''
        Write-Host '  Kết quả này chưa được sao lưu. Sao lưu là cách duy nhất để còn đường lui.' -ForegroundColor Yellow
        Write-Host ''
        Write-Host '   1  Sao lưu trước rồi xóa'
        Write-Host '   2  Xóa luôn, không sao lưu'
        Write-Host '   Enter để hủy'
        $bc = Read-Line '  Chọn'
        if ($bc -eq '1') {
            Invoke-Backup
            if ($null -eq $script:LastBackup -or $script:LastBackup.ScanStamp -ne $script:ScanStamp) {
                Write-Host ''; Write-Host '  Chưa sao lưu được. Dừng lại, chưa xóa gì.' -ForegroundColor Yellow; return
            }
        }
        elseif ($bc -eq '2') { Write-Host '  Đã chọn xóa mà không sao lưu.' -ForegroundColor DarkGray }
        else { Write-Host '  Đã hủy. Không tệp nào bị đụng đến.' -ForegroundColor Green; return }
    }

    $bk = $script:LastBackup
    if ($null -ne $bk -and $bk.ScanStamp -eq $script:ScanStamp -and ($bk.Fail -gt 0 -or $bk.VerifyFail -gt 0)) {
        Write-Host ''
        Write-Host ('  Đã chặn: sao lưu chưa sạch (chép lỗi {0:N0}, xác minh lỗi {1:N0}).' -f $bk.Fail, $bk.VerifyFail) -ForegroundColor Red
        Write-Host  '  Xóa bây giờ sẽ mất vĩnh viễn đúng những tệp chưa được sao lưu tốt.' -ForegroundColor Red
        Write-Host ''
        if (-not (Test-ConfirmPhrase (Read-Line '  Vẫn muốn xóa? Gõ chính xác:  TÔI CHẤP NHẬN MẤT') 'TÔI CHẤP NHẬN MẤT' 'TOI CHAP NHAN MAT')) {
            Write-Host '  Đã hủy. Không tệp nào bị đụng đến.' -ForegroundColor Green; return
        }
    }

    Write-Host ''
    if ($isRealData) {
        if (-not (Test-ConfirmPhrase (Read-Line '  Gõ đúng chữ  XÓA  để xác nhận') 'XÓA' 'XOA')) {
            Write-Host '  Đã hủy. Không tệp nào bị đụng đến.' -ForegroundColor Green; return
        }
    } else {
        if (-not (Read-YesNo '  Xóa luôn? (c/k)')) {
            Write-Host '  Đã hủy. Không tệp nào bị đụng đến.' -ForegroundColor Green; return
        }
    }

    if (Test-TouchesZalo) {
        if (-not (Close-Zalo)) { Write-Host '  Dừng lại, chưa xóa gì.' -ForegroundColor Yellow; return }
    }

    $freeBefore = Get-FreeBytes $script:ScanRoot
    Write-Host ''
    Show-EnvironmentWarnings
    Write-Host '  Đang xóa. Có thể bấm Ctrl+C để dừng an toàn bất cứ lúc nào.' -ForegroundColor DarkGray

    $logFile = Join-Path $script:LogDir ('daxoa_' + (Get-Date -Format 'yyyyMMdd_HHmmss') + '.log')
    $w = New-Object IO.StreamWriter($logFile, $false, [Text.Encoding]::UTF8)
    $w.WriteLine('# Nhật ký xóa')
    $w.WriteLine('# Chế độ  : ' + $script:ScanKind)
    if ($hasBackup) { $w.WriteLine('# Sao lưu : có → ' + $script:LastBackup.Dest) }
    else { $w.WriteLine('# Sao lưu : không') }
    $w.WriteLine('# Quét lúc: ' + $script:ScanStamp)
    $w.WriteLine('# Bắt đầu : ' + (Get-Date).ToString('dd/MM/yyyy HH:mm:ss'))
    $w.WriteLine('# Cột: TRẠNGTHÁI<TAB>BYTES<TAB>ĐƯỜNGDẪN')
    $w.Flush()

    $ok = 0; $fail = 0; $missing = 0; $guarded = 0; $trunc = 0; $freed = [long]0; $i = 0
    $errs = New-Object Collections.Generic.List[string]
    $completed = $false
    # Cắt cụt chỉ dành cho cache. Xem ghi chú ở Invoke-TruncateLocked.
    $mayTruncate = ($script:ScanKind -eq 'CACHE ZALO' -or $script:ScanKind -eq 'CACHE HỆ THỐNG')
    $sw = [Diagnostics.Stopwatch]::StartNew()

    try {
        foreach ($f in $script:Scan) {
            $i++
            if ($i % 200 -eq 0) {
                Write-Progress -Activity 'Đang xóa' -Status ("{0:N0}/{1:N0} · lỗi {2:N0}" -f $i, $script:Scan.Count, $fail) `
                               -PercentComplete (($i / $script:Scan.Count) * 100)
            }
            if (Test-Protected $f.Path) { $guarded++; $w.WriteLine("VÙNGBẢOVỆ`t0`t" + $f.Path); continue }
            # Đường dẫn dài cần tiền tố đặc biệt khi Windows tắt hỗ trợ long path
            $lp = Get-LongPath $f.Path

            # Một đối tượng FileInfo thay cho File::Exists cộng New-Object.
            # FileInfo đọc metadata một lần rồi nhớ lại nên bớt được một lượt hỏi
            # đĩa cho mỗi tệp, và ::new() không đi qua tầng cmdlet như New-Object.
            # Đo trên 20.000 tệp: 0,69 ms xuống 0,39 ms mỗi tệp.
            # Dựng trong try vì đường dẫn dị dạng làm hàm dựng ném lỗi, mà bản cũ
            # gặp ca đó thì rơi vào nhánh BIẾNMẤT — giữ nguyên kết cục ấy.
            $fi = $null
            try { $fi = [IO.FileInfo]::new($lp) } catch { }
            if ($null -eq $fi -or -not $fi.Exists) { $missing++; $w.WriteLine("BIẾNMẤT`t0`t" + $f.Path); continue }

            $actual = 0
            try {
                # Đọc cỡ THẬT lúc này chứ không lấy cỡ ghi lúc quét: tệp có thể
                # đã đổi giữa lúc quét và lúc xóa, mà con số này đi thẳng vào
                # nhật ký và vào tổng dung lượng báo đã thu hồi.
                $actual = $fi.Length
                if ($fi.Attributes -band [IO.FileAttributes]::ReadOnly) {
                    $fi.Attributes = $fi.Attributes -bxor [IO.FileAttributes]::ReadOnly
                }
            } catch { }

            try {
                [IO.File]::Delete($lp)
                if ([IO.File]::Exists($lp)) {
                    $fail++; $w.WriteLine("THẤTBẠI`t$actual`t" + $f.Path)
                    if ($errs.Count -lt 30) { $errs.Add($f.Path + ' => vẫn còn sau khi xóa') }
                } else {
                    $ok++; $freed += $actual
                    $w.WriteLine("ĐÃXÓA`t$actual`t" + $f.Path)
                }
            } catch [IO.IOException], [UnauthorizedAccessException] {
                # Hai loại lỗi này nghĩa là tệp đang bị khóa hoặc bị chặn quyền.
                # Với cache thì cắt cụt về 0 byte vẫn thu lại được dung lượng.
                if ($mayTruncate -and $actual -gt 0 -and (Invoke-TruncateLocked $lp)) {
                    $trunc++; $freed += $actual
                    $w.WriteLine("CẮTCỤT`t$actual`t" + $f.Path)
                } else {
                    $fail++; $w.WriteLine("THẤTBẠI`t$actual`t" + $f.Path)
                    if ($errs.Count -lt 30) { $errs.Add($f.Path + ' => ' + $_.Exception.Message) }
                }
            } catch {
                $fail++; $w.WriteLine("THẤTBẠI`t$actual`t" + $f.Path)
                if ($errs.Count -lt 30) { $errs.Add($f.Path + ' => ' + $_.Exception.Message) }
            }
            if ($i % 100 -eq 0) { $w.Flush() }
        }
        $completed = $true
    } finally {
        Write-Progress -Activity 'Đang xóa' -Completed
        $sw.Stop()
        if (-not $completed) { $w.WriteLine('# Đã hủy giữa chừng lúc ' + (Get-Date).ToString('dd/MM/yyyy HH:mm:ss')) }
        $w.WriteLine("# Tổng kết: đã xóa=$ok cắt cụt=$trunc thất bại=$fail biến mất=$missing vùng bảo vệ=$guarded bytes=$freed hoàn tất=$completed")
        $w.Flush(); $w.Dispose()
    }

    $roots = $script:CleanupRoots
    if (@($roots).Count -eq 0) { $roots = @($script:ScanRoot) }
    $rmDir = Remove-EmptyDirs $roots ($script:ScanKind -ne 'CACHE HỆ THỐNG')

    Reset-TreeCache
    Write-Host ''
    if (-not $completed) { Write-Host '  Đã hủy giữa chừng — số liệu dưới là phần đã làm xong.' -ForegroundColor Yellow }
    Write-Host ('  Đã xóa       : {0:N0} tệp' -f $ok) -ForegroundColor Green
    Write-Host ('  Giải phóng   : {0}' -f (Show-Size $freed)) -ForegroundColor Green
    Write-Host ('  Thư mục rỗng : {0}' -f $rmDir) -ForegroundColor Green
    Write-Host ('  Thời gian    : {0:N1} giây' -f $sw.Elapsed.TotalSeconds) -ForegroundColor DarkGray
    if ($missing -gt 0) { Write-Host ('  Biến mất trước khi xóa: {0:N0} tệp (tiến trình khác đã xóa)' -f $missing) -ForegroundColor Yellow }
    if ($guarded -gt 0) { Write-Host ('  Chặn bởi vùng bảo vệ  : {0:N0} tệp' -f $guarded) -ForegroundColor Yellow }
    if ($trunc -gt 0) {
        Write-Host ('  Cắt cụt      : {0:N0} tệp đang bị khóa' -f $trunc) -ForegroundColor Yellow
        Write-Host  '                 Tên tệp còn đó nhưng nội dung đã rỗng, dung lượng đã được thu về.' -ForegroundColor DarkGray
        Write-Host  '                 Ứng dụng giữ chúng sẽ tự dọn hoặc tự tạo lại.' -ForegroundColor DarkGray
    }
    if ($fail -gt 0) {
        Write-Host ('  Thất bại     : {0:N0} tệp (đang bị khóa)' -f $fail) -ForegroundColor Red
        $errs | Select-Object -First 5 | ForEach-Object { Write-Host ('    ' + $_) -ForegroundColor DarkRed }
    }
    Write-Host ('  Nhật ký      : {0}' -f $logFile) -ForegroundColor DarkGray

    $freeAfter = Get-FreeBytes $script:ScanRoot
    if ($freeBefore -ge 0 -and $freeAfter -ge 0) {
        $realGain = $freeAfter - $freeBefore
        $gainTxt = if ($realGain -ge 0) { '+' + (Show-Size $realGain) } else { '−' + (Show-Size ([Math]::Abs($realGain))) }
        Write-Host ''
        Write-Host ('  Ổ đĩa trước  : {0}' -f (Show-Size $freeBefore)) -ForegroundColor Cyan
        Write-Host ('  Ổ đĩa sau    : {0}' -f (Show-Size $freeAfter)) -ForegroundColor Cyan
        Write-Host ('  Thực tế thu được: {0}' -f $gainTxt) -ForegroundColor Cyan

        if ($freed -gt 500MB -and $realGain -lt ($freed * 0.5)) {
            Write-Host ''
            Write-Host '  ─── Cảnh báo: chưa thu được dung lượng thật ───' -ForegroundColor Red
            Write-Host ('  Đã xóa {0} nhưng ổ đĩa chỉ rộng thêm {1}.' -f (Show-Size $freed), $gainTxt) -ForegroundColor Yellow
            Write-Host  '  Nguyên nhân gần như chắc chắn là Shadow Copy của System Restore.'
            Write-Host  '  Khi tồn tại bản chụp, Windows phải giữ lại nội dung cũ của tệp bị xóa,'
            Write-Host  '  nên nó chép các khối đó sang vùng chụp. Dung lượng đổi chủ chứ không'
            Write-Host  '  được trả về ổ đĩa.'
            Write-Host  '  Vào Tùy chọn nâng cao → Shadow Copy để xử lý.' -ForegroundColor Green
        }
    }

    Reset-Scan
}

# ================================================================ shadow copy
# In nguyên văn đầu ra của vssadmin, chỉ bỏ dòng tiêu đề và bản quyền.
# Không lọc theo từ khóa tiếng Anh vì vssadmin bị bản địa hóa theo ngôn ngữ Windows.
function Show-VssStorage {
    $shown = 0
    foreach ($l in (& vssadmin list shadowstorage 2>&1)) {
        $t = "$l".Trim()
        if ($t -eq '') { continue }
        if ($t -match '^vssadmin\s') { continue }
        if ($t -match '(?i)copyright') { continue }
        Write-Host ('   ' + $t)
        $shown++
    }
    if ($shown -eq 0) { Write-Host '   (vssadmin không trả về dữ liệu)' -ForegroundColor DarkGray }
}

function Show-ShadowReport {
    Write-Title 'Shadow Copy — vì sao xóa mà ổ đĩa không rộng ra'
    Write-Host '  Shadow Copy của System Restore dùng cơ chế sao chép khi ghi. Khi tồn tại'
    Write-Host '  bản chụp và bạn xóa tệp đã có mặt lúc chụp, Windows phải giữ lại nội dung'
    Write-Host '  cũ cho bản chụp, nên nó chép các khối đó sang vùng chụp trước khi cho xóa.'
    Write-Host '  Bạn giải phóng X byte khỏi hệ thống tệp và tiêu tốn đúng X byte ở vùng chụp.'
    Write-Host ''
    Write-Host '  Thứ tự thao tác quyết định kết quả:' -ForegroundColor Green
    Write-Host '    Dọn xong rồi mới nhả bản chụp  →  lấy được dung lượng thật' -ForegroundColor Green
    Write-Host '    Dọn khi đang có bản chụp        →  mất trắng vào vùng chụp' -ForegroundColor Yellow
    Write-Host ''
    Write-Host ('  Ổ {0} đang trống: {1}' -f (Get-DriveLabel $script:SysRoot), (Show-Size (Get-FreeBytes $script:SysRoot))) -ForegroundColor Cyan

    if (-not (Test-IsAdmin)) {
        Write-Host ''
        Write-Host '  Cần quyền quản trị để xem và xử lý Shadow Copy.' -ForegroundColor Yellow
        Write-Host ''
        Write-Host '   1  Mở lại công cụ với quyền quản trị'
        Write-Host '   Enter để quay lại'
        if ((Read-Line '  Chọn') -eq '1') { Restart-Elevated | Out-Null }
        return
    }

    Write-Host ''
    Show-VssStorage

    while ($true) {
        Write-Host ''
        Write-Host '   1  Hạ trần shadow storage    (giữ vài bản chụp gần nhất)'
        Write-Host '   2  Xóa bản chụp cũ nhất'
        Write-Host '   3  Xóa toàn bộ bản chụp      (mất hết điểm khôi phục)' -ForegroundColor Yellow
        Write-Host '   4  Xem lại tình trạng'
        Write-Host '   Enter để quay lại'
        $c = Read-Line '  Chọn'
        if ($c -eq '') { return }

        $b = Get-FreeBytes $script:SysRoot
        switch ($c) {
            '1' {
                $n = 0
                if (-not ([int]::TryParse((Read-Line '  Trần mới tính bằng GB (ví dụ 5)'), [ref]$n)) -or $n -lt 1) {
                    Write-Host '  Giá trị không hợp lệ. Không đổi gì.' -ForegroundColor Red; continue
                }
                Write-Host ('  Windows sẽ xóa các bản chụp cũ nhất cho vừa trần {0} GB.' -f $n) -ForegroundColor Yellow
                if (-not (Read-YesNo '  Tiếp tục? (c/k)')) { Write-Host '  Đã hủy.' -ForegroundColor DarkGray; continue }
                & vssadmin resize shadowstorage "/for=$($script:SysDrive)" "/on=$($script:SysDrive)" "/maxsize=${n}GB" 2>&1 |
                    ForEach-Object { '   ' + $_ }
            }
            '2' {
                if (-not (Read-YesNo '  Xóa bản chụp cũ nhất trên ổ C? (c/k)')) { Write-Host '  Đã hủy.' -ForegroundColor DarkGray; continue }
                & vssadmin delete shadows "/for=$($script:SysDrive)" /oldest /quiet 2>&1 | ForEach-Object { '   ' + $_ }
            }
            '3' {
                Write-Host ''
                Write-Host '  Cảnh báo: xóa hết bản chụp sẽ mất toàn bộ điểm khôi phục System Restore' -ForegroundColor Red
                Write-Host '  và tính năng Previous Versions. Bạn sẽ không quay lui được Windows khi' -ForegroundColor Red
                Write-Host '  cài driver hay cập nhật gây lỗi.' -ForegroundColor Red
                Write-Host ''
                if (-not (Test-ConfirmPhrase (Read-Line '  Gõ chính xác:  XÓA HẾT BẢN CHỤP') 'XÓA HẾT BẢN CHỤP' 'XOA HET BAN CHUP')) {
                    Write-Host '  Đã hủy.' -ForegroundColor Green; continue
                }
                & vssadmin delete shadows "/for=$($script:SysDrive)" /all /quiet 2>&1 | ForEach-Object { '   ' + $_ }
            }
            '4' {
                Show-VssStorage
                continue
            }
            default { continue }
        }

        $a = Get-FreeBytes $script:SysRoot
        if ($b -ge 0 -and $a -ge 0) {
            $d = $a - $b
            $t = if ($d -ge 0) { '+' + (Show-Size $d) } else { '−' + (Show-Size ([Math]::Abs($d))) }
            Write-Host ''
            Write-Host ('  Ổ {0}: {1} → {2}   (thực tế thu được {3})' -f (Get-DriveLabel $script:SysRoot), (Show-Size $b), (Show-Size $a), $t) -ForegroundColor Green
        }
    }
}

# ================================================================ vùng bảo vệ + lịch sử
function Show-ProtectedReport {
    Write-Title 'Vùng bảo vệ — chỉ báo cáo, không bao giờ xóa'
    if ([string]::IsNullOrWhiteSpace($script:DataRoot) -or -not (Test-Path -LiteralPath $script:DataRoot)) {
        Write-Host '  Không xác định được thư mục ZaloData.' -ForegroundColor Red; return
    }
    Write-Host ''
    foreach ($n in $script:ProtectedNames) {
        $p = Join-Path $script:DataRoot $n
        if (-not (Test-Path -LiteralPath $p)) { continue }
        $g = @(Get-ChildItem -LiteralPath $p -Recurse -File -Force -ErrorAction SilentlyContinue)
        $b = ($g | Measure-Object Length -Sum).Sum
        if ($null -eq $b) { $b = 0 }
        Write-Host ('   {0,-12} {1,8:N0} tệp · {2}' -f $n, $g.Count, (Show-Size $b)) -ForegroundColor Cyan
    }
    Write-Host ''
    Write-Host '  Database   — cơ sở dữ liệu tin nhắn. Xóa là mất lịch sử chat vĩnh viễn.' -ForegroundColor Red
    Write-Host '  Partitions — dữ liệu phiên đăng nhập. Xóa sẽ phải đăng nhập lại.' -ForegroundColor Red
    Write-Host ''
    Write-Host '  Công cụ chặn cứng hai thư mục trên ở tầng code. Không bộ lọc nào,' -ForegroundColor Green
    Write-Host '  kể cả bộ lọc do bạn tự đặt, chạm được vào chúng.' -ForegroundColor Green
    Write-Host ''
    Write-Host '  Muốn thu gọn vùng này, hãy dùng chức năng quản lý dữ liệu trong chính'
    Write-Host '  ứng dụng Zalo — nơi hiểu cấu trúc cơ sở dữ liệu của nó.'

    Write-Host ''
    Write-Host '  ── Vùng bảo vệ ngoài Zalo' -ForegroundColor Cyan
    Write-Host '  Mức  tất cả  chặn chính nó và mọi thứ bên dưới.' -ForegroundColor DarkGray
    Write-Host '  Mức  gốc     chỉ chặn khi nhắm thẳng vào chính thư mục; con vẫn dọn được.' -ForegroundColor DarkGray
    Write-Host ''
    foreach ($r in ($script:ProtectedRules | Sort-Object { $_.Depth -eq 'any' } -Descending)) {
        $lv = if ($r.Depth -eq 'any') { 'tất cả' } else { 'gốc   ' }
        $co = if ($r.Depth -eq 'any') { 'Cyan' } else { 'DarkGray' }
        Write-Host ('   [{0}]  {1}' -f $lv, $r.Path) -ForegroundColor $co
    }
    Write-Host ''
    Write-Host '  Các mục mức  gốc  là lưới chắn cho catalog.json: một mục ghi nhầm' -ForegroundColor DarkGray
    Write-Host '  "%LOCALAPPDATA%" sẽ bị loại, còn "%LOCALAPPDATA%\npm-cache" vẫn dọn được.' -ForegroundColor DarkGray
}

function Show-History {
    Write-Title 'Lịch sử dọn dẹp'
    $logs = @(Get-ChildItem -LiteralPath $script:LogDir -Filter 'daxoa_*.log' -File -ErrorAction SilentlyContinue | Sort-Object Name)
    if ($logs.Count -eq 0) { Write-Host '  Chưa có lần dọn nào được ghi nhận.' -ForegroundColor DarkGray }
    else {
        $rows = @(); $sumF = 0; $sumB = [long]0
        foreach ($l in $logs) {
            $sum = Get-Content -LiteralPath $l.FullName -Tail 3 -ErrorAction SilentlyContinue |
                   Where-Object { $_ -like '# Tổng kết*' } | Select-Object -First 1
            $kind = Get-Content -LiteralPath $l.FullName -TotalCount 3 -ErrorAction SilentlyContinue |
                    Where-Object { $_ -like '# Chế độ*' } | Select-Object -First 1
            $d = 0; $by = [long]0
            if ($sum -match 'đã xóa=(\d+)') { $d = [int]$Matches[1] }
            # Tệp bị cắt cụt cũng đã trả lại dung lượng, và bytes= đã tính chúng,
            # nên đếm chung để hai cột không mâu thuẫn. Nhật ký cũ không có trường
            # này thì regex đơn giản là không khớp và $d giữ nguyên.
            if ($sum -match 'cắt cụt=(\d+)') { $d += [int]$Matches[1] }
            if ($sum -match 'bytes=(\d+)')  { $by = [long]$Matches[1] }
            $sumF += $d; $sumB += $by
            $rows += [pscustomobject]@{
                Ngày = $l.Name -replace 'daxoa_|\.log', ''
                Chế_độ = if ($kind) { ($kind -replace '# Chế độ\s*:\s*', '').Trim() } else { '?' }
                Tệp = '{0:N0}' -f $d
                Dung_lượng = Show-Size $by
            }
        }
        $rows | Format-Table -AutoSize | Out-String -Width 90 | Write-Host
        Write-Host ('  Tổng cộng: {0:N0} tệp · {1} qua {2} lần dọn' -f $sumF, (Show-Size $sumB), $logs.Count) -ForegroundColor Green
    }

    $all = @(Get-ChildItem -LiteralPath $script:LogDir -File -ErrorAction SilentlyContinue)
    $ab = ($all | Measure-Object Length -Sum).Sum
    if ($null -eq $ab) { $ab = 0 }
    Write-Host ''
    Write-Host ('  Thư mục nhật ký: {0:N0} tệp · {1}' -f $all.Count, (Show-Size $ab)) -ForegroundColor DarkGray
    $r = Read-Line '  Xóa nhật ký cũ hơn bao nhiêu ngày? (Enter để bỏ qua)'
    if ($r -ne '') {
        $n = 0
        if ([int]::TryParse($r, [ref]$n) -and $n -gt 0) {
            $old = @($all | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-$n) })
            if ($old.Count -eq 0) { Write-Host '  Không có nhật ký nào cũ hơn ngưỡng.' -ForegroundColor DarkGray }
            else { $old | Remove-Item -Force -ErrorAction SilentlyContinue
                   Write-Host ('  Đã xóa {0:N0} tệp nhật ký cũ.' -f $old.Count) -ForegroundColor Green }
        } else { Write-Host '  Giá trị không hợp lệ, bỏ qua.' -ForegroundColor Yellow }
    }
}

function Show-TreeSize {
    Write-Title 'Máy đang chiếm bao nhiêu'
    if ($null -ne $script:TreeCache) {
        Write-Host ('  Đang có số liệu đo lúc ' + $script:TreeCache.Stamp) -ForegroundColor DarkGray
        if (Read-YesNo '  Đo lại cho mới? (c/k)') { Reset-TreeCache }
    }
    if ($null -eq $script:TreeCache) { Write-Host '  Đang đo...' -ForegroundColor DarkGray }
    $t = Get-TreeSize
    Write-Host ''
    Write-Host ('  Thư mục Zalo: {0:N0} tệp · {1}' -f $t.Files, (Show-Size $t.Bytes)) -ForegroundColor Green
    Write-Host ''
    $t.Per.GetEnumerator() | Sort-Object { $_.Value.Bytes } -Descending |
        Format-Table @{n='Thư mục'; e={$_.Key}},
                     @{n='Tệp'; e={'{0:N0}' -f $_.Value.Files}},
                     @{n='Dung lượng'; e={Show-Size $_.Value.Bytes}} -AutoSize |
        Out-String -Width 90 | Write-Host
    Show-ScanErrors
    Write-Host ('  Ổ {0} còn trống: {1}' -f (Get-DriveLabel $script:Root), (Show-Size (Get-FreeBytes $script:Root))) -ForegroundColor Cyan
}

# ================================================================ bộ lọc chi tiết
function Set-DateRange {
    Write-Title 'Khoảng thời gian'
    Write-Host '  Lọc theo ngày sửa đổi cuối cùng của tệp.'
    Write-Host ''
    Write-Host '   1  Trước 01/01/2026'
    Write-Host '   2  Cũ hơn 6 tháng'
    Write-Host '   3  Cũ hơn 12 tháng'
    Write-Host '   4  Tự nhập khoảng tùy chọn'
    Write-Host '   Enter để giữ nguyên'
    Write-Host ''
    switch (Read-Line '  Chọn') {
        '1' { $script:FromDate = $null; $script:ToDate = [datetime]'2025-12-31' }
        '2' { $script:FromDate = $null; $script:ToDate = (Get-Date).Date.AddMonths(-6) }
        '3' { $script:FromDate = $null; $script:ToDate = (Get-Date).Date.AddMonths(-12) }
        '4' {
            $f = Read-DateValue '  Từ ngày (dd/MM/yyyy, Enter = từ đầu)'
            $t = Read-DateValue '  Đến ngày (dd/MM/yyyy, Enter = đến nay)'
            if ($null -ne $f -and $null -ne $t -and $f -gt $t) {
                Write-Host '  Từ ngày lớn hơn đến ngày, đã hoán đổi.' -ForegroundColor Yellow
                $tmp = $f; $f = $t; $t = $tmp
            }
            $script:FromDate = $f; $script:ToDate = $t
        }
        default { Write-Host '  Giữ nguyên.' -ForegroundColor DarkGray; return }
    }
    Reset-Scan
    Write-Host ('  Đã đặt: {0}' -f (Show-DateRangeLabel $script:FromDate $script:ToDate)) -ForegroundColor Green
}

function Select-FolderList($title, $current, $isExclude) {
    $all = Get-TopFolders
    if ($all.Count -eq 0) { Write-Host 'Không có thư mục con.' -ForegroundColor Yellow; return $current }
    Write-Title $title
    for ($i = 0; $i -lt $all.Count; $i++) { Write-Host ("   {0,2}  {1}" -f ($i + 1), $all[$i]) }
    Write-Host ''
    Write-Host '   Nhập số cách nhau bởi dấu phẩy, ví dụ: 1,3,5'
    if ($isExclude) { Write-Host '   Gõ  -  để bỏ loại trừ' } else { Write-Host '   Gõ  *  để chọn tất cả' }
    Write-Host '   Enter để giữ nguyên'
    Write-Host ''
    $raw = Read-Line '  Chọn'

    if ($raw -eq '') { Write-Host '  Giữ nguyên.' -ForegroundColor DarkGray; return $current }
    if ((-not $isExclude) -and $raw -eq '*') { Reset-Scan; Write-Host '  Đã đặt: tất cả thư mục.' -ForegroundColor Yellow; return @() }
    if ($isExclude -and $raw -eq '-') { Reset-Scan; Write-Host '  Đã bỏ loại trừ thư mục.' -ForegroundColor Green; return @() }

    $sel = @(); $bad = @()
    foreach ($part in $raw.Split(',')) {
        $p = $part.Trim()
        if ($p -eq '') { continue }
        $n = 0
        if ([int]::TryParse($p, [ref]$n) -and $n -ge 1 -and $n -le $all.Count) { $sel += $all[$n - 1] }
        else { $bad += $p }
    }
    if ($bad.Count -gt 0) {
        Write-Host ''
        Write-Host ('  Giá trị không hợp lệ: ' + ($bad -join ', ')) -ForegroundColor Red
        Write-Host ('  Chỉ nhận số từ 1 đến {0}.' -f $all.Count) -ForegroundColor Red
        Write-Host  '  Bộ lọc giữ nguyên, không đổi gì để tránh xóa nhầm.' -ForegroundColor Yellow
        return $current
    }
    if ($sel.Count -eq 0) { Write-Host '  Không chọn được gì. Giữ nguyên.' -ForegroundColor Yellow; return $current }
    Reset-Scan
    $out = @($sel | Select-Object -Unique)
    Write-Host ('  Đã đặt: ' + ($out -join ', ')) -ForegroundColor Green
    return $out
}

function ConvertTo-ExtList($raw) {
    $list = @()
    foreach ($part in $raw.Split(',')) {
        $e = $part.Trim().ToLower()
        if ($e -eq '') { continue }
        if ($e -ne '(không đuôi)' -and -not $e.StartsWith('.')) { $e = '.' + $e }
        $list += $e
    }
    return @($list | Select-Object -Unique)
}

function Set-ExtFilter {
    Write-Title 'Lọc theo đuôi tệp'
    Write-Host '  Ví dụ: .jxl,.jpg,.mp4'
    Write-Host '  Gõ  (không đuôi)  để bắt tệp không có phần mở rộng.'
    Write-Host '  Gõ  *  để bỏ lọc. Enter để giữ nguyên.'
    Write-Host ''
    $raw = Read-Line '  Đuôi tệp'
    if ($raw -eq '') { Write-Host '  Giữ nguyên.' -ForegroundColor DarkGray; return }
    if ($raw -eq '*') { $script:Exts = @(); Reset-Scan; Write-Host '  Đã đặt: tất cả loại tệp.' -ForegroundColor Green; return }
    $l = ConvertTo-ExtList $raw
    if ($l.Count -eq 0) { Write-Host '  Không nhận diện được gì. Giữ nguyên.' -ForegroundColor Yellow; return }
    $script:Exts = $l; Reset-Scan
    Write-Host ('  Đã đặt: ' + ($l -join ', ')) -ForegroundColor Green
}

function Set-SizeFilter {
    Write-Title 'Kích thước tối thiểu'
    Write-Host '  Chỉ lấy tệp lớn hơn ngưỡng này. Nhập số KB, 0 là không lọc.'
    Write-Host ''
    $raw = Read-Line '  Kích thước tối thiểu (KB)'
    if ($raw -eq '') { Write-Host '  Giữ nguyên.' -ForegroundColor DarkGray; return }
    $n = 0
    if ([int]::TryParse($raw, [ref]$n) -and $n -ge 0) {
        $script:MinSizeKB = $n; Reset-Scan
        Write-Host ("  Đã đặt: từ {0} KB trở lên" -f $n) -ForegroundColor Green
    } else { Write-Host '  Giá trị không hợp lệ, giữ nguyên.' -ForegroundColor Yellow }
}

function Set-ExcludeFilter {
    while ($true) {
        Write-Title 'Loại trừ'
        Write-Host ('  Thư mục loại trừ  : ' + $(if ($script:ExFolders.Count) { $script:ExFolders -join ', ' } else { '(không)' }))
        Write-Host ('  Đuôi tệp loại trừ : ' + $(if ($script:ExExts.Count) { $script:ExExts -join ', ' } else { '(không)' }))
        Write-Host ('  Giữ tệp .rescache : ' + $(if ($script:KeepRescache) { 'Có' } else { 'Không' }))
        Write-Host ''
        Write-Host '   1  Chọn thư mục loại trừ'
        Write-Host '   2  Chọn đuôi tệp loại trừ'
        Write-Host '   3  Bật tắt việc giữ tệp .rescache'
        Write-Host '   Enter để quay lại'
        $c = Read-Line '  Chọn'
        if ($c -eq '') { return }
        switch ($c) {
            '1' { $script:ExFolders = Select-FolderList 'Thư mục loại trừ' $script:ExFolders $true; Read-Line '  Enter để tiếp tục' | Out-Null }
            '2' {
                Write-Host ''
                Write-Host '  Nhập đuôi cần loại trừ, ví dụ: .rescache,.tmp'
                Write-Host '  Gõ  -  để bỏ loại trừ. Enter để giữ nguyên.'
                $raw = Read-Line '  Đuôi loại trừ'
                if ($raw -eq '-') { $script:ExExts = @(); Reset-Scan; Write-Host '  Đã bỏ loại trừ đuôi tệp.' -ForegroundColor Green }
                elseif ($raw -ne '') {
                    $l = ConvertTo-ExtList $raw
                    if ($l.Count -gt 0) { $script:ExExts = $l; Reset-Scan; Write-Host ('  Đã đặt: ' + ($l -join ', ')) -ForegroundColor Green }
                    else { Write-Host '  Không nhận diện được gì. Giữ nguyên.' -ForegroundColor Yellow }
                } else { Write-Host '  Giữ nguyên.' -ForegroundColor DarkGray }
                Read-Line '  Enter để tiếp tục' | Out-Null
            }
            '3' {
                $script:KeepRescache = -not $script:KeepRescache
                Reset-Scan
                Write-Host ('  Giữ tệp .rescache: ' + $(if ($script:KeepRescache) { 'Có' } else { 'Không' })) -ForegroundColor Green
                if (-not $script:KeepRescache) {
                    Write-Host '  Cảnh báo: .rescache là chỉ mục nội bộ của Zalo. Xóa có thể gây lỗi hiển thị.' -ForegroundColor Yellow
                }
                Read-Line '  Enter để tiếp tục' | Out-Null
            }
        }
    }
}

function Set-BackupPolicy {
    Write-Title 'Chính sách sao lưu trước khi xóa'
    Write-Host '  Sao lưu không bắt buộc. Bạn chọn cách công cụ cư xử với dữ liệu thật:'
    Write-Host ''
    Write-Host '   1  Hỏi mỗi lần xóa                (mặc định)'
    Write-Host '   2  Không hỏi, chỉ cần gõ XÓA'
    Write-Host '   3  Bắt buộc sao lưu sạch mới cho xóa'
    Write-Host ''
    Write-Host ('  Hiện tại: ' + (Show-PolicyLabel)) -ForegroundColor Cyan
    Write-Host '  Enter để giữ nguyên.'
    Write-Host ''
    switch (Read-Line '  Chọn') {
        '1' { $script:BackupPolicy = 'HOI' }
        '2' { $script:BackupPolicy = 'KHONG' }
        '3' { $script:BackupPolicy = 'BATBUOC' }
        default { Write-Host '  Giữ nguyên.' -ForegroundColor DarkGray; return }
    }
    Save-Settings
    Write-Host ('  Đã đặt: ' + (Show-PolicyLabel)) -ForegroundColor Green
    Write-Host  '  Lựa chọn này được ghi nhớ cho các lần chạy sau.' -ForegroundColor DarkGray
}

# ================================================================ hồ sơ bộ lọc
function Get-CurrentProfile {
    return [pscustomobject]@{
        FromDate  = if ($null -eq $script:FromDate) { $null } else { $script:FromDate.ToString('yyyy-MM-dd') }
        ToDate    = if ($null -eq $script:ToDate)   { $null } else { $script:ToDate.ToString('yyyy-MM-dd') }
        Folders = $script:Folders; Exts = $script:Exts
        ExFolders = $script:ExFolders; ExExts = $script:ExExts
        MinSizeKB = $script:MinSizeKB; KeepRescache = $script:KeepRescache
    }
}

function Read-Profiles {
    if (-not (Test-Path -LiteralPath $script:ProfileFile)) { return @{} }
    try {
        $o = Get-Content -LiteralPath $script:ProfileFile -Raw | ConvertFrom-Json
        $h = @{}
        $o.PSObject.Properties | ForEach-Object { $h[$_.Name] = $_.Value }
        return $h
    } catch { return @{} }
}

function Show-ProfileMenu {
    Write-Title 'Hồ sơ bộ lọc'
    Write-Host '  Hồ sơ chỉ lưu lại các bộ lọc. Nạp hồ sơ không tự quét và không tự xóa.' -ForegroundColor Green
    Write-Host ''
    $profs = Read-Profiles
    $names = @($profs.Keys | Sort-Object)
    if ($names.Count -eq 0) { Write-Host '  Chưa có hồ sơ nào.' -ForegroundColor DarkGray }
    else {
        for ($i = 0; $i -lt $names.Count; $i++) {
            $p = $profs[$names[$i]]
            Write-Host ("   {0}  {1}" -f ($i + 1), $names[$i]) -ForegroundColor Cyan
            $fd = if ($p.FromDate) { $p.FromDate } else { 'không giới hạn' }
            $td = if ($p.ToDate)   { $p.ToDate }   else { 'không giới hạn' }
            Write-Host ("      {0} → {1}" -f $fd, $td) -ForegroundColor DarkGray
        }
    }
    Write-Host ''
    Write-Host '   L <số>   Nạp hồ sơ        S <tên>  Lưu bộ lọc hiện tại'
    Write-Host '   D <số>   Xóa hồ sơ        Enter    Quay lại'
    $cmd = Read-Line '  Lệnh'
    if ($cmd -eq '') { return }

    $parts = $cmd.Split(' ', 2)
    $op = $parts[0].ToUpper()
    $arg = if ($parts.Count -gt 1) { $parts[1].Trim() } else { '' }

    if ($op -eq 'S') {
        if ($arg -eq '') { Write-Host '  Thiếu tên hồ sơ.' -ForegroundColor Yellow; return }
        $profs[$arg] = Get-CurrentProfile
        ($profs | ConvertTo-Json -Depth 6) | Set-Content -LiteralPath $script:ProfileFile -Encoding UTF8
        Write-Host ('  Đã lưu hồ sơ: ' + $arg) -ForegroundColor Green
    }
    elseif ($op -eq 'L' -or $op -eq 'D') {
        $n = 0
        if (-not ([int]::TryParse($arg, [ref]$n)) -or $n -lt 1 -or $n -gt $names.Count) {
            Write-Host '  Số hồ sơ không hợp lệ. Không đổi gì.' -ForegroundColor Red; return
        }
        $name = $names[$n - 1]
        if ($op -eq 'D') {
            $profs.Remove($name)
            ($profs | ConvertTo-Json -Depth 6) | Set-Content -LiteralPath $script:ProfileFile -Encoding UTF8
            Write-Host ('  Đã xóa hồ sơ: ' + $name) -ForegroundColor Green
        } else {
            $p = $profs[$name]
            $script:FromDate = if ($p.FromDate) { [datetime]::ParseExact($p.FromDate, 'yyyy-MM-dd', $null) } else { $null }
            $script:ToDate   = if ($p.ToDate)   { [datetime]::ParseExact($p.ToDate,   'yyyy-MM-dd', $null) } else { $null }
            $script:Folders = @($p.Folders); $script:Exts = @($p.Exts)
            $script:ExFolders = @($p.ExFolders); $script:ExExts = @($p.ExExts)
            $script:MinSizeKB = [int]$p.MinSizeKB; $script:KeepRescache = [bool]$p.KeepRescache
            Reset-Scan
            Write-Host ('  Đã nạp hồ sơ: ' + $name) -ForegroundColor Green
            Write-Host  '  Bộ lọc đã được điền sẵn. Chưa quét và chưa xóa gì.' -ForegroundColor DarkGray
        }
    }
    else { Write-Host '  Lệnh không hiểu.' -ForegroundColor Yellow }
}

# ================================================================ TRỢ LÝ DẪN DẮT
# Đo một lượt duy nhất rồi tính dung lượng cho từng mốc thời gian,
# thay vì quét lại ba lần.
function Get-AgeBuckets {
    $now = (Get-Date).Date
    $c12 = $now.AddMonths(-12); $c6 = $now.AddMonths(-6); $c2026 = [datetime]'2026-01-01'
    $b = @{ y1 = [long]0; m6 = [long]0; pre2026 = [long]0; total = [long]0
            n1 = 0; n6 = 0; nPre = 0; nTotal = 0 }
    $script:LastScanErrors = 0
    Get-FilesSafe $script:Root | ForEach-Object {
        if (Test-Protected $_.FullName) { return }
        if ($script:KeepRescache -and $_.Extension -eq '.rescache') { return }
        $t = $_.LastWriteTime; $len = $_.Length
        $b.total += $len; $b.nTotal++
        if ($t -lt $c12)   { $b.y1 += $len; $b.n1++ }
        if ($t -lt $c6)    { $b.m6 += $len; $b.n6++ }
        if ($t -lt $c2026) { $b.pre2026 += $len; $b.nPre++ }
    }
    return $b
}

# Trả về $true nếu có kết quả cần người dùng đọc, để bên gọi biết có nên dừng lại không.
function Invoke-WizardZaloOld {
    Clear-Host
    Write-Title 'Dọn dữ liệu Zalo cũ'
    Write-Host '  Đang đo dung lượng theo từng mốc thời gian...' -ForegroundColor DarkGray
    $b = Get-AgeBuckets

    # Chỉ hiện những mốc thật sự có dữ liệu. Mốc 0 byte là lựa chọn vô nghĩa.
    $opts = @()
    if ($b.n1 -gt 0)   { $opts += @{ K = '1'; T = 'Cũ hơn 12 tháng'; B = $b.y1;      N = $b.n1 } }
    if ($b.n6 -gt 0)   { $opts += @{ K = '2'; T = 'Cũ hơn 6 tháng';  B = $b.m6;      N = $b.n6 } }
    if ($b.nPre -gt 0) { $opts += @{ K = '3'; T = 'Trước năm 2026';  B = $b.pre2026; N = $b.nPre } }

    Clear-Host
    Write-Title 'Dọn dữ liệu Zalo cũ'
    Write-Host ''
    if ($opts.Count -eq 0) {
        Write-Host '  Không còn dữ liệu cũ ở các mốc thường dùng.' -ForegroundColor Green
        Write-Host ('  Toàn bộ thư mục Zalo hiện là {0}.' -f (Show-Size $b.total)) -ForegroundColor DarkGray
    } else {
        Write-Host '  Dọn dữ liệu cũ tới mốc nào?'
        Write-Host ''
        foreach ($o in $opts) {
            Write-Host ('   {0}  {1,-16} →  {2,10}   ({3:N0} tệp)' -f $o.K, $o.T, (Show-Size $o.B), $o.N)
        }
        # Số thứ tự giữ nguyên để quen tay, nên phải nói rõ vì sao có chỗ trống
        if ($opts.Count -lt 3) {
            Write-Host ''
            Write-Host '   Các mốc còn lại không hiện vì không còn dữ liệu nào.' -ForegroundColor DarkGray
        }
    }
    Write-Host ''
    Write-Host '   4  Tôi tự nhập ngày'
    Write-Host '   0  Quay lại'
    Write-Host ''
    Show-ScanErrors
    $c = Read-Line '  Chọn'
    if ($c -eq '' -or $c -eq '0') { return $false }

    if ($c -in @('1', '2', '3') -and @($opts | Where-Object { $_.K -eq $c }).Count -eq 0) {
        Write-Host '  Mốc đó không có dữ liệu nào.' -ForegroundColor Yellow
        return $true
    }

    switch ($c) {
        '1' { $script:FromDate = $null; $script:ToDate = (Get-Date).Date.AddMonths(-12) }
        '2' { $script:FromDate = $null; $script:ToDate = (Get-Date).Date.AddMonths(-6) }
        '3' { $script:FromDate = $null; $script:ToDate = [datetime]'2025-12-31' }
        '4' {
            $f = Read-DateValue '  Từ ngày (dd/MM/yyyy, Enter = từ đầu)'
            $t = Read-DateValue '  Đến ngày (dd/MM/yyyy, Enter = đến nay)'
            if ($null -ne $f -and $null -ne $t -and $f -gt $t) { $tmp = $f; $f = $t; $t = $tmp }
            $script:FromDate = $f; $script:ToDate = $t
        }
        default { Write-Host '  Không hiểu lựa chọn.' -ForegroundColor Yellow; return $true }
    }
    Reset-Scan
    Invoke-Scan
    if ($null -eq $script:Scan -or $script:Scan.Count -eq 0) {
        Write-Host ''
        Write-Host '  Không có tệp nào trong khoảng này.' -ForegroundColor Green
        return $true
    }
    Write-Host ''
    if (Read-YesNo '  Xem chi tiết trước khi quyết định? (c/k)') { Show-ScanDetail }
    Invoke-Delete
    return $true
}

function Invoke-WizardReclaim {
    $notice = ''
    while ($true) {
        Clear-Host
        Write-Title 'Lấy lại dung lượng ổ đĩa'
        if ($script:HasZalo) {
            Write-Host '   1  Dữ liệu Zalo cũ theo thời gian'
            Write-Host '   2  Bản trùng lặp trong Zalo        — an toàn nhất, không mất ảnh' -ForegroundColor Green
            Write-Host '   3  Cache của ứng dụng Zalo'
        } else {
            Write-Host '   Máy này chưa cài Zalo nên ba mục về Zalo đã được ẩn.' -ForegroundColor DarkYellow
        }
        Write-Host '   4  Cache hệ thống ngoài Zalo'
        Write-Host '   0  Quay lại'
        if ($notice -ne '') { Write-Host ''; Write-Host ('   ' + $notice) -ForegroundColor Yellow; $notice = '' }
        Write-Host ''
        $c = Read-Line '  Chọn'
        if ($c -eq '' -or $c -eq '0') { return }

        if ($c -in @('1', '2', '3') -and -not $script:HasZalo) {
            $notice = 'Mục đó cần Zalo, mà máy này chưa cài.'
            continue
        }

        # Mỗi nhánh trả về $true nếu có kết quả cần đọc. Không có thì quay lại
        # menu ngay, tránh lời nhắc thừa nuốt mất phím lệnh kế tiếp.
        $coKetQua = $false
        switch ($c) {
            '1' { $coKetQua = Invoke-WizardZaloOld }
            '2' {
                Invoke-DedupScan
                if ($null -ne $script:Scan -and $script:Scan.Count -gt 0) {
                    Write-Host ''
                    if (Read-YesNo '  Xem chi tiết vài cặp trùng? (c/k)') { Show-ScanDetail }
                    Invoke-Delete
                }
                $coKetQua = $true
            }
            '3' {
                Invoke-AppCacheScan
                if ($null -ne $script:Scan -and $script:Scan.Count -gt 0) { Invoke-Delete }
                $coKetQua = $true
            }
            '4' {
                Invoke-CatalogScan
                if ($null -ne $script:Scan -and $script:Scan.Count -gt 0) { Invoke-Delete; $coKetQua = $true }
            }
            default { $notice = ('Không có mục "' + $c + '". Chọn 1 đến 4, hoặc 0 để quay lại.'); continue }
        }
        if ($coKetQua) { Read-Line '  Enter để quay lại menu' | Out-Null }
    }
}

function Show-AdvancedMenu {
    while ($true) {
        Clear-Host
        Write-Title 'Tùy chọn nâng cao'
        Write-Host ('  Bộ lọc: {0} · thư mục {1} · đuôi {2} · từ {3} KB' -f `
                    (Show-DateRangeLabel $script:FromDate $script:ToDate),
                    $(if ($script:Folders.Count) { $script:Folders -join ',' } else { 'tất cả' }),
                    $(if ($script:Exts.Count) { $script:Exts -join ',' } else { 'tất cả' }),
                    $script:MinSizeKB) -ForegroundColor DarkGray
        if ($null -ne $script:Scan) {
            $t = ($script:Scan | Measure-Object Size -Sum).Sum
            if ($null -eq $t) { $t = 0 }
            Write-Host ('  Kết quả quét đang giữ: {0} · {1:N0} tệp · {2}' -f $script:ScanKind, $script:Scan.Count, (Show-Size $t)) -ForegroundColor Yellow
        }
        Write-Host ''
        Write-Host '   1  Khoảng thời gian        2  Thư mục con'
        Write-Host '   3  Đuôi tệp                4  Kích thước tối thiểu'
        Write-Host '   5  Loại trừ                6  Hồ sơ bộ lọc'
        Write-Host ''
        Write-Host '   7  Quét theo bộ lọc        8  Chi tiết + xuất CSV'
        # Nhãn phải nói rõ là xóa TỆP TRÊN ĐĨA. Nhãn cũ "Xóa kết quả quét đang
        # giữ" đọc tự nhiên trong tiếng Việt là "bỏ kết quả quét đi" — một việc
        # vô hại — trong khi phím này gọi Invoke-Delete và xóa vĩnh viễn.
        Write-Host '   9  Sao lưu và xác minh     X  Xóa hẳn tệp trong kết quả quét'
        Write-Host ''
        Write-Host '   K  Khôi phục từ bản sao lưu'
        Write-Host '   V  Shadow Copy             B  Vùng bảo vệ'
        Write-Host '   L  Lịch sử dọn dẹp         C  Chính sách sao lưu'
        Write-Host '   T  Đổi tài khoản Zalo'
        Write-Host '   0  Quay lại'
        Write-Host ''
        $c = (Read-Line '  Chọn').ToUpper()
        if ($c -eq '' -or $c -eq '0') { return }
        switch ($c) {
            '1' { Set-DateRange;   Read-Line '  Enter để tiếp tục' | Out-Null }
            '2' { $script:Folders = Select-FolderList 'Thư mục con' $script:Folders $false; Read-Line '  Enter để tiếp tục' | Out-Null }
            '3' { Set-ExtFilter;   Read-Line '  Enter để tiếp tục' | Out-Null }
            '4' { Set-SizeFilter;  Read-Line '  Enter để tiếp tục' | Out-Null }
            '5' { Set-ExcludeFilter }
            '6' { Show-ProfileMenu; Read-Line '  Enter để tiếp tục' | Out-Null }
            '7' { Invoke-Scan;     Read-Line '  Enter để tiếp tục' | Out-Null }
            '8' { Show-ScanDetail; Read-Line '  Enter để tiếp tục' | Out-Null }
            '9' { Invoke-Backup;   Read-Line '  Enter để tiếp tục' | Out-Null }
            'X' { Invoke-Delete;   Read-Line '  Enter để tiếp tục' | Out-Null }
            'K' { Invoke-Restore;  Read-Line '  Enter để tiếp tục' | Out-Null }
            'V' { Show-ShadowReport; Read-Line '  Enter để tiếp tục' | Out-Null }
            'B' { Show-ProtectedReport; Read-Line '  Enter để tiếp tục' | Out-Null }
            'L' { Show-History;    Read-Line '  Enter để tiếp tục' | Out-Null }
            'C' { Set-BackupPolicy; Read-Line '  Enter để tiếp tục' | Out-Null }
            'T' { Select-Root; Reset-Scan }
            default { }
        }
    }
}

function Show-Home {
    Clear-Host
    Write-Host ''
    Write-Host '  ╔══════════════════════════════════════════════════════════╗' -ForegroundColor DarkCyan
    Write-Host '  ║   DỌN DẸP ZALO — chỉ chạy khi bạn mở công cụ             ║' -ForegroundColor Cyan
    Write-Host '  ╚══════════════════════════════════════════════════════════╝' -ForegroundColor DarkCyan
    Write-Host ''
    $free = Get-FreeBytes $script:SysRoot
    if ($free -ge 0) {
        Write-Host ('   Ổ {0} còn trống    {1}' -f (Get-DriveLabel $script:SysRoot), (Show-Size $free)) -ForegroundColor Cyan
    }
    if ($script:HasZalo) {
        # Đo lần đầu để màn hình chính có thông tin. Nếu lần đo trước quá chậm
        # thì thôi, để người dùng tự bấm 2 khi cần, tránh mỗi lần về đây lại chờ.
        if ($null -eq $script:TreeCache -and ($null -eq $script:TreeScanSec -or $script:TreeScanSec -lt 5)) {
            Write-Host '   Đang đo thư mục Zalo...' -ForegroundColor DarkGray
            Get-TreeSize | Out-Null
        }
        if ($null -ne $script:TreeCache) {
            Write-Host ('   Thư mục Zalo      ' + (Show-Size $script:TreeCache.Bytes)) -ForegroundColor Cyan
        } else {
            Write-Host '   Thư mục Zalo      chưa đo — bấm 2 để xem' -ForegroundColor DarkGray
        }
    }
    if (-not $script:HasZalo) {
        Write-Host '   Máy này chưa cài Zalo — vẫn dọn được cache hệ thống.' -ForegroundColor DarkYellow
    }
    Write-Host ''
    Write-Host '   Bạn muốn làm gì?'
    Write-Host ''
    Write-Host '    1   Lấy lại dung lượng ổ đĩa'
    if ($script:HasZalo) { Write-Host '    2   Xem máy đang chiếm bao nhiêu' }
    Write-Host '    3   Khôi phục dữ liệu đã sao lưu'
    Write-Host ''
    Write-Host '    9   Tùy chọn nâng cao' -ForegroundColor DarkGray
    Write-Host '    0   Thoát' -ForegroundColor DarkGray
    Write-Host ''
}

# ================================================================ main
Initialize-Environment
Read-Settings
Initialize-ProtectedAbs
if ($script:Root -eq '' -or -not (Test-Path -LiteralPath $script:Root)) { Select-Root -Quiet }
$script:HasZalo = ($script:Root -ne '' -and (Test-Path -LiteralPath $script:Root))
if ($script:HasZalo -and $script:DataRoot -eq '') { $script:DataRoot = Resolve-DataRoot $script:Root }

while ($true) {
    Show-Home
    switch (Read-Line '   Chọn') {
        '1' { Invoke-WizardReclaim }
        '2' { if ($script:HasZalo) { Show-TreeSize; Read-Line '  Enter để tiếp tục' | Out-Null } }
        '3' { Invoke-Restore; Read-Line '  Enter để tiếp tục' | Out-Null }
        '9' { Show-AdvancedMenu }
        '0' {
            Write-Host ''
            Write-Host '   Thoát. Không có tác vụ nền nào được đặt lại.' -ForegroundColor DarkGray
            Restore-Environment
            exit 0
        }
        default { }
    }
}
