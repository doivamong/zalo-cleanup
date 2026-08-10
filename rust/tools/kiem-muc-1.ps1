#Requires -Version 5.1
<#
    Kiểm CHÍN mục tiếp cận **mức 1** mà cổng M5 xếp là "cần người thật".

    ------------------------------------------------------------------
    VÌ SAO BỘ CHẠY NÀY TỒN TẠI

    `cong-m5.ps1` liệt kê chín mục mức 1 rồi dừng ở đó, lý do "không cỗ máy nào
    kết luận được". Lý do ấy **sai một nửa**, và đo mới biết:

    egui bật `accesskit`, mà AccessKit phơi toàn bộ cây widget ra UI Automation
    của Windows — kèm **tên, vai trò, trạng thái bật/tắt và khung bao**. Tức là
    hỏi được bằng máy đúng những câu trước đây tưởng phải nhìn bằng mắt:

      · tiêu điểm đang ở phần tử nào      → BP-01, BP-04
      · nút Xóa đã bật chưa               → §8.1-3
      · phần tử nào tràn khỏi vùng vẽ     → DPI-04
      · hai nút cách nhau bao nhiêu dip   → MAU-09

    Phần **thật sự** cần người thì vẫn cần người. Bộ chạy này nói thẳng ra ở
    cuối chứ không gộp vào phần "đạt".

    ------------------------------------------------------------------
    HAI CÁI BẪY ĐÃ SẬP TRONG LÚC DỰNG BỘ CHẠY NÀY, ghi lại để khỏi sập lần nữa

    ① `SetForegroundWindow` **thất bại lặng lẽ**, và Tab đi vào thanh tác vụ.
       Bản đầu báo cáo rất trôi chảy thứ tự tiêu điểm… của thanh tác vụ Windows.
       Nên `Chac-Truoc` bây giờ **chứng minh** cửa sổ đã lên trước rồi mới gửi
       phím, và ném lỗi nếu không. Công thức đúng là gắn luồng vào cửa sổ ĐANG
       giữ tiền cảnh, không phải luồng của cửa sổ đích.

    ② Không có hàm nào ở đây đụng tới chuột, và đó là **một phần của phép thử**:
       BP-01 đòi "rút chuột ra". Thiếu `SetCursorPos` là cố ý.
    ------------------------------------------------------------------

    CHỈ CHẠY TRÊN HỘP CÁT trong %TEMP%. Tự dựng dữ liệu giả, tự dọn.
    Không bao giờ trỏ vào thư mục Zalo thật — chốt chặn ở `Moi-Hop-Cat`.
#>
param(
    [string[]]$Chi = @('tat-ca'),
    [string]$ThuMucAnh = (Join-Path $env:TEMP 'kiem-muc-1'),
    # Bật trình đọc màn hình Narrator thật. Mặc định TẮT vì nó nói thành tiếng
    # và có thể hiện hộp thoại chào mừng.
    [switch]$KemNarrator
)

$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

$goc = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$exe = Join-Path $goc 'rust\target\release\zalo-gui.exe'
if (-not (Test-Path $exe)) { throw "Chưa dựng $exe — chạy: cargo build --release -p zalo-gui" }

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Drawing, System.Windows.Forms

