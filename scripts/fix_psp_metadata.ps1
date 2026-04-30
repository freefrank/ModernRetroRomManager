# Fix PSP metadata.pegasus.txt - Add folder paths to file: fields
# Usage: powershell -ExecutionPolicy Bypass -File fix_psp_metadata.ps1

$pspDir = "D:\samba\psp"
$metadataFile = Join-Path $pspDir "metadata.pegasus.txt"
$outputFile = Join-Path $pspDir "metadata.pegasus.fixed.txt"

# Step 1: Build filename -> relative path mapping (one-time scan)
Write-Host "Building file index (this may take a moment)..."
$fileIndex = @{}

Get-ChildItem -Path $pspDir -Directory | ForEach-Object {
    $folderName = $_.Name
    Get-ChildItem -Path $_.FullName -File -ErrorAction SilentlyContinue | Where-Object {
        $_.Extension -match '\.(iso|cso|pbp|ISO|CSO|PBP)$'
    } | ForEach-Object {
        $relativePath = "$folderName/$($_.Name)"
        $fileIndex[$_.Name] = $relativePath
    }
}

Write-Host "Indexed $($fileIndex.Count) ROM files"

# Step 2: Process metadata file
Write-Host "Processing metadata..."
$content = Get-Content -Path $metadataFile -Encoding UTF8
$output = New-Object System.Collections.ArrayList

$fixedCount = 0
$notFoundCount = 0

foreach ($line in $content) {
    if ($line -match '^file:\s*(.+)$') {
        $filename = $Matches[1].Trim()
        
        # Check if already has a path
        if ($filename -match '[/\\]') {
            [void]$output.Add($line)
        }
        else {
            # Look up in index
            if ($fileIndex.ContainsKey($filename)) {
                $newPath = $fileIndex[$filename]
                [void]$output.Add("file: $newPath")
                $fixedCount++
            }
            else {
                # Not found, keep original
                [void]$output.Add($line)
                $notFoundCount++
                Write-Host "Not found: $filename"
            }
        }
    }
    else {
        [void]$output.Add($line)
    }
}

# Step 3: Write output
Write-Host "`nWriting to $outputFile..."
$output -join "`n" | Set-Content -Path $outputFile -Encoding UTF8 -NoNewline

Write-Host "`nDone!"
Write-Host "Fixed: $fixedCount entries"
Write-Host "Not found: $notFoundCount entries"
Write-Host "`nPlease review $outputFile and rename to metadata.pegasus.txt if correct."
