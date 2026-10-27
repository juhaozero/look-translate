; Portable user data belongs next to the executable, independently of the app.
; Tauri deletes installed files individually and uses a non-recursive RMDir
; for $INSTDIR, so an existing data/ folder keeps the directory in place.
; Never package data/ as an installer resource or recursively remove $INSTDIR.

!macro NSIS_HOOK_PREINSTALL
  ${If} ${FileExists} "$INSTDIR\data"
    DetailPrint "保留并沿用现有数据目录：$INSTDIR\data"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ${If} ${FileExists} "$INSTDIR\data"
    DetailPrint "保留数据目录：$INSTDIR\data（重装到同一目录即可沿用）"
  ${EndIf}
!macroend