Add-Type -Namespace K1 -Name W -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
[DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr h);
[DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
[DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr p);
[DllImport("user32.dll")] public static extern IntPtr SetActiveWindow(IntPtr h);
[DllImport("user32.dll")] public static extern IntPtr SetFocus(IntPtr h);
[DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
[DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
[DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
[DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
[DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h,int x,int y,int w,int t,bool re);
[DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p, IntPtr l);
[DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
[DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
[DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
[DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, System.Text.StringBuilder s, int n);
[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
[DllImport("user32.dll")] public static extern IntPtr GetWindowDpiAwarenessContext(IntPtr h);
[DllImport("user32.dll")] public static extern int GetAwarenessFromDpiAwarenessContext(IntPtr c);
[DllImport("user32.dll")] public static extern bool AreDpiAwarenessContextsEqual(IntPtr a, IntPtr b);
[DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
[DllImport("user32.dll", EntryPoint="SystemParametersInfoW", SetLastError=true)]
  public static extern bool SpiGet(uint a, uint b, ref bool v, uint f);
[DllImport("user32.dll", EntryPoint="SystemParametersInfoW", SetLastError=true)]
  public static extern bool SpiSet(uint a, uint b, IntPtr v, uint f);
[DllImport("user32.dll", EntryPoint="SystemParametersInfoW", SetLastError=true)]
  public static extern bool SpiGetU(uint a, uint b, ref uint v, uint f);
[DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x,int y,int w,int t, uint f);
[DllImport("user32.dll", SetLastError=true)] public static extern uint SendInput(uint n, INPUT[] p, int cb);
[DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
public delegate bool EnumProc(IntPtr h, IntPtr l);
public struct POINT { public int X,Y; }
public struct RECT { public int L,T,R,B; }
[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Explicit, Size=40)]
public struct INPUT { [System.Runtime.InteropServices.FieldOffset(0)] public uint type;
                      [System.Runtime.InteropServices.FieldOffset(8)] public KEYBDINPUT ki; }
'@

# ============================================== gỡ khoá tiền cảnh của Windows
#
# Windows chặn `SetForegroundWindow` từ tiến trình không đang ở tiền cảnh. Bộ
# chạy này khởi động từ một cửa sổ khác, nên nó bị chặn đúng như thiết kế —
# và một lượt chạy đã chết giữa chừng vì thế.
#
# Đặt ngưỡng khoá về 0 **cho phiên đang chạy**, không ghi vào ini, và trả lại
# giá trị cũ khi thoát. Đây là cùng cách mọi bộ chạy giao diện tự động làm.
$SPIF_KHONG_GHI = 0
$SPI_GET_KHOA_TIEN_CANH = 0x2000
$SPI_SET_KHOA_TIEN_CANH = 0x2001
$script:KhoaCu = [uint32]0
if ([K1.W]::SpiGetU($SPI_GET_KHOA_TIEN_CANH, 0, [ref]$script:KhoaCu, 0)) {
    [void][K1.W]::SpiSet($SPI_SET_KHOA_TIEN_CANH, 0, [IntPtr]::Zero, $SPIF_KHONG_GHI)
    $cu = $script:KhoaCu
    [void](Register-EngineEvent PowerShell.Exiting -Action {
        [void][K1.W]::SpiSet(0x2001, 0, [IntPtr][int]$cu, 0)
    })
}

# ================================================================== báo cáo
$script:Pass = 0; $script:Fail = 0
$script:ConLai = New-Object Collections.ArrayList
function Assert($ma, $ten, $ok, $ct) {
    if ($ok) { $script:Pass++; Write-Host ("  [ĐẠT ] {0,-9} {1}" -f $ma, $ten) -ForegroundColor Green }
    else {
        $script:Fail++
        Write-Host ("  [HỎNG] {0,-9} {1}" -f $ma, $ten) -ForegroundColor Red
        if ($ct) { Write-Host "             $ct" -ForegroundColor Red }
    }
}
function Ghi($t) { Write-Host "             $t" -ForegroundColor DarkGray }
$script:MucDangChay = '?'
function Muc($t) {
    $script:MucDangChay = ($t -split ' ')[0]
    Write-Host ''; Write-Host "── $t" -ForegroundColor Yellow
}
function ConNguoi($ma, $viec) { [void]$script:ConLai.Add(@{ Ma = $ma; Viec = $viec }) }
function Lam($ma) { return ($Chi -contains 'tat-ca' -or $Chi -contains $ma) }

# ============================================================== gửi phím
$script:H = [IntPtr]::Zero
function Chac-Truoc {
    for ($i = 0; $i -lt 40; $i++) {
        # Cửa sổ đã thu nhỏ thì phải BUNG RA trước. `SW_SHOW` không bung — nó
        # hiện cửa sổ ở đúng trạng thái đang có, nên một lần thu nhỏ ngoài ý
        # muốn là mọi phím sau đó rơi vào hư không mà bộ chạy vẫn báo "đã lên
        # trước". Đã gặp thật một lần.
        if ([K1.W]::IsIconic($script:H)) {
            [void][K1.W]::ShowWindow($script:H, 9)      # SW_RESTORE
            Start-Sleep -Milliseconds 400
        }
        if ([K1.W]::GetForegroundWindow() -eq $script:H) { return }
        $me = [K1.W]::GetCurrentThreadId()
        $tc = [K1.W]::GetWindowThreadProcessId([K1.W]::GetForegroundWindow(), [IntPtr]::Zero)
        $dc = [K1.W]::GetWindowThreadProcessId($script:H, [IntPtr]::Zero)
        [void][K1.W]::AttachThreadInput($me, $tc, $true)
        [void][K1.W]::AttachThreadInput($me, $dc, $true)
        [void][K1.W]::ShowWindow($script:H, 5)
        [void][K1.W]::BringWindowToTop($script:H)
        [void][K1.W]::SetForegroundWindow($script:H)
        [void][K1.W]::SetActiveWindow($script:H)
        # KHÔNG gọi `SetFocus`. Đo lặp, cùng kịch bản, 5 lượt mỗi nhánh:
        #
        #   có SetFocus  → ứng dụng tự thoát 1/5 lượt
        #   không có     → 0/5
        #
        # Thoát SẠCH — không có bản ghi sự cố nào trong nhật ký Ứng dụng — nên
        # là vòng lặp sự kiện kết thúc chứ không phải sập. Chưa truy tới tận
        # cùng được; đã ghi vào docs/viec-con-lai.md. Ở đây chỉ cần biết
        # `SetFocus` là thừa: hai lời gọi trên đã đủ đưa cửa sổ lên tiền cảnh.
        [void][K1.W]::AttachThreadInput($me, $dc, $false)
        [void][K1.W]::AttachThreadInput($me, $tc, $false)
        Start-Sleep -Milliseconds 250
    }
    # Nói RA cửa sổ nào đang giành, thay vì chỉ báo thất bại. Lần trước là
    # `XamlExplorerHostIslandWindow` của Explorer, và nếu không in ra thì mất
    # thêm một lượt chạy nữa mới biết.
    $fg = [K1.W]::GetForegroundWindow()
    $s = New-Object Text.StringBuilder 300
    [void][K1.W]::GetWindowTextW($fg, $s, 300)
    $sc = New-Object Text.StringBuilder 300
    [void][K1.W]::GetClassNameW($fg, $sc, 300)
    throw ("Không đưa được cửa sổ lên trước — dừng, không gửi phím đi lạc. " +
           "Đang giữ tiền cảnh: lớp='$($sc.ToString())' tên='$($s.ToString())'")
}
function New-Phim($vk, $len) {
    $i = New-Object K1.W+INPUT; $i.type = 1
    $k = New-Object K1.W+KEYBDINPUT; $k.wVk = [uint16]$vk; $k.dwFlags = $(if ($len) { 2 } else { 0 })
    $i.ki = $k; return $i
}
function New-Chu([char]$c, $len) {
    $i = New-Object K1.W+INPUT; $i.type = 1
    $k = New-Object K1.W+KEYBDINPUT; $k.wVk = 0; $k.wScan = [uint16]$c
    $k.dwFlags = $(if ($len) { 6 } else { 4 })       # KEYEVENTF_UNICODE | KEYUP
    $i.ki = $k; return $i
}
function Gui($m) { [void][K1.W]::SendInput([uint32]$m.Count, [K1.W+INPUT[]]$m, 40) }
$VK = @{ TAB = 0x09; ENTER = 0x0D; ESC = 0x1B; SPACE = 0x20; BS = 0x08; CTRL = 0x11
         V = 0x56; A = 0x41; END = 0x23 }

# Kiểm tiền cảnh **sau khi gửi**, không chỉ trước.
#
# Kiểm-rồi-gửi để hở một khe: cửa sổ khác giành tiền cảnh đúng giữa hai bước
# thì phím rơi sang đó. Đã xảy ra thật một lần — Chrome cướp mất giữa chừng.
# Không cứu được lô phím vừa gửi, nhưng chặn được lô thứ hai, và phần BP-01
# gõ `Ctrl+A` rồi `Backspace` thì một lô đã là quá đủ để hỏng việc người khác.
function Sau-Khi-Gui {
    if ([K1.W]::GetForegroundWindow() -ne $script:H) {
        $s = New-Object Text.StringBuilder 300
        [void][K1.W]::GetWindowTextW([K1.W]::GetForegroundWindow(), $s, 300)
        throw ("Tiền cảnh bị giành GIỮA lúc gửi phím — dừng ngay. Cửa sổ đang giữ: '" +
               $s.ToString() + "'. Phím vừa gửi có thể đã rơi sang đó.")
    }
}
function Phim($vk, $lan = 1, $nghi = 110) {
    Chac-Truoc
    for ($i = 0; $i -lt $lan; $i++) { Gui @((New-Phim $vk $false), (New-Phim $vk $true)); Start-Sleep -Milliseconds $nghi }
    Sau-Khi-Gui
}
function PhimVoi($mod, $vk) {
    Chac-Truoc
    Gui @((New-Phim $mod $false), (New-Phim $vk $false), (New-Phim $vk $true), (New-Phim $mod $true))
    Start-Sleep -Milliseconds 150
    Sau-Khi-Gui
}
# Gửi đúng chuỗi ký tự Unicode mà bộ gõ tiếng Việt gửi ở bước cuối.
function GoChu([string]$s, $nghi = 70) {
    Chac-Truoc
    foreach ($c in $s.ToCharArray()) {
        Gui @((New-Chu $c $false), (New-Chu $c $true))
        Start-Sleep -Milliseconds $nghi
        Sau-Khi-Gui
    }
}

# ==================================================================== UIA
# UIA thỉnh thoảng ném `ElementNotAvailable` khi cây đang được dựng lại giữa
# hai khung vẽ. Đó là nhiễu của phép đo, không phải lỗi của công cụ — thử lại
# vài nhịp rồi mới chịu thua.
function Cay {
    $r = $null
    $all = $null
    for ($lan = 0; $lan -lt 8; $lan++) {
        try {
            $r = [System.Windows.Automation.AutomationElement]::FromHandle($script:H)
            $all = $r.FindAll([System.Windows.Automation.TreeScope]::Descendants,
                [System.Windows.Automation.Condition]::TrueCondition)
            break
        } catch { Start-Sleep -Milliseconds 250; $all = $null }
    }
    $ds = New-Object Collections.ArrayList
    if ($null -eq $all) { return $ds }
    foreach ($e in $all) {
        try {
            $b = $e.Current.BoundingRectangle
            [void]$ds.Add([pscustomobject]@{
                Loai = $e.Current.ControlType.ProgrammaticName.Replace('ControlType.', '')
                Ten  = $e.Current.Name
                Bat  = $e.Current.IsEnabled
                X = [int]$b.X; Y = [int]$b.Y; W = [int]$b.Width; H = [int]$b.Height
            })
        } catch { }
    }
    return $ds
}
function TieuDiem {
    try {
        $f = [System.Windows.Automation.AutomationElement]::FocusedElement
        return [pscustomobject]@{
            Loai = $f.Current.ControlType.ProgrammaticName.Replace('ControlType.', '')
            Ten  = $f.Current.Name
        }
    } catch { return [pscustomobject]@{ Loai = '?'; Ten = '(không đọc được)' } }
}
function TenTieuDiem { $t = TieuDiem; return $t.Ten }

# Nội dung ô nhập, đọc qua `ValuePattern` — để biết có gì cần xóa trước khi gõ,
# thay vì bổ một nhát `Ctrl+A` vào bất cứ cửa sổ nào đang nhận phím.
function Doc-O-Nhap {
    # Thử ba đường và NÓI RA đường nào ăn. Không phải mọi nhà cung cấp UIA đều
    # phơi `ValuePattern`; im lặng trả chuỗi rỗng khi đọc hỏng là cách phép thử
    # báo "chưa gõ được" trong khi tay vẫn gõ xong — đã dính đúng bẫy ấy một lần.
    $e = $null
    try {
        $r = [System.Windows.Automation.AutomationElement]::FromHandle($script:H)
        $e = $r.FindFirst([System.Windows.Automation.TreeScope]::Descendants,
            (New-Object System.Windows.Automation.PropertyCondition(
                [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
                [System.Windows.Automation.ControlType]::Edit)))
    } catch { }
    if ($null -eq $e) { $script:CachDoc = 'không thấy ô nhập'; return '' }
    try {
        $vp = $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
        $script:CachDoc = 'ValuePattern'
        return [string]$vp.Current.Value
    } catch { }
    try {
        $v = $e.GetCurrentPropertyValue([System.Windows.Automation.ValuePattern]::ValueProperty)
        if ($null -ne $v) { $script:CachDoc = 'ValueProperty'; return [string]$v }
    } catch { }
    try {
        $t = $e.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
        $script:CachDoc = 'TextPattern'
        return [string]$t.DocumentRange.GetText(-1)
    } catch { }
    $script:CachDoc = 'không đọc được bằng đường nào'
    return ''
}
$script:CachDoc = '?'
function Tim($cay, $mau) { return ($cay | Where-Object { $_.Ten -like $mau } | Select-Object -First 1) }
function CoChu($cay, $mau) { return [bool](Tim $cay $mau) }

# Nút của khung cửa sổ, KHÔNG phải của ứng dụng. Bấm Space lên `Minimize` là
# cửa sổ thu nhỏ và phần còn lại của phép thử gõ vào hư không.
$KHUNG = @('Minimize', 'Maximize', 'Restore', 'Close', 'System', 'System Menu Bar')

# Tab cho tới khi tiêu điểm rơi đúng nút cần, rồi trả về true. KHÔNG dùng chuột.
function Toi-Nut($mau, $toi_da = 25) {
    for ($i = 0; $i -lt $toi_da; $i++) {
        Phim $VK.TAB
        $t = TieuDiem
        if ($KHUNG -contains $t.Ten) { continue }
        if ($t.Ten -like $mau) { return $true }
    }
    return $false
}
function Bam-Bang-Space { Phim $VK.SPACE; Start-Sleep -Milliseconds 700 }

# Tên màn hình đang mở = tiêu đề, tức phần tử Text đầu tiên trong cây — **trừ**
# dải thông báo đường lui của ĐM-08.
#
# Dải ấy nằm trong `TopBottomPanel` phía trên, nên khi trình đọc màn hình bật
# lên thì nó thành Text đầu tiên. Đứng đầu là ĐÚNG — người dùng trình đọc màn
# hình phải gặp đường lui trước mọi thứ khác. Sai là ở hàm này, và nó đã báo
# hỏng oan ĐM-08e một lần vì thế.
function Man-Hien-Tai {
    $c = Cay
    $t = $c | Where-Object { $_.Loai -eq 'Text' -and $_.Ten -notlike '*trình đọc màn hình*' } |
        Select-Object -First 1
    if ($t) { return $t.Ten } else { return '(không rõ)' }
}
# Đi tới một nút rồi bấm. Nói ra khi không tới được, thay vì lặng lẽ đi tiếp —
# lặng lẽ đi tiếp là cách một phép thử báo "đạt" trên một màn hình khác.
function Di($mau, $cho = 1200) {
    if (-not (Toi-Nut $mau)) {
        Write-Host ("       ✗ không Tab tới được '{0}' — đang ở màn '{1}'" -f $mau, (Man-Hien-Tai)) -ForegroundColor DarkYellow
        return $false
    }
    Bam-Bang-Space
    Start-Sleep -Milliseconds $cho
    return $true
}

# ---------------------------------------------------------------------------
# Đi lại KHÔNG qua bàn phím toàn cục, bằng `InvokePattern` của UIA.
#
# BP-01, BP-04 và §8.1-3 **phải** gõ phím thật — chúng chính là phép thử về bàn
# phím, gọi hàm thay thì chẳng kiểm được gì. Nhưng MAU-01, MAU-09 và ĐM-08 chỉ
# cần **tới đúng màn hình**; gõ phím toàn cục ở đó là rủi ro thừa.
#
# Rủi ro ấy không phải chuyện lý thuyết: máy này đang có người dùng, và một
# lượt chạy đã bị Chrome giành mất tiền cảnh **giữa** lúc kiểm và lúc gửi phím.
# Phần BP-01 gõ `Ctrl+A` rồi `Backspace` — rơi vào một bảng tính đang mở là
# hỏng dữ liệu thật của người ta.
function Bam-Qua-UIA($mau) {
    for ($lan = 0; $lan -lt 6; $lan++) {
        try {
            $r = [System.Windows.Automation.AutomationElement]::FromHandle($script:H)
            $tat = $r.FindAll([System.Windows.Automation.TreeScope]::Descendants,
                [System.Windows.Automation.Condition]::TrueCondition)
            foreach ($e in $tat) {
                if ($e.Current.ControlType -ne [System.Windows.Automation.ControlType]::Button) { continue }
                if ($e.Current.Name -notlike $mau) { continue }
                if (-not $e.Current.IsEnabled) { continue }
                $ip = $e.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
                $ip.Invoke()
                return $true
            }
        } catch { }
        Start-Sleep -Milliseconds 300
    }
    return $false
}
function Di-An-Toan($mau, $cho = 1200) {
    if (-not (Bam-Qua-UIA $mau)) {
        Write-Host ("       ✗ không gọi được nút '{0}' — đang ở màn '{1}'" -f $mau, (Man-Hien-Tai)) -ForegroundColor DarkYellow
        return $false
    }
    Start-Sleep -Milliseconds $cho
    return $true
}

# =============================================================== hộp cát
function Moi-Hop-Cat($soTep = 30) {
    $sb = Join-Path $env:TEMP ('k1_' + [Guid]::NewGuid().ToString('N').Substring(0, 8))
    if ($sb -notlike (Join-Path $env:TEMP '*') -or $sb -like '*ZaloData*') {
        throw "Đường dẫn hộp cát không an toàn: $sb"
    }
    New-Item -ItemType Directory -Force (Join-Path $sb 'video') | Out-Null
    $rnd = New-Object Random 20260802
    foreach ($i in 1..$soTep) {
        $p = Join-Path $sb "video\tep_$i.bin"
        $b = New-Object byte[] (2048 + $i * 64)
        $rnd.NextBytes($b)
        [IO.File]::WriteAllBytes($p, $b)
        (Get-Item $p).LastWriteTime = [datetime]'2023-05-10'
    }
    return $sb
}
function Dem-Tep($p) { return @(Get-ChildItem $p -Recurse -File -Force -EA SilentlyContinue).Count }

function Mo-App($sb, $w = 0, $h = 0, $canTienCanh = $true) {
    $p = Start-Process $exe -ArgumentList '-Root', $sb -PassThru
    $script:H = [IntPtr]::Zero
    for ($i = 0; $i -lt 80; $i++) {
        $p.Refresh()
        if ($p.MainWindowHandle -ne [IntPtr]::Zero) { $script:H = $p.MainWindowHandle; break }
        Start-Sleep -Milliseconds 250
    }
    if ($script:H -eq [IntPtr]::Zero) { throw 'Cửa sổ không mở ra' }
    Start-Sleep -Seconds 2
    if ($w -gt 0) {
        # Đặt cỡ **vùng vẽ** đúng bằng $w×$h. MoveWindow nhận cỡ cả khung, nên
        # phải bù phần viền — không bù thì DPI-04 đo hụt vài chục dip.
        $rc = New-Object K1.W+RECT; $rw = New-Object K1.W+RECT
        [void][K1.W]::GetClientRect($script:H, [ref]$rc); [void][K1.W]::GetWindowRect($script:H, [ref]$rw)
        $bx = ($rw.R - $rw.L) - ($rc.R - $rc.L); $by = ($rw.B - $rw.T) - ($rc.B - $rc.T)
        [void][K1.W]::MoveWindow($script:H, 30, 30, $w + $bx, $h + $by, $true)
        Start-Sleep -Milliseconds 900
    }
    # Ghim lên trên cùng suốt lượt chạy: cửa sổ khác đè lên là ảnh chụp hỏng và
    # tiền cảnh bị giành mất giữa chừng.
    [void][K1.W]::SetWindowPos($script:H, [IntPtr](-1), 0, 0, 0, 0, 0x0003)  # TOPMOST|NOMOVE|NOSIZE
    # Phần đi bằng UIA không cần tiền cảnh — và không đòi tiền cảnh nghĩa là
    # chạy được ngay cả khi người dùng đang làm việc khác trên máy.
    if ($canTienCanh) { Chac-Truoc } else { Start-Sleep -Milliseconds 500 }
    return $p
}
function Dong-App($p) {
    if ($p -and -not $p.HasExited) { Stop-Process -Id $p.Id -Force -EA SilentlyContinue }
    Start-Sleep -Milliseconds 500
}
function Don-Nhat-Ky {
    Get-ChildItem (Join-Path $goc 'rust\target\release\logs') -File -EA SilentlyContinue |
        Where-Object { $_.LastWriteTime -gt (Get-Date).AddMinutes(-20) } |
        Remove-Item -Force -EA SilentlyContinue
}

# ============================================================== chụp màn
function Chup($ten) {
    $r = New-Object K1.W+RECT
    $ok = [K1.W]::GetWindowRect($script:H, [ref]$r)
    $w = $r.R - $r.L; $ht = $r.B - $r.T
    # Khung 0×0 nghĩa là tay cầm cửa sổ đã chết, tức ứng dụng đã thoát. Nói
    # thẳng ra, đừng để `New-Object Bitmap` ném "Parameter is not valid" rồi cả
    # lượt chạy dừng ở một câu chẳng liên quan gì tới nguyên nhân.
    if (-not $ok -or $w -le 0 -or $ht -le 0) {
        $con = @(Get-Process zalo-gui -EA SilentlyContinue).Count
        throw ("Không chụp được '$ten': khung ${w}×${ht}, GetWindowRect=$ok, " +
               "còn $con tiến trình zalo-gui đang chạy")
    }
    $bmp = New-Object Drawing.Bitmap $w, $ht
    $g = [Drawing.Graphics]::FromImage($bmp)
    $dc = $g.GetHdc(); [void][K1.W]::PrintWindow($script:H, $dc, 2); $g.ReleaseHdc($dc); $g.Dispose()
    if (-not (Test-Path $ThuMucAnh)) { New-Item -ItemType Directory -Force $ThuMucAnh | Out-Null }
    $bmp.Save((Join-Path $ThuMucAnh "$ten.png"), [Drawing.Imaging.ImageFormat]::Png)
    # Bản greyscale theo hệ số độ chói ITU-R BT.601 — đúng thứ mà người mù màu
    # toàn phần và máy in đen trắng nhìn thấy.
    $x = New-Object Drawing.Bitmap $w, $ht
    $g2 = [Drawing.Graphics]::FromImage($x)
    $m = New-Object Drawing.Imaging.ColorMatrix
    $m.Matrix00 = 0.299; $m.Matrix01 = 0.299; $m.Matrix02 = 0.299
    $m.Matrix10 = 0.587; $m.Matrix11 = 0.587; $m.Matrix12 = 0.587
    $m.Matrix20 = 0.114; $m.Matrix21 = 0.114; $m.Matrix22 = 0.114
    $at = New-Object Drawing.Imaging.ImageAttributes; $at.SetColorMatrix($m)
    $g2.DrawImage($bmp, (New-Object Drawing.Rectangle 0, 0, $w, $ht), 0, 0, $w, $ht, [Drawing.GraphicsUnit]::Pixel, $at)
    $g2.Dispose()
    $x.Save((Join-Path $ThuMucAnh "$ten.xam.png"), [Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose(); $x.Dispose()
}

Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host '  CHÍN MỤC TIẾP CẬN MỨC 1 — chạy trên giao diện thật, trong hộp cát' -ForegroundColor Cyan
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host "  Ảnh chụp để lại ở: $ThuMucAnh" -ForegroundColor DarkGray
# Chỉ dọn thư mục ảnh khi chạy TRỌN BỘ. Chạy từng phần bằng `-Chi` mà vẫn dọn
# thì phần chạy sau xóa mất ảnh của phần trước, và bộ ảnh giao cho người thử
# chỉ còn đúng phần chạy cuối cùng.
if ($Chi -contains 'tat-ca' -and (Test-Path $ThuMucAnh)) {
    Remove-Item $ThuMucAnh -Recurse -Force -EA SilentlyContinue
}

# ═════════════════════════════════════════════ ① BP-01 · chỉ dùng bàn phím
if (Lam 'BP-01') {
    Muc 'BP-01 · rút chuột, chạy trọn kịch bản bằng bàn phím'
    $sb = Moi-Hop-Cat 30
    $truoc = Dem-Tep $sb
    $p = $null
    try {
        $p = Mo-App $sb
        $b = @{}   # từng chặng đi được hay không

        # Đi lại IM LẶNG là cách một lượt chạy hỏng mà không ai biết nó hỏng ở
        # đâu. Mỗi chặng nói ra mình đứng ở màn nào sau khi đi.
        function Buoc($ten, $mau, $cho = 700) {
            $ok = Toi-Nut $mau
            if ($ok) { Bam-Bang-Space; Start-Sleep -Milliseconds $cho }
            Write-Host ("       {0} {1,-26} → màn '{2}'" -f $(if ($ok) { '·' } else { '✗' }), $ten, (Man-Hien-Tai)) `
                -ForegroundColor $(if ($ok) { 'DarkGray' } else { 'DarkYellow' })
            return $ok
        }

        $b['chọn nguồn'] = Buoc 'chọn nguồn' '*Lấy lại dung lượng*'
        $b['quét'] = Buoc 'quét' '*Quét dữ liệu cũ hơn 12 tháng*' 4500
        $c = Cay
        $b['ra kết quả quét'] = (CoChu $c '*Số tệp*')

        $b['xem danh sách'] = Buoc 'xem danh sách' '*Xem danh sách tệp sắp mất*' 3500
        $c = Cay
        $b['thấy danh sách tệp'] = (CoChu $c '*Những tệp sắp mất*')
        Chup 'bp01-xem-danh-sach'

        Phim $VK.ESC; Start-Sleep -Seconds 1        # BP-06, quay về kết quả quét
        Ghi ("sau Esc: màn '{0}'" -f (Man-Hien-Tai))

        # ---- sao lưu, gồm cả gõ đường dẫn vào ô nhập bằng bàn phím
        $b['vào màn sao lưu'] = Buoc 'vào màn sao lưu' '*Sao lưu trước khi xóa*' 2000
        $dich = Join-Path $env:TEMP ('k1sl_' + [Guid]::NewGuid().ToString('N').Substring(0, 6))
        # Ô nhập đứng trước nút; Tab một lần từ đầu màn là tới nó.
        for ($i = 0; $i -lt 6; $i++) {
            Phim $VK.TAB
            if ((TieuDiem).Loai -eq 'Edit') { break }
        }
        $b['tới được ô nhập bằng Tab'] = ((TieuDiem).Loai -eq 'Edit')
        # KHÔNG dùng `Ctrl+A` rồi `Backspace` để dọn ô nhập. Nếu tiền cảnh bị
        # giành đúng lúc ấy thì cặp phím đó chọn hết rồi xóa hết trong cửa sổ
        # người khác đang mở. Hỏi UIA xem ô đang có gì, rồi xóa đúng bấy nhiêu
        # ký tự — thường là không có gì để xóa.
        $cu = Doc-O-Nhap
        Ghi ("ô nhập đang có {0} ký tự (đọc bằng {1}), tiêu điểm ở {2}" -f `
            $cu.Length, $script:CachDoc, (TieuDiem).Loai)
        if ($cu.Length -gt 0) { Phim $VK.END; Phim $VK.BS $cu.Length 40 }
        GoChu $dich
        Start-Sleep -Milliseconds 500
        # Đọc bằng `ValuePattern`, KHÔNG bằng `Name`. Nội dung ô nhập nằm ở
        # Value; Name của nó rỗng. Bản đầu tìm tên tệp trong Name của cả cây
        # nên báo "chưa gõ được" trong khi tay vẫn sao lưu xong 31 tệp vào
        # đúng thư mục ấy — hai kết quả chọi nhau, và cái sai là phép đo.
        $trong_o = Doc-O-Nhap
        Ghi ("ô nhập sau khi gõ: '{0}' (đọc bằng {1}); cần '{2}'" -f $trong_o, $script:CachDoc, $dich)
        $b['gõ được đường dẫn'] = ($trong_o -eq $dich)
        $b['bắt đầu sao lưu'] = Buoc 'bắt đầu sao lưu' '*Bắt đầu sao lưu*' 6000
        $b['sao lưu xong'] = ((Dem-Tep $dich) -ge $truoc)
        Ghi ("sao lưu: {0} tệp ở {1}" -f (Dem-Tep $dich), $dich)

        # Về lại kết quả quét.
        #
        # `Về trang chủ` là ngõ cụt: trang chủ không có đường nào tới kết quả
        # quét. Chỉ đi bằng `← Quay lại kết quả quét`, và nếu không thấy nút ấy
        # thì đây là hỏng thật chứ không phải đi nhầm đường.
        for ($i = 0; $i -lt 4; $i++) {
            if (CoChu (Cay) '*Xóa vĩnh viễn…*') { break }
            if (Toi-Nut '*Quay lại kết quả quét*' 8) { Bam-Bang-Space; Start-Sleep -Seconds 1 }
            elseif (Toi-Nut '*Quay lại*' 8) { Bam-Bang-Space; Start-Sleep -Seconds 1 }
            else { break }
        }
        $b['về được màn kết quả quét'] = (CoChu (Cay) '*Xóa vĩnh viễn…*')

        # ---- trang xác nhận
        $b['mở trang xác nhận'] = Toi-Nut '*Xóa vĩnh viễn…*'
        if ($b['mở trang xác nhận']) { Bam-Bang-Space; Start-Sleep -Seconds 2 }
        $b['ra trang xác nhận'] = (CoChu (Cay) '*Gõ đúng chữ*')
        for ($i = 0; $i -lt 5; $i++) {
            if ((TieuDiem).Loai -eq 'Edit') { break }
            Phim $VK.TAB
        }
        GoChu 'XÓA'
        Start-Sleep -Milliseconds 1200               # chờ hết khóa mồi 600 ms
        $c = Cay
        $nut = Tim $c '*Xóa vĩnh viễn'
        $b['nút xóa bật lên sau khi gõ'] = ($nut -and $nut.Bat)

        $b['bấm được nút xóa bằng bàn phím'] = $false
        if (Toi-Nut '*Xóa vĩnh viễn' 6) {
            Phim $VK.SPACE; Start-Sleep -Seconds 3
            if ((Dem-Tep $sb) -lt $truoc) { $b['bấm được nút xóa bằng bàn phím'] = $true }
            else {
                Phim $VK.ENTER; Start-Sleep -Seconds 3
                if ((Dem-Tep $sb) -lt $truoc) { $b['bấm được nút xóa bằng bàn phím'] = $true }
            }
        }
        Chup 'bp01-trang-xac-nhan'

        foreach ($k in $b.Keys | Sort-Object) {
            Write-Host ("       {0} {1}" -f $(if ($b[$k]) { '✓' } else { '✗' }), $k) `
                -ForegroundColor $(if ($b[$k]) { 'DarkGreen' } else { 'DarkYellow' })
        }
        $truocXoa = @('chọn nguồn', 'quét', 'ra kết quả quét', 'xem danh sách', 'thấy danh sách tệp',
            'vào màn sao lưu', 'tới được ô nhập bằng Tab', 'gõ được đường dẫn', 'bắt đầu sao lưu',
            'sao lưu xong', 'về được màn kết quả quét', 'mở trang xác nhận', 'ra trang xác nhận',
            'nút xóa bật lên sau khi gõ')
        $hong = @($truocXoa | Where-Object { -not $b[$_] })
        Assert 'BP-01a' 'Toàn bộ kịch bản TỚI TRƯỚC lệnh xóa làm được bằng bàn phím' `
            ($hong.Count -eq 0) ("chặng hỏng: " + ($hong -join ', '))

        # Đây là chỗ hai mục mức 1 đâm vào nhau, và mã đã chọn phía nào.
        #
        # Chỉ kết luận khi ĐÃ thật sự đứng trên trang xác nhận với nút đã bật.
        # Không có vế ấy thì "không bấm được" chỉ nghĩa là chưa đi tới nơi, mà
        # báo cáo lại đọc như thể đã chứng minh được điều gì.
        if ($b['ra trang xác nhận'] -and $b['nút xóa bật lên sau khi gõ']) {
            Assert 'BP-01b' 'Bấm được LỆNH XÓA bằng bàn phím' $b['bấm được nút xóa bằng bàn phím'] `
                ('Đã đứng đúng trên trang xác nhận, nút đã BẬT, gõ cả Space lẫn Enter mà ' +
                 '0 tệp mất. BP-05 điều 1/2/6 nuốt sạch Enter và Space ở đây, nên KHÔNG CÓ ' +
                 'đường nào từ bàn phím tới lệnh xóa. Hai mục mức 1 đâm nhau — xem kết luận.')
        } else {
            Assert 'BP-01b' 'Đi tới được trang xác nhận với nút đã bật, để mà đo' $false `
                'chưa tới nơi, nên chưa kết luận được gì về đường bàn phím tới lệnh xóa'
        }
        Remove-Item $dich -Recurse -Force -EA SilentlyContinue
    } catch {
        # In cả KIỂU ngoại lệ và DÒNG ném ra. "Access is denied" trần trụi thì
        # không truy được về đâu, mà mỗi lượt chạy lại tốn một cửa sổ máy yên tĩnh.
        Assert $script:MucDangChay 'chạy trọn phần này, không đứt giữa chừng' $false `
            ("{0}: {1}  (dòng {2}: {3})" -f $_.Exception.GetType().Name, $_.Exception.Message,
                $_.InvocationInfo.ScriptLineNumber, $_.InvocationInfo.Line.Trim())
    } finally {
        Dong-App $p
        Remove-Item $sb -Recurse -Force -EA SilentlyContinue
        Don-Nhat-Ky
    }
}

# ═════════════════════════════════════════ ② BP-04 · giam tiêu điểm, Tab 30
if (Lam 'BP-04') {
    Muc 'BP-04 · Tab 30 lần trên trang xác nhận, tiêu điểm không thoát ra'
    $sb = Moi-Hop-Cat 10
    $p = $null
    try {
        $p = Mo-App $sb
        if (Toi-Nut '*Lấy lại dung lượng*') { Bam-Bang-Space }
        if (Toi-Nut '*Quét dữ liệu cũ hơn 12 tháng*') { Bam-Bang-Space; Start-Sleep -Seconds 4 }
        if (Toi-Nut '*Xem danh sách tệp sắp mất*') { Bam-Bang-Space; Start-Sleep -Seconds 3 }
        Phim $VK.ESC; Start-Sleep -Seconds 1
        if (Toi-Nut '*Xóa vĩnh viễn…*') { Bam-Bang-Space; Start-Sleep -Seconds 2 }
        $tren_trang = (CoChu (Cay) '*Gõ đúng chữ*')
        Assert 'BP-04-0' 'Vào được trang xác nhận để mà đo' $tren_trang ''

        if ($tren_trang) {
            $vong = New-Object Collections.ArrayList
            for ($i = 1; $i -le 30; $i++) {
                Phim $VK.TAB 1 70
                $t = TieuDiem
                [void]$vong.Add("$($t.Loai)|$($t.Ten)")
            }
            $rieng = $vong | Select-Object -Unique
            Ghi ("30 chặng Tab rơi vào {0} phần tử khác nhau:" -f @($rieng).Count)
            foreach ($r in $rieng) { Ghi "   $r" }

            # Bộ widget HỢP LỆ của trang xác nhận. Bất cứ thứ gì khác là tiêu
            # điểm đã thoát ra ngoài — kể cả nó vẫn nằm trong cửa sổ này.
            $hop_le = @('*|Hủy', '*|Xóa vĩnh viễn', 'Edit|*', '*|Mở bản dòng lệnh')
            $lot = @($vong | Where-Object { $x = $_; -not ($hop_le | Where-Object { $x -like $_ }) })
            Assert 'BP-04' 'Cả 30 chặng Tab nằm trong bộ widget của trang xác nhận' `
                ($lot.Count -eq 0) ("thoát ra: " + (($lot | Select-Object -Unique) -join ' · '))

            # Nền phải vô hiệu: không widget nào của MÀN TRƯỚC còn bấm được.
            $c = Cay
            $con_nen = @($c | Where-Object {
                $_.Loai -eq 'Button' -and $_.Bat -and
                ($_.Ten -like '*Xem danh sách*' -or $_.Ten -like '*Sao lưu trước khi xóa*' -or
                 $_.Ten -like '*Bỏ kết quả này*' -or $_.Ten -like '*Quay lại*')
            })
            Assert 'BP-04n' 'Không widget nào của màn trước còn sống dưới trang xác nhận' `
                ($con_nen.Count -eq 0) (($con_nen | ForEach-Object { $_.Ten }) -join ', ')
            Chup 'bp04-trang-xac-nhan'
        }
    } catch {
        # Một phần đứt giữa chừng KHÔNG được kéo theo cả bộ chạy: các phần khác
        # vẫn đo được, và phần đứt phải hiện ra là ĐỎ chứ không phải là im lặng.
        Assert $script:MucDangChay 'chạy trọn phần này, không đứt giữa chừng' $false $_.Exception.Message
    } finally { Dong-App $p; Remove-Item $sb -Recurse -Force -EA SilentlyContinue; Don-Nhat-Ky }
}

# ═══════════════════════════════ ③ §8.1-3 · gõ XOÁ kiểu đặt dấu mới
if (Lam '8.1-3') {
    Muc '§8.1-3 · gõ XOÁ bằng bộ gõ tiếng Việt, kiểu đặt dấu mới'
    Ghi 'Gửi ĐÚNG chuỗi ký tự mà Unikey gửi ở bước cuối: backspace rồi ký tự đã ghép.'
    $sb = Moi-Hop-Cat 10
    $p = $null
    try {
        $p = Mo-App $sb
        if (Toi-Nut '*Lấy lại dung lượng*') { Bam-Bang-Space }
        if (Toi-Nut '*Quét dữ liệu cũ hơn 12 tháng*') { Bam-Bang-Space; Start-Sleep -Seconds 4 }
        if (Toi-Nut '*Xem danh sách tệp sắp mất*') { Bam-Bang-Space; Start-Sleep -Seconds 3 }
        Phim $VK.ESC; Start-Sleep -Seconds 1
        if (Toi-Nut '*Xóa vĩnh viễn…*') { Bam-Bang-Space; Start-Sleep -Seconds 2 }

        # Bốn kiểu gõ, mô phỏng đúng luồng ký tự mà bộ gõ đẩy vào ứng dụng.
        # Telex `XOAS`: Unikey xóa lùi rồi chèn nguyên âm đã mang dấu.
        $ca = @(
            @{ Ten = 'kiểu MỚI  · XOA + BS + Á  (dấu trên A)'; Ky = @('X', 'O', 'A', '<BS>', [string][char]0x00C1); Mong = $true }
            @{ Ten = 'kiểu CŨ   · XOA + BS×2 + ÓA (dấu trên O)'; Ky = @('X', 'O', 'A', '<BS>', '<BS>', [string][char]0x00D3, 'A'); Mong = $true }
            @{ Ten = 'không dấu · XOA'; Ky = @('X', 'O', 'A'); Mong = $true }
            @{ Ten = 'tổ hợp NFD· XOA + U+0301 sau A'; Ky = @('X', 'O', 'A', [string][char]0x0301); Mong = $true }
            @{ Ten = 'chữ thường· xoá kiểu mới'; Ky = @('x', 'o', 'a', '<BS>', [string][char]0x00E1); Mong = $false }
        )
        foreach ($t in $ca) {
            for ($i = 0; $i -lt 5; $i++) { if ((TieuDiem).Loai -eq 'Edit') { break }; Phim $VK.TAB }
            # Xóa đúng số ký tự đang có, không `Ctrl+A`. Xem chú thích ở BP-01:
            # `Ctrl+A` lạc vào cửa sổ người khác là chọn hết rồi xóa hết.
            $cu = Doc-O-Nhap
            if ($cu.Length -gt 0) { Phim $VK.END; Phim $VK.BS $cu.Length 40 }
            Start-Sleep -Milliseconds 200
            foreach ($k in $t.Ky) {
                if ($k -eq '<BS>') { Phim $VK.BS 1 80 } else { GoChu $k 80 }
            }
            Start-Sleep -Milliseconds 1200          # qua khóa mồi 600 ms
            $nut = Tim (Cay) '*Xóa vĩnh viễn'
            $bat = [bool]($nut -and $nut.Bat)
            Assert '8.1-3' ("{0} → nút xóa {1}" -f $t.Ten, $(if ($bat) { 'BẬT' } else { 'tắt' })) `
                ($bat -eq $t.Mong) ("mong đợi " + $(if ($t.Mong) { 'BẬT' } else { 'tắt' }))
        }
        Chup '813-o-nhap'
    } catch {
        # Một phần đứt giữa chừng KHÔNG được kéo theo cả bộ chạy: các phần khác
        # vẫn đo được, và phần đứt phải hiện ra là ĐỎ chứ không phải là im lặng.
        Assert $script:MucDangChay 'chạy trọn phần này, không đứt giữa chừng' $false $_.Exception.Message
    } finally { Dong-App $p; Remove-Item $sb -Recurse -Force -EA SilentlyContinue; Don-Nhat-Ky }
}

# ═══════════════════════════ ④ DPI-04 / DPI-08 / DPI-01 · cỡ màn và vị trí
if (Lam 'DPI') {
    Muc 'DPI-04 · vừa 1366×768 @125%  (tức 1092×614 dip)'
    $sb = Moi-Hop-Cat 40
    $p = $null
    try {
        $p = Mo-App $sb 1092 614

        # DPI-01 — mục MỨC 1 mà cổng M5 chưa từng nhắc tới. Không có tệp
        # manifest nào trong kho mã, nên phải hỏi tiến trình đang chạy.
        #
        # `GetAwarenessFromDpiAwarenessContext` KHÔNG phân biệt được V2: kiểu
        # `DPI_AWARENESS` chỉ có UNAWARE/SYSTEM/PER_MONITOR, và V2 cũng trả về
        # PER_MONITOR. Bản đầu của phép đo này báo hỏng vì lý do ấy — hỏng ở
        # phép đo chứ không phải ở công cụ. Phải so thẳng hai **ngữ cảnh**.
        $ctx = [K1.W]::GetWindowDpiAwarenessContext($script:H)
        $aw = [K1.W]::GetAwarenessFromDpiAwarenessContext($ctx)
        $ten = @{ 0 = 'UNAWARE'; 1 = 'SYSTEM_AWARE'; 2 = 'PER_MONITOR_AWARE'; 3 = 'UNAWARE_GDISCALED' }
        $laV2 = [K1.W]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-4))
        $laV1 = [K1.W]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-3))
        Ghi ("DPI awareness: {0} · PerMonitorV2={1} · PerMonitorV1={2} · DPI cửa sổ={3}" -f `
            $ten[$aw], $laV2, $laV1, [K1.W]::GetDpiForWindow($script:H))
        Assert 'DPI-01' 'Tiến trình chạy ở PerMonitorV2' $laV2 `
            "đang là $($ten[$aw]) chứ không phải V2 — chữ sẽ mờ khi kéo sang màn hình DPI khác"

        # DPI-08 — trang xác nhận là một TRANG trong cùng cửa sổ, không phải cửa
        # sổ con. Chứng minh bằng cách đếm cửa sổ cấp cao nhìn thấy được của
        # tiến trình: một cái thì không có gì để mở lệch chỗ.
        # Lọc theo PID của đúng tiến trình, và đọc tiêu đề bằng bản Unicode.
        # Bản đầu quên cả hai: `GetWindowText` mặc định là bản ANSI nên tiêu đề
        # về thành `D?n d?p Zalo`, phép so tên trượt, và bộ đếm chỉ bắt được một
        # cửa sổ phụ của winit rồi báo "đạt". Một cổng an toàn xanh vì lý do sai
        # thì tệ hơn là không có cổng.
        $ds = New-Object Collections.ArrayList
        $pidApp = $p.Id
        $cb = [K1.W+EnumProc] {
            param($hh, $ll)
            $pp = 0
            [void][K1.W]::GetWindowThreadProcessId($hh, [ref]$pp)
            if ($pp -ne $pidApp) { return $true }
            $s = New-Object Text.StringBuilder 300
            [void][K1.W]::GetWindowTextW($hh, $s, 300)
            $sc = New-Object Text.StringBuilder 300
            [void][K1.W]::GetClassNameW($hh, $sc, 300)
            $rr = New-Object K1.W+RECT
            [void][K1.W]::GetWindowRect($hh, [ref]$rr)
            [void]$ds.Add(@{ H = $hh; Ten = $s.ToString(); Lop = $sc.ToString()
                             Hien = [K1.W]::IsWindowVisible($hh)
                             W = $rr.R - $rr.L; H2 = $rr.B - $rr.T })
            return $true
        }

        $rc = New-Object K1.W+RECT
        [void][K1.W]::GetClientRect($script:H, [ref]$rc)
        $pt = New-Object K1.W+POINT
        [void][K1.W]::ClientToScreen($script:H, [ref]$pt)
        $phai = $pt.X + ($rc.R - $rc.L)
        $duoi = $pt.Y + ($rc.B - $rc.T)
        Ghi ("vùng vẽ {0}×{1} dip, mép phải ở x={2}" -f ($rc.R - $rc.L), ($rc.B - $rc.T), $phai)

        # Duyệt mọi màn hình, mỗi màn đo phần tử nào tràn khỏi mép phải.
        $tran_tong = New-Object Collections.ArrayList
        function Do-Tran($nhan) {
            $c = Cay
            $noi_dung = $c | Where-Object { $_.Loai -in @('Text', 'Button', 'Edit', 'CheckBox', 'RadioButton') }
            $tran = @($noi_dung | Where-Object { ($_.X + $_.W) -gt ($phai + 1) })
            Chup "dpi04-$nhan"
            if ($tran.Count -gt 0) {
                foreach ($t in $tran) { [void]$tran_tong.Add("$nhan : '$($t.Ten)' hết ở x=$($t.X + $t.W), quá $(($t.X + $t.W) - $phai) dip") }
            }
            Write-Host ("       {0} {1,-22} {2,2} phần tử, tràn {3}" -f `
                $(if ($tran.Count -eq 0) { '✓' } else { '✗' }), $nhan, @($noi_dung).Count, $tran.Count) `
                -ForegroundColor $(if ($tran.Count -eq 0) { 'DarkGreen' } else { 'DarkYellow' })
            return $nhan
        }

        $da_duyet = New-Object Collections.ArrayList
        function Ghe($nhan) { [void]$da_duyet.Add($nhan); Do-Tran $nhan | Out-Null }

        Ghe 'trang-chu'
        if (Di '*Xem vùng bảo vệ*' 2000) { Ghe 'vung-bao-ve'; [void](Di '*Quay lại*') }
        if (Di '*Khôi phục dữ liệu*' 2000) { Ghe 'khoi-phuc'; [void](Di '*Quay lại*') }
        if (Di '*Lấy lại dung lượng*') { Ghe 'lay-lai-dung-luong' }
        if (Di '*Quét dữ liệu cũ hơn 12 tháng*' 4500) { Ghe 'ket-qua-quet' }
        if (Di '*Xem danh sách tệp sắp mất*' 4500) { Ghe 'xem-danh-sach' }
        Phim $VK.ESC; Start-Sleep -Seconds 1
        if (Di '*Sao lưu trước khi xóa*' 2000) { Ghe 'sao-luu'; [void](Di '*Quay lại*' 1500) }
        if (Di '*Xóa vĩnh viễn…*' 2500) { Ghe 'xac-nhan-xoa' }

        # Duyệt HẾT mọi màn hình là một phần của cổng. Bỏ sót một màn rồi báo
        # "không tràn" là báo cáo về những màn đã ghé, không phải về công cụ.
        $can_ghe = @('trang-chu', 'vung-bao-ve', 'khoi-phuc', 'lay-lai-dung-luong',
            'ket-qua-quet', 'xem-danh-sach', 'sao-luu', 'xac-nhan-xoa')
        $thieu = @($can_ghe | Where-Object { $da_duyet -notcontains $_ })
        Assert 'DPI-04a' 'Ghé đủ 8 màn hình để mà đo' ($thieu.Count -eq 0) ("chưa ghé: " + ($thieu -join ', '))
        Assert 'DPI-04' 'Không phần tử nào tràn khỏi vùng vẽ ở 1092×614 dip' `
            ($tran_tong.Count -eq 0) (($tran_tong | Select-Object -First 6) -join ' | ')

        # ---- DPI-08.
        #
        # Hội đồng đòi: trang xác nhận mở **canh giữa cửa sổ cha, cùng màn hình,
        # chặn cửa sổ cha**. Ở đây nó không phải cửa sổ con — nó là một TRANG vẽ
        # trong chính cửa sổ ấy. Nên câu hỏi đúng không phải "đếm được mấy cửa
        # sổ" mà là "widget của trang xác nhận nằm ở đâu".
        #
        # Đếm cửa sổ suông thì lệch: winit nào cũng có một cửa sổ phụ 16×16 tên
        # rỗng để nhận thông điệp, và nếu lấy nó làm cớ báo hỏng thì phép thử
        # đang đo winit chứ không đo công cụ này.
        [void][K1.W]::EnumWindows($cb, [IntPtr]::Zero)
        Ghi ("cửa sổ cấp cao của tiến trình {0}: {1}" -f $pidApp, $ds.Count)
        foreach ($w in $ds) { Ghi ("   hiện={0} {1}×{2} lớp='{3}' tên='{4}'" -f $w.Hien, $w.W, $w.H2, $w.Lop, $w.Ten) }

        $tren_trang = CoChu (Cay) '*Gõ đúng chữ*'
        Assert 'DPI-08a' 'Đang thật sự đứng trên trang xác nhận lúc đo' $tren_trang `
            'không ở trên trang xác nhận — mọi phép đo dưới đây vô nghĩa'

        # ① Không cửa sổ nào KHÁC cửa sổ chính phơi ra widget. Cửa sổ nào có
        #    widget thì mới có thể mở lệch chỗ hay mở sang màn hình khác.
        $co_widget = New-Object Collections.ArrayList
        foreach ($w in $ds) {
            if ($w.H -eq $script:H) { continue }
            try {
                $e = [System.Windows.Automation.AutomationElement]::FromHandle($w.H)
                $n = $e.FindAll([System.Windows.Automation.TreeScope]::Descendants,
                    [System.Windows.Automation.Condition]::TrueCondition).Count
                if ($n -gt 0) { [void]$co_widget.Add("$($w.Lop) '$($w.Ten)' có $n widget") }
            } catch { }
        }
        Assert 'DPI-08b' 'Không cửa sổ nào ngoài cửa sổ chính phơi ra widget' `
            ($co_widget.Count -eq 0) ($co_widget -join ' | ')

        # ② Mọi widget của trang xác nhận nằm gọn trong vùng vẽ của cửa sổ chính.
        $c = Cay
        $cua_trang = @($c | Where-Object { $_.Ten -like '*Xóa vĩnh viễn' -or $_.Ten -eq 'Hủy' -or $_.Loai -eq 'Edit' })
        $ngoai = @($cua_trang | Where-Object {
            $_.X -lt $pt.X -or $_.Y -lt $pt.Y -or ($_.X + $_.W) -gt $phai -or ($_.Y + $_.H) -gt $duoi })
        foreach ($t in $cua_trang) { Ghi ("   '{0}' ở ({1},{2}) {3}×{4}" -f $t.Ten, $t.X, $t.Y, $t.W, $t.H) }
        Assert 'DPI-08c' 'Widget của trang xác nhận nằm gọn trong vùng vẽ cửa sổ chính' `
            ($cua_trang.Count -ge 3 -and $ngoai.Count -eq 0) `
            "thấy $($cua_trang.Count) widget, $($ngoai.Count) cái nằm ngoài"

        # ③ Cùng màn hình với cửa sổ cha — hiển nhiên đúng khi nó vẽ trong cùng
        #    cửa sổ, nhưng vẫn đo để câu kết luận là số đo chứ không phải suy luận.
        $man_cha = [Windows.Forms.Screen]::FromHandle($script:H).DeviceName
        $khac_man = @($cua_trang | Where-Object {
            $p2 = New-Object Drawing.Point ($_.X + [int]($_.W / 2)), ($_.Y + [int]($_.H / 2))
            [Windows.Forms.Screen]::FromPoint($p2).DeviceName -ne $man_cha })
        Assert 'DPI-08d' "Trang xác nhận nằm trên cùng màn hình với cửa sổ cha ($man_cha)" `
            ($khac_man.Count -eq 0) "$($khac_man.Count) widget rơi sang màn hình khác"
    } catch {
        # Một phần đứt giữa chừng KHÔNG được kéo theo cả bộ chạy: các phần khác
        # vẫn đo được, và phần đứt phải hiện ra là ĐỎ chứ không phải là im lặng.
        Assert $script:MucDangChay 'chạy trọn phần này, không đứt giữa chừng' $false $_.Exception.Message
    } finally { Dong-App $p; Remove-Item $sb -Recurse -Force -EA SilentlyContinue; Don-Nhat-Ky }
}

# ══════════════════════ ⑤ MAU-01 / MAU-09 / §8.1-2 · bỏ hết màu vẫn hiểu
if (Lam 'MAU') {
    Muc 'MAU-01 · ba mức rủi ro phân biệt được khi bỏ hết màu'
    $sb = Moi-Hop-Cat 12
    $p = $null
    try {
        $p = Mo-App $sb 0 0 $false
        [void](Di-An-Toan '*Lấy lại dung lượng*')
        Chup 'mau01-ba-muc'
        $c = Cay
        # Ba dòng mức rủi ro nằm cạnh nhau trên đúng màn này. Đọc ra từ cây UIA,
        # tức là đọc đúng thứ trình đọc màn hình đọc và mắt người nhìn thấy.
        $dong = @($c | Where-Object { $_.Loai -eq 'Text' -and $_.Ten -match '^[^\s]' -and
            ($_.Ten -like '*An toàn*' -or $_.Ten -like '*Cần cân nhắc*' -or $_.Ten -like '*mất vĩnh viễn*') })
        foreach ($d in $dong) { Ghi "   $($d.Ten)" }

        # Đếm theo MỨC, không theo DÒNG.
        #
        # Bản đầu của phép đo này so số ký hiệu riêng với số dòng, rồi báo hỏng.
        # Đọc lại đầu ra thì hỏng nằm ở phép đo: màn này có **ba lối quét nhưng
        # chỉ hai mức** — tìm bản trùng lặp và quét cache Zalo đều là "An toàn".
        # Hai dòng cùng mức thì đương nhiên chung một ký hiệu, và phải thế.
        #
        # Câu hỏi đúng của MAU-01: ký hiệu và câu chữ có **đi cùng nhau** không —
        # hai mức khác nhau thì phải khác cả hai, hai dòng cùng mức thì phải
        # giống cả hai. Lệch một vế là một trong hai lớp mã hóa đang nói dối.
        $ky = @($dong | ForEach-Object { $_.Ten.Substring(0, 1) } | Select-Object -Unique)
        $chu = @($dong | ForEach-Object { ($_.Ten -split '\s{2,}')[1] } | Select-Object -Unique)
        Ghi ("thấy {0} dòng thuộc {1} mức khác nhau" -f @($dong).Count, $chu.Count)
        Assert 'MAU-01a' 'Ký hiệu và câu chữ khớp nhau một-một theo mức rủi ro' `
            ($chu.Count -ge 2 -and $ky.Count -eq $chu.Count) `
            ("{0} câu chữ nhưng {1} ký hiệu — hai mức đang chung ký hiệu, hoặc một mức có hai ký hiệu" -f $chu.Count, $ky.Count)
        Assert 'MAU-01b' 'Mỗi câu chữ nói ra HẬU QUẢ, không chỉ dán nhãn mức độ' `
            (@($chu | Where-Object { $_ -match 'mất|tải lại' }).Count -eq $chu.Count) `
            ("câu chữ: " + ($chu -join ' | '))
        # Mức giữa — "Cần cân nhắc" — KHÔNG có lối nào tới từ giao diện: nó thuộc
        # CACHE HỆ THỐNG, mà bản đồ họa chưa mở lối quét ấy. Nói ra chứ đừng để
        # báo cáo đọc như thể cả ba mức đã được người thử nhìn qua.
        if (-not (CoChu $dong '*Cần cân nhắc*')) {
            ConNguoi 'MAU-01' ('Mức "Cần cân nhắc" KHÔNG hiện ở bất kỳ màn nào của bản đồ họa ' +
                '(nó thuộc CACHE HỆ THỐNG, giao diện chưa mở lối quét ấy). Người thử chỉ xếp được hai mức.')
        }

        # Đo trên ẢNH GREYSCALE thật, không đo trên chuỗi trong mã nguồn: câu hỏi
        # của MAU-01 là "bỏ màu đi thì trên MÀN HÌNH còn phân biệt được không".
        $anh = Join-Path $ThuMucAnh 'mau01-ba-muc.xam.png'
        $bm = [Drawing.Bitmap]::FromFile($anh)
        $oX = $script:H; $rw = New-Object K1.W+RECT
        [void][K1.W]::GetWindowRect($script:H, [ref]$rw)
        $khac = New-Object Collections.ArrayList
        foreach ($d in $dong) {
            # Ô vuông ôm lấy ký hiệu đầu dòng, quy về tọa độ trong ảnh.
            $x0 = $d.X - $rw.L; $y0 = $d.Y - $rw.T
            $tong = 0; $dem = 0; $vet = ''
            for ($y = $y0; $y -lt [math]::Min($y0 + $d.H, $bm.Height); $y++) {
                for ($x = $x0; $x -lt [math]::Min($x0 + 14, $bm.Width); $x++) {
                    if ($x -lt 0 -or $y -lt 0) { continue }
                    $g = $bm.GetPixel($x, $y).R
                    $tong += $g; $dem++
                    $vet += $(if ($g -lt 140) { '#' } else { '.' })
                }
            }
            [void]$khac.Add([pscustomobject]@{
                Ky = $d.Ten.Substring(0, 1); Muc = ($d.Ten -split '\s{2,}')[1]
                Vet = $vet; TB = [int]($tong / [math]::Max($dem, 1)) })
        }
        # So theo MỨC: hai mức khác nhau phải cho hai hình khác nhau, và hai
        # dòng CÙNG mức phải cho cùng một hình. Đếm suông số hình riêng thì
        # hai dòng "An toàn" giống nhau lại bị tính là hỏng — mà đó mới là đúng.
        $theo_muc = $khac | Group-Object Muc
        foreach ($g in $theo_muc) {
            $v = @($g.Group | ForEach-Object { $_.Vet } | Select-Object -Unique)
            Ghi ("   mức '{0}' · ký hiệu '{1}' · {2} dòng · {3} hình khác nhau · xám TB {4}" -f `
                $g.Name, $g.Group[0].Ky, $g.Count, $v.Count, $g.Group[0].TB)
        }
        $trong_muc_giong = @($theo_muc | Where-Object {
            (@($_.Group | ForEach-Object { $_.Vet } | Select-Object -Unique)).Count -ne 1 })
        $hinh_moi_muc = @($theo_muc | ForEach-Object { $_.Group[0].Vet } | Select-Object -Unique)
        Assert 'MAU-01c' 'Trên ẢNH ĐÃ BỎ MÀU, mỗi mức vẽ ra một hình riêng' `
            ($hinh_moi_muc.Count -eq $theo_muc.Count -and $theo_muc.Count -ge 2) `
            'hai mức vẽ ra cùng một hình sau khi bỏ màu — lớp ký hiệu đã biến mất'
        Assert 'MAU-01d' 'Hai dòng cùng một mức vẽ ra cùng một hình' `
            ($trong_muc_giong.Count -eq 0) `
            'cùng một mức mà hai dòng vẽ khác nhau — ký hiệu không còn nói lên mức nữa'
        $bm.Dispose()

        # ---- MAU-09: nút phá hủy khác nút Hủy bằng chữ + biểu tượng + vị trí
        Muc 'MAU-09 · nút phá hủy khác nút Hủy bằng chữ, biểu tượng và vị trí'
        [void](Di-An-Toan '*Quét dữ liệu cũ hơn 12 tháng*' 4500)
        [void](Di-An-Toan '*Xem danh sách tệp sắp mất*' 4000)
        [void](Di-An-Toan '*Quay lại*' 1500)
        [void](Di-An-Toan '*Xóa vĩnh viễn…*' 2500)
        Chup 'mau09-hai-nut'
        $c = Cay
        $huy = Tim $c 'Hủy'
        $xoa = Tim $c '*Xóa vĩnh viễn'
        if ($huy -and $xoa) {
            Ghi ("Hủy : x={0} y={1} {2}×{3}  '{4}'" -f $huy.X, $huy.Y, $huy.W, $huy.H, $huy.Ten)
            Ghi ("Xóa : x={0} y={1} {2}×{3}  '{4}'" -f $xoa.X, $xoa.Y, $xoa.W, $xoa.H, $xoa.Ten)
            Assert 'MAU-09a' 'Khác nhau về CHỮ' ($huy.Ten -ne $xoa.Ten) ''
            Assert 'MAU-09b' 'Nút phá hủy có biểu tượng, nút Hủy thì không' `
                ($xoa.Ten -match '^\W' -and $huy.Ten -notmatch '^\W') "Hủy='$($huy.Ten)' Xóa='$($xoa.Ten)'"
            $cach = $xoa.X - ($huy.X + $huy.W)
            Ghi ("khoảng cách hai nút: {0} dip" -f $cach)
            Assert 'MAU-09c' 'Khác nhau về VỊ TRÍ, và cách nhau đủ xa để không bấm nhầm' `
                ($cach -ge 48) "chỉ cách $cach dip, DPI-05 đòi ≥ 48"
            Assert 'MAU-09d' 'Nút Hủy đứng TRƯỚC nút phá hủy theo thứ tự đọc' ($huy.X -lt $xoa.X) ''
        } else {
            Assert 'MAU-09' 'Tìm được hai nút trên trang xác nhận' $false 'không thấy Hủy hoặc Xóa vĩnh viễn'
        }
    } catch {
        # Một phần đứt giữa chừng KHÔNG được kéo theo cả bộ chạy: các phần khác
        # vẫn đo được, và phần đứt phải hiện ra là ĐỎ chứ không phải là im lặng.
        Assert $script:MucDangChay 'chạy trọn phần này, không đứt giữa chừng' $false $_.Exception.Message
    } finally { Dong-App $p; Remove-Item $sb -Recurse -Force -EA SilentlyContinue; Don-Nhat-Ky }
}

# ═════════════════════════════════ ⑥ ĐM-08 · trình đọc màn hình
if (Lam 'DM-08') {
    Muc 'ĐM-08 · phát hiện trình đọc màn hình và mở đường lui'
    $SPI_GETSCREENREADER = 0x0046
    $SPI_SETSCREENREADER = 0x0047
    $co = $false
    [void][K1.W]::SpiGet($SPI_GETSCREENREADER, 0, [ref]$co, 0)
    Ghi "SPI_GETSCREENREADER lúc bắt đầu: $co"

    # Vế ①: một trình đọc màn hình THẬT có bật cờ ấy không.
    if ($KemNarrator) {
        $nar = Join-Path $env:SystemRoot 'System32\Narrator.exe'
        Start-Process $nar | Out-Null
        $bat = $false
        for ($i = 0; $i -lt 30; $i++) {
            Start-Sleep -Milliseconds 500
            [void][K1.W]::SpiGet($SPI_GETSCREENREADER, 0, [ref]$bat, 0)
            if ($bat) { break }
        }
        Assert 'ĐM-08a' 'Trình đọc màn hình THẬT (Narrator) bật SPI_GETSCREENREADER' $bat `
            'Narrator chạy mà cờ không lên — phép dò của ta dựa vào một cờ không ai bật'
        Stop-Process -Name Narrator -Force -EA SilentlyContinue
        Start-Sleep -Seconds 2
    } else {
        ConNguoi 'ĐM-08a' 'Bật NVDA thật (máy này chưa cài) và xem dải thông báo có hiện ra không. Chạy lại với -KemNarrator để thử bằng Narrator của Windows.'
        Ghi 'Bỏ qua vế Narrator (chạy lại với -KemNarrator để bật). Máy này CHƯA CÀI NVDA.'
    }

    # Vế ②: giao diện có phản ứng đúng khi cờ ấy bật không. Đặt cờ ở mức phiên,
    # không ghi vào ini, và trả lại ở `finally`.
    $sb = Moi-Hop-Cat 6
    $p = $null
    $daDat = $false
    try {
        [void][K1.W]::SpiSet($SPI_SETSCREENREADER, 1, [IntPtr]::Zero, 0)
        $daDat = $true
        $x = $false
        [void][K1.W]::SpiGet($SPI_GETSCREENREADER, 0, [ref]$x, 0)
        Assert 'ĐM-08b' 'Đặt được cờ trình đọc màn hình để thử' $x ''

        $p = Mo-App $sb 0 0 $false
        Start-Sleep -Seconds 5              # app dò lại theo nhịp 3 giây
        $c = Cay
        Chup 'dm08-dai-duong-lui'
        Assert 'ĐM-08c' 'Dải thông báo hiện ra và nói rõ bản dòng lệnh là đường chính thức' `
            (CoChu $c '*đường tiếp cận chính thức*') 'không thấy câu nhắn nào'
        $nut = Tim $c 'Mở bản dòng lệnh'
        Assert 'ĐM-08d' 'Có nút [Mở bản dòng lệnh] và nút ấy đang bật' ([bool]($nut -and $nut.Bat)) ''

        # Dải phải hiện trên MỌI màn hình, không chỉ trang chủ. Đây là chỗ dễ
        # sai nhất của ĐM-08: người ta bật trình đọc màn hình lúc đã đi sâu vào
        # giữa luồng, và đó đúng là lúc cần đường lui nhất.
        [void](Di-An-Toan '*Xem vùng bảo vệ*' 2000)
        $sau_khi_di = Man-Hien-Tai
        Ghi "đã đi tới màn '$sau_khi_di'"
        Assert 'ĐM-08e' 'Dải vẫn hiện khi đã đi sâu vào giữa luồng' `
            ((CoChu (Cay) '*đường tiếp cận chính thức*') -and $sau_khi_di -like '*bảo vệ*') `
            'dải biến mất khi rời trang chủ, hoặc không rời được trang chủ để mà kiểm'

        # Bấm thật, và xem bản dòng lệnh có chạy lên không.
        $truoc_cli = @(Get-Process zalo-cli -EA SilentlyContinue).Count
        $bam = Bam-Qua-UIA 'Mở bản dòng lệnh'
        Start-Sleep -Seconds 3
        $sau_cli = @(Get-Process zalo-cli -EA SilentlyContinue).Count
        Assert 'ĐM-08f' 'Bấm nút thì bản dòng lệnh chạy lên thật' ($bam -and $sau_cli -gt $truoc_cli) `
            "gọi được nút: $bam; trước $truoc_cli, sau $sau_cli tiến trình zalo-cli"
        Get-Process zalo-cli -EA SilentlyContinue | Stop-Process -Force -EA SilentlyContinue
    } catch {
        Assert $script:MucDangChay 'chạy trọn phần này, không đứt giữa chừng' $false $_.Exception.Message
    } finally {
        Dong-App $p
        if ($daDat) { [void][K1.W]::SpiSet($SPI_SETSCREENREADER, 0, [IntPtr]::Zero, 0) }
        $z = $true
        [void][K1.W]::SpiGet($SPI_GETSCREENREADER, 0, [ref]$z, 0)
        Ghi "SPI_GETSCREENREADER sau khi dọn: $z"
        Get-Process zalo-cli -EA SilentlyContinue | Stop-Process -Force -EA SilentlyContinue
        Remove-Item $sb -Recurse -Force -EA SilentlyContinue
        Don-Nhat-Ky
    }
}

# ══════════════════════════════════════════════════════ phần còn lại cho người
ConNguoi '§8.1-2' ("Ba người thử nhìn ảnh greyscale ở $ThuMucAnh và xếp mức rủi ro — cần 33/33. " +
    "Phần máy đo được (ký hiệu, chữ, hình sau khi bỏ màu) đã chạy ở trên.")
ConNguoi 'MAU-09' 'Người thử nhìn ảnh mau09-hai-nut.xam.png và chỉ ra đâu là nút Hủy. Phần đo được (chữ, biểu tượng, vị trí, khoảng cách) đã chạy ở trên.'

Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host ("  Đo được bằng máy : ĐẠT {0} · HỎNG {1}" -f $script:Pass, $script:Fail) `
    -ForegroundColor $(if ($script:Fail -eq 0) { 'Green' } else { 'Red' })
Write-Host ("  Còn cần người    : {0} việc" -f $script:ConLai.Count) -ForegroundColor Yellow
foreach ($v in $script:ConLai) { Write-Host ("     {0,-8} {1}" -f $v.Ma, $v.Viec) -ForegroundColor DarkYellow }
Write-Host "  Ảnh chụp: $ThuMucAnh" -ForegroundColor DarkGray
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
if ($script:Fail -gt 0) { exit 1 }
exit 0
