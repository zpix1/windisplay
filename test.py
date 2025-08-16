# -*- coding: utf-8 -*-
# Requires: Python 3.9+ on Windows
# Usage examples:
#   python dpi_scale.py list
#   python dpi_scale.py set 125             # main monitor, Way 1 (default)
#   python dpi_scale.py set 150 --monitor 2 # set 2nd monitor, Way 1
#   python dpi_scale.py set 125 --method registry   # Way 2 (global) + Explorer restart
#
# Undocumented per-monitor method inspired by:
#   - https://github.com/lihas/windows-DPI-scaling-sample
#   - https://github.com/imniko/SetDPI
#
import argparse
import ctypes
import subprocess
import sys
import time
from ctypes import wintypes
import winreg

# --------------------------
# Common utilities
# --------------------------

DPI_VALS = (100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500)


def err(msg, code=1):
    print(f"[!] {msg}", file=sys.stderr)
    sys.exit(code)


def info(msg):
    print(f"[*] {msg}")


# --------------------------
# Way 1: Undocumented per-monitor (ctypes + DisplayConfig*)
# --------------------------

user32 = ctypes.WinDLL("user32", use_last_error=True)

QDC_ONLY_ACTIVE_PATHS = 0x00000002


# Basic Win32 types
class LUID(ctypes.Structure):
    _fields_ = [
        ("LowPart", wintypes.DWORD),
        ("HighPart", wintypes.LONG),
    ]


class DISPLAYCONFIG_DEVICE_INFO_HEADER(ctypes.Structure):
    _fields_ = [
        ("type", wintypes.INT),  # negative for our hidden types
        ("size", wintypes.DWORD),
        ("adapterId", LUID),
        ("id", wintypes.DWORD),  # sourceId
    ]


# Hidden device info types (reverse-engineered)
DISPLAYCONFIG_DEVICE_INFO_GET_DPI_SCALE = -3
DISPLAYCONFIG_DEVICE_INFO_SET_DPI_SCALE = -4


# GET: returns min/cur/max relative to recommended
class DISPLAYCONFIG_SOURCE_DPI_SCALE_GET(ctypes.Structure):
    _fields_ = [
        ("header", DISPLAYCONFIG_DEVICE_INFO_HEADER),
        ("minScaleRel", wintypes.INT),
        ("curScaleRel", wintypes.INT),
        ("maxScaleRel", wintypes.INT),
    ]


assert (
    ctypes.sizeof(DISPLAYCONFIG_SOURCE_DPI_SCALE_GET) == 0x20
), "Struct size mismatch GET (expected 0x20)"


# SET: takes desired relative value to recommended
class DISPLAYCONFIG_SOURCE_DPI_SCALE_SET(ctypes.Structure):
    _fields_ = [
        ("header", DISPLAYCONFIG_DEVICE_INFO_HEADER),
        ("scaleRel", wintypes.INT),
    ]


assert (
    ctypes.sizeof(DISPLAYCONFIG_SOURCE_DPI_SCALE_SET) == 0x18
), "Struct size mismatch SET (expected 0x18)"


# Structures for QueryDisplayConfig (we only need adapterId + sourceId)
class DISPLAYCONFIG_RATIONAL(ctypes.Structure):
    _fields_ = [("Numerator", wintypes.UINT), ("Denominator", wintypes.UINT)]


class DISPLAYCONFIG_PATH_SOURCE_INFO(ctypes.Structure):
    _fields_ = [
        ("adapterId", LUID),
        ("id", wintypes.UINT),
        ("modeInfoIdx", wintypes.UINT),
        ("statusFlags", wintypes.UINT),
    ]


class DISPLAYCONFIG_PATH_TARGET_INFO(ctypes.Structure):
    _fields_ = [
        ("adapterId", LUID),
        ("id", wintypes.UINT),
        ("modeInfoIdx", wintypes.UINT),
        ("outputTechnology", wintypes.UINT),
        ("rotation", wintypes.UINT),
        ("scaling", wintypes.UINT),
        ("refreshRate", DISPLAYCONFIG_RATIONAL),
        ("scanLineOrdering", wintypes.UINT),
        ("targetAvailable", wintypes.BOOL),
        ("statusFlags", wintypes.UINT),
    ]


class DISPLAYCONFIG_PATH_INFO(ctypes.Structure):
    _fields_ = [
        ("sourceInfo", DISPLAYCONFIG_PATH_SOURCE_INFO),
        ("targetInfo", DISPLAYCONFIG_PATH_TARGET_INFO),
        ("flags", wintypes.UINT),
    ]


