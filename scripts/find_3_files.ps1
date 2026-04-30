# 搜索3个未修复的文件

$base = "D:\samba\psp"

$targets = @(
    "伊苏vs.空之轨迹 抉择传奇[v1.2][简][FFC汉化组].iso",
    "街～命运的交叉点～[V3汉化修复版20220505][简][施珂昱].iso",
    "魔法少女奈叶A's 王牌之战 携带版[繁][v1.21].iso"
)

foreach ($target in $targets) {
    Write-Host "Searching: $target" -ForegroundColor Yellow
    
    $found = $false
    Get-ChildItem -Path $base -Directory | ForEach-Object {
        $subfolder = $_
        Get-ChildItem -Path $subfolder.FullName -File -ErrorAction SilentlyContinue | Where-Object {
            $_.Name -eq $target
        } | ForEach-Object {
            Write-Host "  Found: $($subfolder.Name)/$($_.Name)" -ForegroundColor Green
            $found = $true
        }
    }
    
    if (-not $found) {
        Write-Host "  NOT FOUND" -ForegroundColor Red
    }
    Write-Host ""
}
