## CLI Usage

WinDisplay can be used from the command line for automation and scripting. When no command is provided, the GUI application starts.

### List Monitors

Display information about all connected monitors:

```bash
WinDisplay.exe list
```

All commands use `--monitor-idx` to specify which monitor to control (0-based index). Use the `list` command to see all monitors and their indices.

### Input Source Control

Switch monitor input source (requires DDC/CI support):

```bash
# Set input source
WinDisplay.exe set-input --monitor-idx 0 --source hdmi1

# Get current input source
WinDisplay.exe get-input --monitor-idx 0
```

Supported input sources: `vga1`, `vga2`, `dvi1`, `dvi2`, `dp1`, `dp2`, `hdmi1`, `hdmi2`, `hdmi3`, `usbc1`, `usbc2`, `usbc3`, or custom hex codes like `0x11` (DDC/CI).

### Resolution Control

Change monitor resolution and refresh rate:

```bash
# Set resolution with specific refresh rate
WinDisplay.exe set-resolution --monitor-idx 0 --width 1920 --height 1080 --refresh-hz 144

# Set resolution (uses default refresh rate)
WinDisplay.exe set-resolution --monitor-idx 0 --width 2560 --height 1440
```

### Brightness Control

Adjust monitor brightness (requires DDC/CI support):

```bash
# Set brightness
WinDisplay.exe set-brightness --monitor-idx 0 --percent 75

# Get current brightness
WinDisplay.exe get-brightness --monitor-idx 0
```

### Orientation Control

Rotate monitor display:

```bash
WinDisplay.exe set-orientation --monitor-idx 0 --degrees 90
```

Valid values: `0`, `90`, `180`, `270` degrees.

### Scale Control

Change display scaling without logging out:

```bash
WinDisplay.exe set-scale --monitor-idx 0 --percent 150
```

Common scale values: `100`, `125`, `150`, `175`, `200`.

> [!WARNING]
> For now, scale value in CLI might be incorrect, use one from UI.

### HDR Control

Enable or disable HDR mode:

```bash
# Enable HDR
WinDisplay.exe set-hdr --monitor-idx 0 --enable true

# Disable HDR
WinDisplay.exe set-hdr --monitor-idx 0 --enable false
```