# We don't actually use MODE_INFO but the API insists we pass a buffer.
# Define a conservative oversize structure so the OS can write safely.
class DISPLAYCONFIG_MODE_INFO(ctypes.Structure):
    _fields_ = [
        ("infoType", wintypes.UINT),
        ("id", wintypes.UINT),
        ("adapterId", LUID),
        ("_blob", ctypes.c_ubyte * 128),  # oversized union payload
    ]


# Prototypes
user32.GetDisplayConfigBufferSizes.argtypes = [
    wintypes.UINT,
    ctypes.POINTER(wintypes.UINT),
    ctypes.POINTER(wintypes.UINT),
]
user32.GetDisplayConfigBufferSizes.restype = wintypes.LONG

user32.QueryDisplayConfig.argtypes = [
    wintypes.UINT,
    ctypes.POINTER(wintypes.UINT),
    ctypes.POINTER(DISPLAYCONFIG_PATH_INFO),
    ctypes.POINTER(wintypes.UINT),
    ctypes.POINTER(DISPLAYCONFIG_MODE_INFO),
    ctypes.c_void_p,  # currentTopologyId (optional)
]
user32.QueryDisplayConfig.restype = wintypes.LONG

user32.DisplayConfigGetDeviceInfo.argtypes = [
    ctypes.POINTER(DISPLAYCONFIG_DEVICE_INFO_HEADER)
]
user32.DisplayConfigGetDeviceInfo.restype = wintypes.LONG

user32.DisplayConfigSetDeviceInfo.argtypes = [
    ctypes.POINTER(DISPLAYCONFIG_DEVICE_INFO_HEADER)
]
user32.DisplayConfigSetDeviceInfo.restype = wintypes.LONG

ERROR_SUCCESS = 0


def _get_paths_modes():
    numPaths = wintypes.UINT(0)
    numModes = wintypes.UINT(0)
    rc = user32.GetDisplayConfigBufferSizes(
        QDC_ONLY_ACTIVE_PATHS, ctypes.byref(numPaths), ctypes.byref(numModes)
    )
    if rc != ERROR_SUCCESS:
        err(f"GetDisplayConfigBufferSizes failed: {rc}")

    paths_arr = (DISPLAYCONFIG_PATH_INFO * numPaths.value)()
    modes_arr = (DISPLAYCONFIG_MODE_INFO * numModes.value)()
    rc = user32.QueryDisplayConfig(
        QDC_ONLY_ACTIVE_PATHS,
        ctypes.byref(numPaths),
        paths_arr,
        ctypes.byref(numModes),
        modes_arr,
        None,
    )
    if rc != ERROR_SUCCESS:
        err(f"QueryDisplayConfig failed: {rc}")
    return [paths_arr[i] for i in range(numPaths.value)]


def _dpi_info_for_source(adapterId: LUID, sourceId: int):
    req = DISPLAYCONFIG_SOURCE_DPI_SCALE_GET()
    req.header.type = DISPLAYCONFIG_DEVICE_INFO_GET_DPI_SCALE
    req.header.size = ctypes.sizeof(req)
    req.header.adapterId = adapterId
    req.header.id = sourceId
    rc = user32.DisplayConfigGetDeviceInfo(ctypes.byref(req.header))
    if rc != ERROR_SUCCESS:
        err(f"DisplayConfigGetDeviceInfo(GET_DPI) failed: {rc}")
    # Map relative indices to absolute percentages using DPI_VALS
    min_abs_idx = abs(req.minScaleRel)
    # Bounds check: OS may allow up to 500%
    if min_abs_idx + req.maxScaleRel >= len(DPI_VALS):
        # Clamp to table
        max_idx = len(DPI_VALS) - 1
    else:
        max_idx = min_abs_idx + req.maxScaleRel
    cur_idx = min_abs_idx + req.curScaleRel
    return {
        "min": DPI_VALS[min_abs_idx],
        "max": DPI_VALS[max_idx],
        "current": DPI_VALS[cur_idx],
        "recommended": DPI_VALS[min_abs_idx],
        "rel": {"min": req.minScaleRel, "cur": req.curScaleRel, "max": req.maxScaleRel},
    }


def list_monitors():
    paths = _get_paths_modes()
    print("Found active display sources:")
    for i, p in enumerate(paths, start=1):
        info_dict = _dpi_info_for_source(p.sourceInfo.adapterId, p.sourceInfo.id)
        print(
            f"{i}. SourceId={p.sourceInfo.id} "
            f"| Current: {info_dict['current']}% "
            f"(Recommended: {info_dict['recommended']}%, Range: {info_dict['min']}–{info_dict['max']}%)"
        )


