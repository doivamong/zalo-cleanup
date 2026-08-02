#Requires -Version 5.1
<#
    §8.1-1 của hội đồng — phép thử ma sát trên GIAO DIỆN THẬT.

    "Giữ phím Enter liên tục ... tới lúc trang xác nhận xóa mở, giữ thêm 5 giây.
     Lặp lại với Space và với chuột nhấp liên tục vào tọa độ nút Xóa.
     ĐẠT khi 0 tệp biến mất cả ba lần."

    Vì sao phép thử này tồn tại dù máy trạng thái đã có 60 phép thử đơn vị: hai
    thứ đó kiểm hai chuyện khác nhau. Phép thử đơn vị kiểm **luật**; phép thử
    này kiểm luật ấy có thật sự nối đúng vào cửa sổ thật không. Chúng tách nhau
    hẳn — egui cho kích hoạt nút bằng Enter và Space ở tầng thư viện, chỗ máy
    trạng thái không nhìn thấy được.

    ------------------------------------------------------------------
    CHỈ CHẠY TRÊN HỘP CÁT trong %TEMP%. Script tự dựng dữ liệu giả và tự dọn.
    Nó KHÔNG BAO GIỜ được trỏ vào thư mục Zalo thật — có chốt chặn ở dưới.
    ------------------------------------------------------------------

    Tọa độ bấm tính theo **gốc vùng vẽ** lấy từ `ClientToScreen`, không theo
    `GetWindowRect`: khung cửa sổ của winit có viền vô hình, nên hai gốc lệch
    nhau vài chục điểm ảnh và mọi cú bấm trượt hết.
#>
$ErrorActionPreference = 'Stop'
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

$exe = Join-Path (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent) 'rust\target\release\zalo-gui.exe'
if (-not (Test-Path $exe)) { throw "Chưa dựng $exe" }

