$logoDir = "C:\Users\dotSlash\Git\ModernRetroRomManager\src-tauri\resources\logo"

# 映射表：当前文件名 -> 目标文件名（基于CSV命名）
$mapping = @{
    # Nintendo 系列
    "FC.png" = "Nintendo - Nintendo Entertainment System.png"
    "FC-HD.png" = "Nintendo - Nintendo Entertainment System.png"  # 合并到FC
    "SFC.png" = "Nintendo - Super Nintendo Entertainment System.png"
    "SFC-MSU1.png" = "Nintendo - Super Nintendo Entertainment System.png"  # 合并到SFC
    "SFC-CD.png" = "Nintendo - Super Nintendo Entertainment System.png"  # 合并到SFC
    "GB.png" = "Nintendo - Game Boy.png"
    "GBC.png" = "Nintendo - Game Boy Color.png"
    "GBA.png" = "Nintendo - Game Boy Advance.png"
    "N64.png" = "Nintendo - Nintendo 64.png"
    "NGC.png" = "Nintendo - GameCube.png"
    "NDS.png" = "Nintendo - Nintendo DS.png"
    "3DS.png" = "Nintendo - Nintendo 3DS.png"
    "WII.png" = "Nintendo - Wii.png"
    "WII Ware.png" = "Nintendo - Wii.png"  # 合并到Wii
    "VB.png" = "Nintendo - Virtual Boy.png"
    "POKE MINI.png" = "Nintendo - Pokemon Mini.png"
    "GAME WATCH.png" = "Nintendo - Game & Watch.png"

    # Sega 系列
    "MD.png" = "Sega - Mega Drive - Genesis.png"
    "MD-32X.png" = "Sega - 32X.png"
    "SEGA-32X.png" = "Sega - 32X.png"
    "MD-CD.png" = "Sega - Mega CD & Sega CD.png"
    "SEGA-CD.png" = "Sega - Mega CD & Sega CD.png"
    "DC.png" = "Sega - Dreamcast.png"
    "SS.png" = "Sega - Saturn.png"
    "GG.png" = "Sega - Game Gear.png"
    "SMS.png" = "Sega - Master System.png"
    "SEGA-MS.png" = "Sega - Master System.png"

    # Sony 系列
    "PS.png" = "Sony - PlayStation.png"
    "PS1.png" = "Sony - PlayStation.png"
    "PSP.png" = "Sony - PlayStation Portable.png"
    "PS2.png" = "Sony - PlayStation 2.png"
    "PS3.png" = "Sony - PlayStation 3.png"

    # Bandai 系列
    "WS.png" = "Bandai - WonderSwan.png"
    "WSC.png" = "Bandai - WonderSwan Color.png"

    # SNK 系列
    "NGPC.png" = "SNK - Neo Geo Pocket Color.png"

    # NEC 系列
    "PCE.png" = "NEC - PC Engine - TurboGrafx-16.png"
    "PCE-CD.png" = "NEC - PC Engine - TurboGrafx-16.png"  # 合并

    # Panasonic
    "3DO.png" = "Panasonic - 3DO Interactive Multiplayer.png"

    # Atari 系列
    "LYNX.png" = "Atari - Lynx.png"
    "ATARI.png" = "Atari - Atari 2600.png"

    # Arcade
    "NEOGEO-CD.png" = "Arcade - NEOGEO.png"
    "NAOMI.png" = "Arcade - NAOMI.png"
    "MODEL2.png" = "Arcade - MODEL2.png"
    "MODEL3.png" = "Arcade - MODEL3.png"

    # Microsoft
    "PC.png" = "Microsoft - PC.png"
    "DOS.png" = "Microsoft - DOS.png"

    # 其他
    "OPENBOR.png" = "OpenBOR.png"
    "PC-FX.png" = "NEC - PC-FX.png"
    "SWITCH.png" = "Nintendo - Switch.png"
    "TeknoParrot.png" = "Arcade - TeknoParrot.png"
}

Write-Host "开始处理 logo 文件..."
Write-Host ""

# 第一步：删除有数字前缀的重复文件
Write-Host "=== 第一步：删除数字前缀的重复文件 ==="
$files = Get-ChildItem $logoDir -Filter "*.png"
foreach ($file in $files) {
    if ($file.Name -match "^\d+\s+(.+)$") {
        $baseName = $matches[1]
        $nonNumbered = Join-Path $logoDir $baseName
        if (Test-Path $nonNumbered) {
            Write-Host "删除重复: $($file.Name)"
            Remove-Item $file.FullName -Force
        } else {
            # 如果没有无前缀版本，重命名去掉前缀
            Write-Host "重命名: $($file.Name) -> $baseName"
            Rename-Item $file.FullName -NewName $baseName
        }
    }
}

Write-Host ""
Write-Host "=== 第二步：删除 hack 相关文件 ==="
$hackFiles = Get-ChildItem $logoDir -Filter "*hack*.png"
foreach ($file in $hackFiles) {
    Write-Host "删除 hack 文件: $($file.Name)"
    Remove-Item $file.FullName -Force
}

Write-Host ""
Write-Host "=== 第三步：按映射表重命名 ==="
foreach ($oldName in $mapping.Keys) {
    $oldPath = Join-Path $logoDir $oldName
    $newName = $mapping[$oldName]
    $newPath = Join-Path $logoDir $newName

    if (Test-Path $oldPath) {
        if (Test-Path $newPath) {
            Write-Host "目标已存在，删除源文件: $oldName"
            Remove-Item $oldPath -Force
        } else {
            Write-Host "重命名: $oldName -> $newName"
            Rename-Item $oldPath -NewName $newName
        }
    }
}

Write-Host ""
Write-Host "完成！"
