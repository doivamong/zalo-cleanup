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
    [Parameter(Mandatory)][ValidateSet('rules', 'protect', 'root', 'tones')]
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
                  'Build-ProtectedIndex', 'Initialize-ProtectedAbs')) {
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
    'tones' {
        while ($null -ne ($line = [Console]::In.ReadLine())) {
            [Console]::Out.WriteLine((Remove-ToneMarks $line))
        }
    }
}
