# Minimal spec; CI uses the CLI but this can be used locally if preferred
block_cipher = None

a = Analysis([
    'windisplay/__main__.py',
],
             pathex=[],
             binaries=[],
             datas=[('windisplay/assets/app.ico', 'windisplay/assets')],
             hiddenimports=[],
             hookspath=[],
             hooksconfig={},
             runtime_hooks=[],
             excludes=[],
             win_no_prefer_redirects=False,
             win_private_assemblies=False,
             cipher=block_cipher,
             noarchive=False)
pyz = PYZ(a.pure, a.zipped_data,
             cipher=block_cipher)
exe = EXE(pyz,
          a.scripts,
          [],
          exclude_binaries=True,
          name='WinDisplay',
          debug=False,
          bootloader_ignore_signals=False,
          strip=False,
           upx=False,
          console=False,
          disable_windowed_traceback=False,
          target_arch=None,
          codesign_identity=None,
          entitlements_file=None,
          icon='windisplay/assets/app.ico')
coll = COLLECT(exe,
               a.binaries,
               a.zipfiles,
               a.datas,
               strip=False,
               upx=False,
               upx_exclude=[],
               name='WinDisplay')