Add-Type -Namespace MS -Name I -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
[DllImport("user32.dll")] public static extern void mouse_event(uint f,uint x,uint y,uint d,int e);
[DllImport("user32.dll")] public static extern void keybd_event(byte vk,byte sc,uint f,int e);
[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
[DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
public struct POINT { public int X,Y; }
'@

$script:H = [IntPtr]::Zero
function Cho-CuaSo($proc) {
    for ($i = 0; $i -lt 80; $i++) {
        $proc.Refresh()
        if ($proc.MainWindowHandle -ne [IntPtr]::Zero) { $script:H = $proc.MainWindowHandle; return }
        Start-Sleep -Milliseconds 250
    }
    throw 'Cửa sổ không mở ra'
}
function Goc-Ve {
    $p = New-Object MS.I+POINT
    [void][MS.I]::ClientToScreen($script:H, [ref]$p)
    return $p
}
function Len-Truoc { [void][MS.I]::SetForegroundWindow($script:H); Start-Sleep -Milliseconds 250 }
function Bam($dx, $dy) {
    $g = Goc-Ve; Len-Truoc
    [void][MS.I]::SetCursorPos($g.X + $dx, $g.Y + $dy); Start-Sleep -Milliseconds 150
    [MS.I]::mouse_event(0x02, 0, 0, 0, 0); Start-Sleep -Milliseconds 80
    [MS.I]::mouse_event(0x04, 0, 0, 0, 0); Start-Sleep -Milliseconds 600
}
# Giữ phím: gửi KEYDOWN liên tục KHÔNG kèm KEYUP, đúng như bàn phím thật sinh ra
# khi người ta đè phím. Đây là chỗ điều 6 của BP-05 phải chặn.
function Giu-Phim($vk, $giay) {
    Len-Truoc
    $het = (Get-Date).AddSeconds($giay)
    while ((Get-Date) -lt $het) { [MS.I]::keybd_event($vk, 0, 0, 0); Start-Sleep -Milliseconds 25 }
    [MS.I]::keybd_event($vk, 0, 2, 0)
    Start-Sleep -Milliseconds 400
}
function Go-Phim($vk) {
    [MS.I]::keybd_event($vk, 0, 0, 0); Start-Sleep -Milliseconds 40
    [MS.I]::keybd_event($vk, 0, 2, 0); Start-Sleep -Milliseconds 150
}
# Gõ chữ HOA phải giữ Shift. Bản đầu của bộ chạy này gửi phím trần và ô nhập
# nhận `xoa` chữ thường — mà chữ thường thì KHÔNG được mở khóa (TV-01), nên nút
# xóa tắt đúng như phải thế. Lỗi nằm ở bộ chạy, không nằm ở công cụ; nhưng nếu
# không nhìn vào ô nhập thì rất dễ kết luận ngược lại.
function Go-Chu-Hoa($vk) {
    [MS.I]::keybd_event(0x10, 0, 0, 0)          # SHIFT xuống
    [MS.I]::keybd_event($vk, 0, 0, 0); Start-Sleep -Milliseconds 40
    [MS.I]::keybd_event($vk, 0, 2, 0)
    [MS.I]::keybd_event(0x10, 0, 2, 0)          # SHIFT lên
    Start-Sleep -Milliseconds 150
}
function Dem-Tep($p) { return @(Get-ChildItem $p -Recurse -File -Force -EA SilentlyContinue).Count }

# ---------------------------------------------------------------- hộp cát
$sb = Join-Path $env:TEMP ('mstest_' + [Guid]::NewGuid().ToString('N').Substring(0, 8))
if ($sb -notlike (Join-Path $env:TEMP '*') -or $sb -like '*ZaloData*') {
    throw "Đường dẫn hộp cát không an toàn: $sb"
}
New-Item -ItemType Directory -Force (Join-Path $sb 'video') | Out-Null
$rnd = New-Object Random 20260802
foreach ($i in 1..30) {
    $p = Join-Path $sb "video\tep_$i.bin"
    $b = New-Object byte[] (2048 + $i * 64)
    $rnd.NextBytes($b)
    [IO.File]::WriteAllBytes($p, $b)
    (Get-Item $p).LastWriteTime = [datetime]'2023-05-10'
}
$truoc = Dem-Tep $sb
Write-Host "Hộp cát : $sb" -ForegroundColor DarkGray
Write-Host "Dựng    : $truoc tệp, ngày 10/05/2023" -ForegroundColor DarkGray

$script:Pass = 0; $script:Fail = 0
function Assert($ten, $ok, $ct) {
    if ($ok) { $script:Pass++; Write-Host "  [ĐẠT ] $ten" -ForegroundColor Green }
    else { $script:Fail++; Write-Host "  [HỎNG] $ten" -ForegroundColor Red
           if ($ct) { Write-Host "         $ct" -ForegroundColor DarkRed } }
}

$proc = $null
try {
    $proc = Start-Process $exe -ArgumentList '-Root', $sb -PassThru
    Cho-CuaSo $proc
    Start-Sleep -Seconds 2

    # Trang chủ → Lấy lại dung lượng → Quét dữ liệu cũ hơn 12 tháng
    Bam 82 140
    Bam 97 184
    Start-Sleep -Seconds 3
    Assert 'Quét xong, chưa mất tệp nào' ((Dem-Tep $sb) -eq $truoc) "còn $(Dem-Tep $sb)/$truoc"

    # Mở chốt xem trước rồi rời danh sách bằng Esc (BP-06).
    Bam 93 140
    Start-Sleep -Seconds 3
    Go-Phim 0x1B
    Start-Sleep -Seconds 1

    Write-Host ''
    Write-Host '── Ba phép thử ma sát §8.1-1' -ForegroundColor Yellow

    # ① Giữ Enter ở màn kết quả quét — nút xóa vừa mới bật lên ở đây.
    Giu-Phim 0x0D 5
    Assert '① giữ Enter 5 giây ở màn kết quả quét' ((Dem-Tep $sb) -eq $truoc) "còn $(Dem-Tep $sb)/$truoc"

    # Vào trang xác nhận xóa, rồi đưa con trỏ về đúng chỗ nút Xóa và để yên ở
    # đó suốt phần còn lại — người dùng đang rê chuột sẵn ở nút.
    Bam 68 216
    Start-Sleep -Seconds 2
    $g = Goc-Ve
    [void][MS.I]::SetCursorPos($g.X + 145, $g.Y + 220)

    # ① tiếp — giữ Enter ngay trên trang xác nhận.
    Giu-Phim 0x0D 5
    Assert '① giữ Enter 5 giây trên TRANG XÁC NHẬN' ((Dem-Tep $sb) -eq $truoc) "còn $(Dem-Tep $sb)/$truoc"

    # ② Giữ Space.
    Giu-Phim 0x20 5
    Assert '② giữ Space 5 giây trên trang xác nhận' ((Dem-Tep $sb) -eq $truoc) "còn $(Dem-Tep $sb)/$truoc"

    # ③ Nhấp liên tục vào ĐÚNG TỌA ĐỘ nút Xóa, không gõ gì cả.
    #
    # Đây là cách hội đồng viết: người dùng nhấp sẵn ở chỗ nút sắp hiện ra và
    # cứ thế nhấp. Không gõ cụm từ, nên nút không bao giờ được phép bật.
    for ($i = 0; $i -lt 200; $i++) {
        [MS.I]::mouse_event(0x02, 0, 0, 0, 0); Start-Sleep -Milliseconds 10
        [MS.I]::mouse_event(0x04, 0, 0, 0, 0); Start-Sleep -Milliseconds 10
    }
    Assert '③ nhấp liên tục 200 lần vào tọa độ nút Xóa, không gõ gì' `
        ((Dem-Tep $sb) -eq $truoc) "còn $(Dem-Tep $sb)/$truoc"

    # ④ Gõ đúng cụm từ RỒI nhấp ngay lập tức — khóa mồi 600 ms phải chặn cú
    #    nhấp đầu. Đây là ca người dùng đang giữ chuột lúc ký tự cuối vừa khớp.
    Bam 123 187          # bấm vào ô nhập để nó nhận phím
    # Trước khi gõ đúng, thử gõ CHỮ THƯỜNG — TV-01 đòi nó không được mở khóa.
    foreach ($k in 0x58, 0x4F, 0x41) { Go-Phim $k }
    $g = Goc-Ve
    [void][MS.I]::SetCursorPos($g.X + 145, $g.Y + 220)
    [MS.I]::mouse_event(0x02, 0, 0, 0, 0); Start-Sleep -Milliseconds 10
    [MS.I]::mouse_event(0x04, 0, 0, 0, 0); Start-Sleep -Seconds 2
    Assert 'TV-01 · gõ "xoa" CHỮ THƯỜNG thì nút không mở khóa' `
        ((Dem-Tep $sb) -eq $truoc) "còn $(Dem-Tep $sb)/$truoc"

    # Xóa sạch ô nhập rồi gõ lại đúng dạng CHỮ HOA.
    Bam 123 187
    for ($i = 0; $i -lt 6; $i++) { Go-Phim 0x08 }     # BACKSPACE
    foreach ($k in 0x58, 0x4F, 0x41) { Go-Chu-Hoa $k }   # X O A — TV-01 nhận dạng không dấu
    $g = Goc-Ve
    [void][MS.I]::SetCursorPos($g.X + 145, $g.Y + 220)
    [MS.I]::mouse_event(0x02, 0, 0, 0, 0); Start-Sleep -Milliseconds 10
    [MS.I]::mouse_event(0x04, 0, 0, 0, 0)
    Start-Sleep -Seconds 2
    Assert '④ gõ đúng cụm từ rồi nhấp NGAY — khóa mồi 600 ms chặn được' `
        ((Dem-Tep $sb) -eq $truoc) "còn $(Dem-Tep $sb)/$truoc"

    # Kiểm ngược: chờ hết khóa mồi rồi nhấp bình thường thì PHẢI xóa được thật.
    #
    # Không có vế này thì "0 tệp biến mất" có thể chỉ vì cú nhấp của ta trượt,
    # hoặc vì cả đường xóa đã hỏng — và bốn phép thử trên thành vô nghĩa. Một
    # phép thử an toàn luôn xanh là một phép thử chưa chứng minh được gì.
    Start-Sleep -Seconds 1
    Bam 145 220
    Start-Sleep -Seconds 5
    $sau = Dem-Tep $sb
    Assert 'Kiểm ngược: chờ hết khóa mồi rồi nhấp thì XÓA ĐƯỢC THẬT' ($sau -lt $truoc) `
        "vẫn còn $sau/$truoc — cú nhấp trượt hoặc đường xóa hỏng, bốn phép thử trên thành vô nghĩa"
} finally {
    if ($proc -and -not $proc.HasExited) { Stop-Process -Id $proc.Id -Force -EA SilentlyContinue }
    Start-Sleep -Milliseconds 500
    if (Test-Path $sb) { Remove-Item $sb -Recurse -Force -EA SilentlyContinue }
    Get-ChildItem (Join-Path (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent) 'logs') `
        -Filter 'daxoa_*.log' -File -EA SilentlyContinue |
        Where-Object { $_.LastWriteTime -gt (Get-Date).AddMinutes(-10) } |
        Remove-Item -Force -EA SilentlyContinue
}

Write-Host ''
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
Write-Host ("  ĐẠT: {0}    HỎNG: {1}" -f $script:Pass, $script:Fail) `
    -ForegroundColor $(if ($script:Fail -eq 0) { 'Green' } else { 'Red' })
Write-Host '════════════════════════════════════════════════════════════════' -ForegroundColor Cyan
if ($script:Fail -gt 0) { Write-Host '  §8.1-1 CHƯA ĐẠT' -ForegroundColor Red; exit 1 }
Write-Host '  §8.1-1 ĐẠT' -ForegroundColor Green
exit 0
