#Requires -Version 5.1
<#
    BẢN THẦN CHÚ cho bộ đối chiếu song song của mốc M1.

    Bóc thẳng các hàm an toàn ra khỏi ZaloCleanup.ps1 bằng AST — đúng cách mà
    ZaloCleanup.Tests.ps1 vẫn làm — rồi trả lời từng đường dẫn đọc từ stdin.
    Không chép lại logic vào đây: chép là hai bản trôi khỏi nhau, mà cả điểm của
    bộ đối chiếu là bắt trôi.

    Chế độ:
      -Mode rules    In ra bộ luật vùng bảo vệ đang dùng, mỗi dòng "muc<TAB>đường dẫn".
                     Bản Rust nạp đúng bộ này để hai bên xuất phát từ cùng đầu vào.
      -Mode protect  Đọc từng dòng ở stdin, in "1" nếu bị chặn, "0" nếu không.
      -Mode root     Như trên nhưng hỏi Test-ProtectedRoot.
      -Mode tones    Đọc từng dòng, in ra chuỗi đã bỏ dấu thanh.

    -DataRoot đặt thư mục dữ liệu Zalo giả định, để hai bên cùng một điều kiện.
#>
param(
    [Parameter(Mandatory)][ValidateSet('rules', 'protect', 'root', 'tones', 'walk', 'hash', 'quicksig', 'ext')]
    [string]$Mode,
    [string]$DataRoot = '',
    [string]$ToolDir = ''
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$OutputEncoding = [Text.Encoding]::UTF8

$tool = Join-Path (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent) 'ZaloCleanup.ps1'
if (-not (Test-Path -LiteralPath $tool)) { throw "không thấy $tool" }

$ast = [System.Management.Automation.Language.Parser]::ParseFile($tool, [ref]$null, [ref]$null)
foreach ($fn in @('Get-CanonPath', 'Remove-ToneMarks', 'Test-Protected', 'Test-ProtectedRoot',
                  'Build-ProtectedIndex', 'Initialize-ProtectedAbs',
                  'Get-FilesSafe', 'Get-Sha256Full', 'Get-QuickSig')) {
    $node = $ast.Find({ param($x)
        $x -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $x.Name -eq $fn }, $true)
    if ($null -eq $node) { throw "không bóc được hàm $fn" }
    Invoke-Expression $node.Extent.Text
}

# Dựng đúng trạng thái mà công cụ thật dùng.
$script:ProtectedNames = @('Database', 'Partitions')
$script:SysDrive = $env:SystemDrive
$script:SysRoot  = $env:SystemDrive + '\'
$script:ToolDir  = if ($ToolDir -ne '') { $ToolDir } else { Split-Path $tool -Parent }
$script:DataRoot = $DataRoot
$script:ProtectedExact = $null
Initialize-ProtectedAbs

switch ($Mode) {
    'rules' {
        foreach ($r in $script:ProtectedRules) {
            $m = if ($r.Depth -eq 'any') { 'tatca' } else { 'goc' }
            [Console]::Out.WriteLine($m + "`t" + $r.Path)
        }
    }
    'protect' {
        while ($null -ne ($line = [Console]::In.ReadLine())) {
            [Console]::Out.WriteLine($(if (Test-Protected $line) { '1' } else { '0' }))
        }
    }
    'root' {
        while ($null -ne ($line = [Console]::In.ReadLine())) {
            [Console]::Out.WriteLine($(if (Test-ProtectedRoot $line) { '1' } else { '0' }))
        }
    }
    'walk' {
        # Duyệt một cây và in ra "<số lỗi>" rồi từng đường dẫn tệp, mỗi dòng một.
        # Dùng CHÍNH Get-FilesSafe của công cụ, không chép lại.
        $goc = [Console]::In.ReadLine()
        $script:LastScanErrors = 0
        $tep = @(Get-FilesSafe $goc)
        [Console]::Out.WriteLine($script:LastScanErrors)
        foreach ($f in $tep) { [Console]::Out.WriteLine($f.FullName) }
    }
    'hash' {
        $sha = [Security.Cryptography.SHA256]::Create()
        try {
            while ($null -ne ($line = [Console]::In.ReadLine())) {
                try { [Console]::Out.WriteLine((Get-Sha256Full $line $sha)) }
                catch { [Console]::Out.WriteLine('LỖI') }
            }
        } finally { $sha.Dispose() }
    }
    'quicksig' {
        $sha = [Security.Cryptography.SHA256]::Create()
        try {
            while ($null -ne ($line = [Console]::In.ReadLine())) {
                try {
                    $co = (New-Object IO.FileInfo $line).Length
                    [Console]::Out.WriteLine((Get-QuickSig $line $co $sha))
                } catch { [Console]::Out.WriteLine('LỖI') }
            }
        } finally { $sha.Dispose() }
    }
    'ext' {
        # Phần mở rộng theo đúng .NET, để đối chiếu với luật đã cài bên Rust.
        while ($null -ne ($line = [Console]::In.ReadLine())) {
            [Console]::Out.WriteLine('[' + [IO.Path]::GetExtension($line) + ']')
        }
    }
    'tones' {
        while ($null -ne ($line = [Console]::In.ReadLine())) {
            [Console]::Out.WriteLine((Remove-ToneMarks $line))
        }
    }
}
