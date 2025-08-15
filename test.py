# monitors.py
import json
import shutil
import subprocess
import sys

PS_SCRIPT = r"""
$toStr = {
  param([UInt16[]]$arr)
  if (-not $arr) { return $null }
  ($arr | ForEach-Object { if ($_ -gt 0 -and $_ -lt 256) { [char]$_ } }) -join ''
}

$ids = Get-CimInstance -Namespace root\wmi -Class WmiMonitorID -ErrorAction SilentlyContinue

$basic = @{}
Get-CimInstance -Namespace root\wmi -Class WmiMonitorBasicDisplayParams -ErrorAction SilentlyContinue | ForEach-Object {
  $basic[$_.InstanceName] = $_
}

$conn = @{}
Get-CimInstance -Namespace root\wmi -Class WmiMonitorConnectionParams -ErrorAction SilentlyContinue | ForEach-Object {
  $conn[$_.InstanceName] = $_
}

$results = @()
foreach ($m in $ids) {
  $inst = $m.InstanceName
  $b = $basic[$inst]
  $c = $conn[$inst]
  $results += [pscustomobject]@{
    InstanceName           = $inst
    Manufacturer           = (& $toStr $m.ManufacturerName)
    Model                  = (& $toStr $m.UserFriendlyName)
    SerialNumber           = (& $toStr $m.SerialNumberID)
    ProductCodeId          = (& $toStr $m.ProductCodeID)
    WeekOfManufacture      = $m.WeekOfManufacture
    YearOfManufacture      = $m.YearOfManufacture
    MaxImageSizeCm         = if ($b) { @($b.MaxHorizontalImageSize, $b.MaxVerticalImageSize) } else { $null }
    VideoOutputTechnology  = if ($c) { [uint32]$c.VideoOutputTechnology } else { $null }
    ConnectionActive       = if ($c) { [bool]$c.Active } else { $null }
  }
}

$results | ConvertTo-Json -Depth 4
"""


def run_powershell(script: str) -> str:
    for exe in ("pwsh", "powershell"):
        if shutil.which(exe):
            proc = subprocess.run(
                [exe, "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script],
                capture_output=True,
                text=True,
            )
            if proc.returncode == 0 and proc.stdout.strip():
                return proc.stdout
    proc = subprocess.run(
        [
            r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "PowerShell command failed")
    return proc.stdout


# Map D3DKMDT/DISPLAYCONFIG codes to friendly names
# (see Microsoft docs)
VIDEO_TECH = {
    -2: "Uninitialized",
    -1: "Other",
    0: "VGA",
    1: "S-Video",
    2: "Composite",
    3: "Component",
    4: "DVI",
    5: "HDMI",
    6: "LVDS / MIPI-DSI",
    8: "D-Jpn",
    9: "SDI",
    10: "DisplayPort (external)",
    11: "DisplayPort (embedded)",
    12: "UDI (external)",
    13: "UDI (embedded)",
    14: "SDTV dongle",
    15: "Miracast (wireless)",
    16: "Indirect (wired)",
    2147483648: "Internal (adapter)",
}


def is_builtin_from_video(code: int | None) -> bool | None:
    if code is None:
        return None
    # Embedded / LVDS / explicit INTERNAL are built-in
    return code in (6, 11, 13, 2147483648)


def get_monitors():
    raw = run_powershell(PS_SCRIPT)
    data = json.loads(raw)
    if isinstance(data, dict):
        data = [data]
    out = []
    for m in data or []:
        code = m.get("VideoOutputTechnology")
        out.append(
            {
                "manufacturer": m.get("Manufacturer") or "Unknown",
                "model": m.get("Model") or "Unknown",
                "serial": m.get("SerialNumber"),
                "product_code": m.get("ProductCodeId"),
                "manufacture_week": m.get("WeekOfManufacture"),
                "manufacture_year": m.get("YearOfManufacture"),
                "instance": m.get("InstanceName"),
                "connection": VIDEO_TECH.get(
                    code, f"Unknown ({code})" if code is not None else None
                ),
                "built_in": is_builtin_from_video(code),
                "active": m.get("ConnectionActive"),
            }
        )
    return out


def main():
    monitors = get_monitors()
    if "--json" in sys.argv:
        print(json.dumps(monitors, ensure_ascii=False, indent=2))
        return
    print(f"Found {len(monitors)} monitor(s):")
    for i, m in enumerate(monitors, 1):
        print(m)
        line = f"{i}. {m['model']} — {m['manufacturer']}"
        extras = []
        extras.append(
            "Built-in"
            if m["built_in"]
            else "External" if m["built_in"] is not None else "Built-in: unknown"
        )
        if m.get("connection"):
            extras.append(m["connection"])
        if m.get("serial"):
            extras.append(f"S/N: {m['serial']}")
        if m.get("manufacture_year"):
            y = m["manufacture_year"]
            w = m.get("manufacture_week")
            extras.append(f"Made: {y}" + (f" (week {w})" if w else ""))
        print(line + " | " + " | ".join(extras))


if __name__ == "__main__":
    main()