def set_scale_per_monitor(percent: int, monitor_index: int):
    paths = _get_paths_modes()
    if monitor_index < 1 or monitor_index > len(paths):
        err(f"Monitor index {monitor_index} out of range (1..{len(paths)})")
    p = paths[monitor_index - 1]
    # Look up current DPI info for relative math
    di = _dpi_info_for_source(p.sourceInfo.adapterId, p.sourceInfo.id)
    # Clamp to supported range
    percent = max(di["min"], min(di["max"], percent))
    # Convert absolute percent -> relative step vs recommended
    try:
        idx_target = DPI_VALS.index(percent)
        idx_reco = DPI_VALS.index(di["recommended"])
    except ValueError:
        err(f"Unsupported DPI {percent}%. Supported: {', '.join(map(str, DPI_VALS))}")
    rel = idx_target - idx_reco

    pkt = DISPLAYCONFIG_SOURCE_DPI_SCALE_SET()
    pkt.header.type = DISPLAYCONFIG_DEVICE_INFO_SET_DPI_SCALE
    pkt.header.size = ctypes.sizeof(pkt)
    pkt.header.adapterId = p.sourceInfo.adapterId
    pkt.header.id = p.sourceInfo.id
    pkt.scaleRel = wintypes.INT(rel)
    rc = user32.DisplayConfigSetDeviceInfo(ctypes.byref(pkt.header))
    if rc != ERROR_SUCCESS:
        err(f"DisplayConfigSetDeviceInfo(SET_DPI) failed: {rc}")
    info(f"Applied {percent}% to monitor #{monitor_index} (per-monitor, immediate).")


# --------------------------
# Way 2: Registry (global) + Explorer restart
# --------------------------

LOGPIXELS_BY_PERCENT = {
    100: 96,
    125: 120,
    150: 144,
    175: 168,
    200: 192,
    225: 216,
    250: 240,
    300: 288,
    350: 336,
    400: 384,
    450: 432,
    500: 480,
}


def set_scale_registry_global(percent: int, restart_explorer: bool = True):
    # Enable Win8DpiScaling and set LogPixels under HKCU
    if percent not in LOGPIXELS_BY_PERCENT:
        err(
            f"Unsupported DPI {percent}%. Supported: {', '.join(map(str, LOGPIXELS_BY_PERCENT.keys()))}"
        )
    logpx = LOGPIXELS_BY_PERCENT[percent]
    with winreg.OpenKey(
        winreg.HKEY_CURRENT_USER, r"Control Panel\Desktop", 0, winreg.KEY_SET_VALUE
    ) as k:
        winreg.SetValueEx(k, "Win8DpiScaling", 0, winreg.REG_DWORD, 1)
        winreg.SetValueEx(k, "LogPixels", 0, winreg.REG_DWORD, logpx)
    info(f"Wrote HKCU\\Control Panel\\Desktop (Win8DpiScaling=1, LogPixels={logpx}).")
    if restart_explorer:
        # Restart Explorer to pick up changes without sign-out
        try:
            subprocess.run(
                ["taskkill", "/F", "/IM", "explorer.exe"],
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
        except subprocess.CalledProcessError:
            pass
        time.sleep(0.8)
        subprocess.Popen(["explorer.exe"])
        info("Explorer restarted.")
    else:
        info(
            "You may need to restart Explorer or sign out/in for all apps to pick this up."
        )


# --------------------------
# CLI
# --------------------------


def main():
    ap = argparse.ArgumentParser(
        description="Change Windows display scaling (per-monitor undocumented OR global registry)."
    )
    sub = ap.add_subparsers(dest="cmd", required=True)

    sub.add_parser(
        "list", help="List active monitors with current/recommended DPI (Way 1)."
    )

    sp = sub.add_parser("set", help="Set scaling.")
    sp.add_argument(
        "percent", type=int, help="Target scale percent (e.g., 100, 125, 150, ...)."
    )
    sp.add_argument(
        "--monitor", type=int, default=1, help="1-based monitor index (Way 1 only)."
    )
    sp.add_argument(
        "--method",
        choices=["undoc", "registry"],
        default="undoc",
        help="undoc = per-monitor, immediate; registry = global + Explorer restart.",
    )
    sp.add_argument(
        "--no-restart-explorer",
        action="store_true",
        help="Registry method: do not restart Explorer.",
    )

    args = ap.parse_args()

    if args.cmd == "list":
        list_monitors()
    elif args.cmd == "set":
        if args.method == "undoc":
            set_scale_per_monitor(args.percent, args.monitor)
        else:
            set_scale_registry_global(
                args.percent, restart_explorer=(not args.no_restart_explorer)
            )


if __name__ == "__main__":
    if sys.platform != "win32":
        err("This script must be run on Windows.")
    main()
