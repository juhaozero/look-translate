; Look Translate NSIS installer hooks.
; https://v2.tauri.app/distribute/windows-installer/

; Ask whether to keep a backup of the portable `data/` folder before uninstall
; removes the install directory.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} ${FileExists} "$INSTDIR\data"
    MessageBox MB_YESNO|MB_ICONQUESTION \
      "是否删除配置目录 data/？$\r$\n$\r$\n路径：$INSTDIR\data$\r$\n$\r$\n选「是」：配置与 API Key 随卸载删除。$\r$\n选「否」：先备份到 %LOCALAPPDATA%\Look Translate\data，再继续卸载。" \
      IDYES skip_data_backup IDNO backup_data

    backup_data:
      CreateDirectory "$LOCALAPPDATA\Look Translate"
      ; Stage into data.tmp so an existing backup is only replaced after a successful copy.
      RMDir /r "$LOCALAPPDATA\Look Translate\data.tmp"
      CreateDirectory "$LOCALAPPDATA\Look Translate\data.tmp"
      ClearErrors
      CopyFiles /SILENT "$INSTDIR\data\*.*" "$LOCALAPPDATA\Look Translate\data.tmp"
      IfErrors backup_failed
      RMDir /r "$LOCALAPPDATA\Look Translate\data"
      Rename "$LOCALAPPDATA\Look Translate\data.tmp" "$LOCALAPPDATA\Look Translate\data"
      MessageBox MB_OK|MB_ICONINFORMATION \
        "已备份配置到：$\r$\n$LOCALAPPDATA\Look Translate\data$\r$\n$\r$\n重装后可将该目录复制回安装目录旁的 data\。"
      Goto done_data_prompt

    backup_failed:
      RMDir /r "$LOCALAPPDATA\Look Translate\data.tmp"
      MessageBox MB_OK|MB_ICONEXCLAMATION \
        "备份失败，未改动已有备份（若存在）。$\r$\n请手动复制：$INSTDIR\data"
      Goto done_data_prompt

    skip_data_backup:
      DetailPrint "User chose to delete data/ with uninstall"

    done_data_prompt:
  ${EndIf}
!macroend
