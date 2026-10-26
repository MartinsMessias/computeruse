# Comprehensive demo of all computeruse functions
$ErrorActionPreference = "Continue"
$cu = "D:\Documents\Default Project\computeruse\target\release\cu.exe"
function C { & $cu @args 2>&1 | Out-Null }
function CJ { & $cu @args 2>&1 }

Write-Host "=== DEMO: ALL FUNCTIONS ===" -ForegroundColor Cyan

# 1. Screen info
Write-Host "`n[1] screen-size" -ForegroundColor Yellow
CJ screen-size
Write-Host "[2] monitors" -ForegroundColor Yellow
CJ monitors

# 2. Screenshot
Write-Host "`n[3] screenshot (full)" -ForegroundColor Yellow
$r = CJ screenshot | ConvertFrom-Json
[System.IO.File]::WriteAllBytes("demo_full.png", [System.Convert]::FromBase64String($r.png_base64))
Write-Host "saved demo_full.png $($r.width)x$($r.height)"

Write-Host "`n[4] screenshot-all" -ForegroundColor Yellow
$r = CJ screenshot-all | ConvertFrom-Json
[System.IO.File]::WriteAllBytes("demo_all.png", [System.Convert]::FromBase64String($r.png_base64))
Write-Host "saved demo_all.png $($r.width)x$($r.height)"

# 3. Cursor
Write-Host "`n[5] cursor" -ForegroundColor Yellow
CJ cursor
Write-Host "[6] move 500 500" -ForegroundColor Yellow
C move 500 500
Write-Host "[7] move-rel 100 50" -ForegroundColor Yellow
C move-rel 100 50
Write-Host "[8] cursor (after move)" -ForegroundColor Yellow
CJ cursor

# 4. Pixel
Write-Host "`n[9] pixel 500 500" -ForegroundColor Yellow
CJ pixel 500 500

# 5. Launch + window management
Write-Host "`n[10] launch notepad" -ForegroundColor Yellow
C launch notepad.exe
Start-Sleep -Seconds 2
$h = (CJ find-window "Notepad" | ConvertFrom-Json).hwnd
Write-Host "notepad hwnd=$h"
Write-Host "[11] foreground" -ForegroundColor Yellow
CJ foreground | Select-Object -First 3
Write-Host "[12] focus --hwnd" -ForegroundColor Yellow
C focus --hwnd $h
Write-Host "[13] windows (visible)" -ForegroundColor Yellow
(CJ windows | ConvertFrom-Json).windows | Select-Object -First 3 title
Write-Host "[14] find-window" -ForegroundColor Yellow
CJ find-window "Notepad" | Select-Object -First 4

# 6. Screenshot window
Write-Host "`n[15] screenshot-window" -ForegroundColor Yellow
$r = CJ screenshot-window $h | ConvertFrom-Json
[System.IO.File]::WriteAllBytes("demo_win.png", [System.Convert]::FromBase64String($r.png_base64))
Write-Host "saved demo_win.png $($r.width)x$($r.height)"
Write-Host "[16] screenshot-active" -ForegroundColor Yellow
$r = CJ screenshot-active | ConvertFrom-Json
[System.IO.File]::WriteAllBytes("demo_active.png", [System.Convert]::FromBase64String($r.png_base64))
Write-Host "saved demo_active.png $($r.width)x$($r.height)"

# 7. Window ops
Write-Host "`n[17] minimize" -ForegroundColor Yellow
C minimize $h
Start-Sleep -Milliseconds 500
Write-Host "[18] restore" -ForegroundColor Yellow
C restore $h
Start-Sleep -Milliseconds 500
Write-Host "[19] maximize" -ForegroundColor Yellow
C maximize $h
Start-Sleep -Milliseconds 500
Write-Host "[20] restore" -ForegroundColor Yellow
C restore $h
Start-Sleep -Milliseconds 500
Write-Host "[21] move-window" -ForegroundColor Yellow
C move-window $h 200 200 700 500
Start-Sleep -Milliseconds 300
CJ find-window "Notepad" | Select-String "rect" -Context 0,4

# 8. Mouse actions
Write-Host "`n[22] click left 400 400" -ForegroundColor Yellow
C click left --x 400 --y 400
Write-Host "[23] double-click" -ForegroundColor Yellow
C dbl-click left --x 400 --y 400
Write-Host "[24] mouse-down/mouse-up" -ForegroundColor Yellow
C mouse-down left
Start-Sleep -Milliseconds 100
C mouse-up left
Write-Host "[25] drag 300 300 450 450" -ForegroundColor Yellow
C drag 300 300 450 450 left
Write-Host "[26] scroll 3" -ForegroundColor Yellow
C scroll 3

# 9. Keyboard
Write-Host "`n[27] type" -ForegroundColor Yellow
C type "Demo completo da lib computeruse!"
Write-Host "[28] key enter" -ForegroundColor Yellow
C key enter
Write-Host "[29] type (line 2)" -ForegroundColor Yellow
C type "Todas as funcoes demonstradas."
Write-Host "[30] hotkey ctrl+a" -ForegroundColor Yellow
C hotkey "ctrl+a"
Write-Host "[31] key delete" -ForegroundColor Yellow
C key delete
Start-Sleep -Milliseconds 200

# 10. Clipboard
Write-Host "`n[32] clipboard-set" -ForegroundColor Yellow
C clipboard-set "Texto copiado pela lib!"
Write-Host "[33] clipboard-get" -ForegroundColor Yellow
CJ clipboard-get

# 11. Wait
Write-Host "`n[34] wait 500" -ForegroundColor Yellow
C wait 500
Write-Host "[35] wait-for-window" -ForegroundColor Yellow
CJ wait-for-window "Notepad" --timeout 2000 | Select-Object -First 2

# 12. UI Automation
Write-Host "`n[36] uia-focused" -ForegroundColor Yellow
CJ uia-focused | Select-Object -First 6
Write-Host "[37] uia-at 400 400" -ForegroundColor Yellow
CJ uia-at 400 400 | Select-Object -First 6
Write-Host "[38] uia-children (hwnd 0 = root, first 3)" -ForegroundColor Yellow
(CJ uia-children --hwnd 0 | ConvertFrom-Json).children | Select-Object -First 3 name, control_type
Write-Host "[39] uia-find 'Notepad'" -ForegroundColor Yellow
(CJ uia-find "Notepad" | ConvertFrom-Json).elements | Select-Object -First 2 name, control_type

# 13. Process management
Write-Host "`n[40] processes (first 3)" -ForegroundColor Yellow
(CJ processes | ConvertFrom-Json).processes | Select-Object -First 3 exe_name, pid
Write-Host "[41] find-process notepad" -ForegroundColor Yellow
CJ find-process "notepad" | Select-Object -First 4

# 14. Batch exec
Write-Host "`n[42] exec (batch)" -ForegroundColor Yellow
$batch = '[{"action":"screen_size"},{"action":"cursor"},{"action":"pixel","x":100,"y":100},{"action":"wait","ms":100}]'
$batch | & $cu exec 2>&1 | Select-Object -First 8

# 15. Cleanup
Write-Host "`n[43] close notepad" -ForegroundColor Yellow
C close $h
Write-Host "=== DEMO COMPLETE ===" -ForegroundColor Cyan
