from __future__ import annotations

import sys
import threading
import time
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple
import os
import logging
import tempfile
import webbrowser

import win32api
import win32con
from PIL import Image
import importlib.resources as resources

os.environ.setdefault("PYSTRAY_BACKEND", "win32")
import pystray

# Win32 constants and helpers
ENUM_CURRENT_SETTINGS = -1
CDS_UPDATEREGISTRY = 0x00000001
CDS_TEST = 0x00000002
DISP_CHANGE_SUCCESSFUL = 0

# Simple logging to temp file for diagnostics
_LOG_PATH = os.path.join(tempfile.gettempdir(), "windisplay_tray.log")
logging.basicConfig(
    filename=_LOG_PATH,
    level=logging.DEBUG,
    format="%(asctime)s %(levelname)s: %(message)s",
)
logging.debug("windisplay starting up")


@dataclass(frozen=True)
class DisplayMode:
    width: int
    height: int
    bits_per_pixel: int
    display_frequency: int

    def as_label(self) -> str:
        return f"{self.width}x{self.height} @ {self.display_frequency}Hz"


@dataclass(frozen=True)
class Monitor:
    device_name: str  # e.g., "\\\\.\\DISPLAY1"
    friendly_name: str  # e.g., "DISPLAY1 (Primary)"


def _enumerate_monitors() -> List[Monitor]:
    monitors: List[Monitor] = []
    i = 0
    while True:
        try:
            dev = win32api.EnumDisplayDevices(None, i)
        except win32api.error:
            break
        if not dev.DeviceName:
            break
        if dev.StateFlags & win32con.DISPLAY_DEVICE_ATTACHED_TO_DESKTOP:
            friendly = dev.DeviceString or dev.DeviceName
            if dev.StateFlags & win32con.DISPLAY_DEVICE_PRIMARY_DEVICE:
                friendly += " (Primary)"
            monitors.append(Monitor(device_name=dev.DeviceName, friendly_name=friendly))
        i += 1
    logging.debug("enumerated monitors: %s", [m.friendly_name for m in monitors])
    return monitors


def _enumerate_modes(device_name: str) -> List[DisplayMode]:
    modes: List[DisplayMode] = []
    i = 0
    while True:
        try:
            dm = win32api.EnumDisplaySettings(device_name, i)
        except win32api.error:
            break
        if not dm:
            break
        modes.append(
            DisplayMode(
                width=dm.PelsWidth,
                height=dm.PelsHeight,
                bits_per_pixel=dm.BitsPerPel,
                display_frequency=dm.DisplayFrequency,
            )
        )
        i += 1
    # Sort by width desc, height desc, refresh desc; dedupe equal tuples
    unique: Dict[Tuple[int, int, int], DisplayMode] = {}
    for m in modes:
        key = (m.width, m.height, m.display_frequency)
        if key not in unique:
            unique[key] = m
    sorted_modes = sorted(
        unique.values(),
        key=lambda m: (m.width, m.height, m.display_frequency),
        reverse=True,
    )
    logging.debug("enumerated %d unique modes for %s", len(sorted_modes), device_name)
    return sorted_modes


def _get_current_mode(device_name: str) -> Optional[DisplayMode]:
    try:
        dm = win32api.EnumDisplaySettings(device_name, ENUM_CURRENT_SETTINGS)
        if dm:
            return DisplayMode(
                dm.PelsWidth, dm.PelsHeight, dm.BitsPerPel, dm.DisplayFrequency
            )
    except win32api.error:
        return None
    return None


def _apply_mode(device_name: str, mode: DisplayMode) -> bool:
    # Start from current devmode to avoid leaving unspecified fields unset
    try:
        devmode = win32api.EnumDisplaySettings(device_name, ENUM_CURRENT_SETTINGS)
    except win32api.error:
        return False

    devmode.PelsWidth = mode.width
    devmode.PelsHeight = mode.height
    devmode.BitsPerPel = mode.bits_per_pixel
    devmode.DisplayFrequency = mode.display_frequency
    devmode.Fields = (
        win32con.DM_PELSWIDTH
        | win32con.DM_PELSHEIGHT
        | win32con.DM_BITSPERPEL
        | win32con.DM_DISPLAYFREQUENCY
    )

    # Test first
    result_test = win32api.ChangeDisplaySettingsEx(device_name, devmode, Flags=CDS_TEST)
    if result_test != DISP_CHANGE_SUCCESSFUL:
        logging.warning(
            "mode test failed code=%s for %s on %s", result_test, mode, device_name
        )
        return False
    # Try immediate apply without touching registry
    result = win32api.ChangeDisplaySettingsEx(device_name, devmode, Flags=0)
    if result != DISP_CHANGE_SUCCESSFUL:
        # Fallback: write to registry with no reset, then commit
        try:
            result2 = win32api.ChangeDisplaySettingsEx(
                device_name,
                devmode,
                Flags=win32con.CDS_UPDATEREGISTRY | win32con.CDS_NORESET,
            )
        except AttributeError:
            # Older pywin32 may expose CDS_UPDATEREGISTRY via our const
            result2 = win32api.ChangeDisplaySettingsEx(
                device_name, devmode, Flags=CDS_UPDATEREGISTRY | win32con.CDS_NORESET
            )
        logging.debug("registry stage result=%s", result2)
        # Commit the change across the system
        result_commit = win32api.ChangeDisplaySettingsEx(None, None, Flags=0)
        logging.debug("commit stage result=%s", result_commit)
        ok = (
            result2 == DISP_CHANGE_SUCCESSFUL
            and result_commit == DISP_CHANGE_SUCCESSFUL
        )
    else:
        ok = True
    logging.info("apply mode %s on %s -> %s", mode, device_name, ok)
    return ok


class WinDisplayTray:
    def __init__(self) -> None:
        self.icon: Optional[pystray.Icon] = None
        self._menu_lock = threading.Lock()
        self._image: Optional[Image.Image] = None
        self._menu: Optional[pystray.Menu] = None
        # Preferred refresh rate per monitor (Hz). Defaults to current at startup
        self._preferred_freq: Dict[str, int] = {}

    def _load_tray_icon_from_assets(self) -> Image.Image:
        logging.info("loading tray icon from assets")
        path = os.path.join(sys._MEIPASS, "assets/app.ico")
        ico = Image.open(path)
        return ico.convert("RGBA")

    def _open_github(
        self,
        icon: Optional[pystray.Icon] = None,
        item: Optional[pystray.MenuItem] = None,
    ) -> None:
        try:
            webbrowser.open("https://github.com/zpix1/windisplay")
        except Exception as exc:
            logging.exception("open github failed: %s", exc)

    def _build_menu(self) -> pystray.Menu:
        try:
            monitors = _enumerate_monitors()

            def make_monitor_menu(mon: Monitor, index: int) -> pystray.MenuItem:
                current = _get_current_mode(mon.device_name)
                modes = _enumerate_modes(mon.device_name)

                # Initialize preferred freq to current
                if current and mon.device_name not in self._preferred_freq:
                    self._preferred_freq[mon.device_name] = current.display_frequency

                # Unique resolutions
                seen_res: Dict[Tuple[int, int], DisplayMode] = {}
                for m in modes:
                    key = (m.width, m.height)
                    if key not in seen_res:
                        seen_res[key] = m
                all_resolutions = sorted(
                    seen_res.keys(), key=lambda wh: (wh[0], wh[1]), reverse=True
                )

                # Compute aspect ratio of current monitor
                def aspect_ratio_tuple(w: int, h: int) -> Tuple[int, int]:
                    import math

                    g = math.gcd(w, h)
                    return (w // g, h // g)

                cur_ratio = (
                    aspect_ratio_tuple(current.width, current.height)
                    if current
                    else None
                )

                # Popular resolutions map
                popular_labels: Dict[Tuple[int, int], str] = {
                    (1280, 720): "720p",
                    (1920, 1080): "1080p",
                    (2560, 1440): "2K",
                    (3840, 2160): "4K",
                    (7680, 4320): "8K",
                }

                # Popular list = same aspect ratio resolutions + explicitly labeled common ones present
                popular_set: List[Tuple[int, int]] = []
                if cur_ratio is not None:
                    for w, h in all_resolutions:
                        if aspect_ratio_tuple(w, h) == cur_ratio:
                            popular_set.append((w, h))
                for wh, _label in popular_labels.items():
                    if wh in all_resolutions and wh not in popular_set:
                        popular_set.append(wh)

                # Deduplicate while preserving order
                seen: set = set()
                popular_resolutions: List[Tuple[int, int]] = []
                for wh in popular_set:
                    if wh not in seen:
                        seen.add(wh)
                        popular_resolutions.append(wh)

                other_resolutions = [wh for wh in all_resolutions if wh not in seen]

                # Unique refresh rates
                freqs = sorted({m.display_frequency for m in modes}, reverse=True)

                def apply_resolution(width: int, height: int):
                    def _inner(icon=None, item=None, *args, **kwargs):
                        logging.info(
                            "click: set resolution %sx%s on %s",
                            width,
                            height,
                            mon.device_name,
                        )
                        try:
                            pref = self._preferred_freq.get(mon.device_name)
                            # Find mode with exact res and preferred freq; fallback to highest available freq
                            candidate = None
                            if pref is not None:
                                for m in modes:
                                    if (
                                        m.width == width
                                        and m.height == height
                                        and m.display_frequency == pref
                                    ):
                                        candidate = m
                                        break
                            if candidate is None:
                                best = [
                                    m
                                    for m in modes
                                    if m.width == width and m.height == height
                                ]
                                if best:
                                    candidate = max(
                                        best, key=lambda m: m.display_frequency
                                    )
                            if candidate is None and current is not None:
                                # Fallback to current mode
                                candidate = DisplayMode(
                                    width,
                                    height,
                                    current.bits_per_pixel,
                                    current.display_frequency,
                                )
                            if candidate is not None:
                                ok = _apply_mode(mon.device_name, candidate)
                                logging.info(
                                    "applied resolution %sx%s @ %sHz -> %s",
                                    candidate.width,
                                    candidate.height,
                                    candidate.display_frequency,
                                    ok,
                                )
                                self.refresh()
                        except Exception:
                            logging.exception("error applying resolution")

                    return _inner

                def apply_frequency(freq: int):
                    def _inner(icon=None, item=None, *args, **kwargs):
                        logging.info(
                            "click: set frequency %sHz on %s", freq, mon.device_name
                        )
                        try:
                            # Remember preference
                            self._preferred_freq[mon.device_name] = freq
                            # Keep current resolution, switch frequency if possible
                            cur = _get_current_mode(mon.device_name)
                            if cur is None:
                                return
                            # Find mode with same res and desired freq; fallback to closest available (max <= desired, else max)
                            exact = None
                            same_res = [
                                m
                                for m in modes
                                if m.width == cur.width and m.height == cur.height
                            ]
                            for m in same_res:
                                if m.display_frequency == freq:
                                    exact = m
                                    break
                            candidate = exact
                            if candidate is None and same_res:
                                # Pick the one with highest frequency
                                candidate = max(
                                    same_res, key=lambda m: m.display_frequency
                                )
                            if candidate is not None:
                                ok = _apply_mode(mon.device_name, candidate)
                                logging.info(
                                    "applied frequency %sHz at %sx%s -> %s",
                                    candidate.display_frequency,
                                    candidate.width,
                                    candidate.height,
                                    ok,
                                )
                                self.refresh()
                        except Exception:
                            logging.exception("error applying frequency")

                    return _inner

                # Build Resolution submenu
                res_items: List[pystray.MenuItem] = []
                if current is not None:
                    res_items.append(
                        pystray.MenuItem(
                            f"Current: {current.width}x{current.height}",
                            None,
                            enabled=False,
                        )
                    )
                    res_items.append(pystray.Menu.SEPARATOR)
                # Popular first
                for w, h in popular_resolutions:
                    base = f"{w}x{h}"
                    label_suffix = (
                        f" ({popular_labels[(w, h)]})"
                        if (w, h) in popular_labels
                        else ""
                    )
                    label = base + label_suffix
                    if current and (w, h) == (current.width, current.height):
                        label = "✓ " + label
                    res_items.append(pystray.MenuItem(label, apply_resolution(w, h)))
                # Others under More...
                if other_resolutions:
                    more_items: List[pystray.MenuItem] = []
                    for w, h in other_resolutions:
                        label = f"{w}x{h}"
                        if current and (w, h) == (current.width, current.height):
                            label = "✓ " + label
                        more_items.append(
                            pystray.MenuItem(label, apply_resolution(w, h))
                        )
                    res_items.append(pystray.Menu.SEPARATOR)
                    res_items.append(
                        pystray.MenuItem("More…", pystray.Menu(*more_items))
                    )

                # Build Refresh submenu
                pref_freq = self._preferred_freq.get(
                    mon.device_name, current.display_frequency if current else None
                )
                freq_items: List[pystray.MenuItem] = []
                if current is not None:
                    freq_items.append(
                        pystray.MenuItem(
                            f"Current: {current.display_frequency} Hz",
                            None,
                            enabled=False,
                        )
                    )
                    freq_items.append(pystray.Menu.SEPARATOR)
                for f in freqs:
                    label = f"{f} Hz"
                    if pref_freq and abs(f - int(pref_freq)) <= 1:
                        label = "✓ " + label
                    freq_items.append(pystray.MenuItem(label, apply_frequency(f)))

                items: List[pystray.MenuItem] = []
                if current is not None:
                    items.append(
                        pystray.MenuItem(
                            f"Current: {current.as_label()}", None, enabled=False
                        )
                    )
                    items.append(pystray.Menu.SEPARATOR)
                items.append(pystray.MenuItem("Resolution", pystray.Menu(*res_items)))
                items.append(
                    pystray.MenuItem("Refresh Rate", pystray.Menu(*freq_items))
                )

                display_name = f"Monitor {index+1}"
                return pystray.MenuItem(display_name, pystray.Menu(*items))

            monitor_items = [make_monitor_menu(m, i) for i, m in enumerate(monitors)]

            # About submenu items
            try:
                from . import __version__
            except Exception:
                __version__ = "unknown"
            about_items = [
                pystray.MenuItem(f"WinDisplay v{__version__}", None, enabled=False),
                pystray.Menu.SEPARATOR,
                pystray.MenuItem("Open GitHub", self._open_github),
            ]

            actions = [
                *monitor_items,
                pystray.Menu.SEPARATOR,
                pystray.MenuItem("About", pystray.Menu(*about_items)),
                pystray.MenuItem("Refresh", self.refresh),
                pystray.MenuItem("Exit", self.stop),
            ]
            return pystray.Menu(*actions)
        except Exception as exc:  # Defensive: never fail icon creation
            logging.exception("error building menu: %s", exc)
            fallback = [
                pystray.MenuItem(f"Error building menu: {exc}", None, enabled=False),
                pystray.Menu.SEPARATOR,
                pystray.MenuItem("Refresh", self.refresh),
                pystray.MenuItem("Exit", self.stop),
            ]
            menu = pystray.Menu(*fallback)
            self._menu = menu
            return menu

    def _on_ready(self, icon: pystray.Icon) -> None:
        try:
            icon.visible = True
            self.refresh()
            try:
                icon.notify("WinDisplay is running")
            except Exception:
                pass
            logging.info("tray icon is visible")
        except Exception as exc:
            logging.exception("error in _on_ready: %s", exc)

    def refresh(
        self,
        icon: Optional[pystray.Icon] = None,
        item: Optional[pystray.MenuItem] = None,
    ) -> None:
        if not self.icon:
            return
        with self._menu_lock:
            self.icon.menu = self._build_menu()
            try:
                self.icon.update_menu()
            except Exception:
                # Some pystray backends may not expose update_menu
                pass

    def run(self) -> None:
        image = self._load_tray_icon_from_assets()
        self._image = image  # Keep strong reference
        menu = self._build_menu()
        self.icon = pystray.Icon("WinDisplay", image, "WinDisplay", menu)
        try:
            # Run the tray loop on a separate thread so Ctrl+C in console can stop it
            self.icon.run_detached(self._on_ready)
        except Exception as exc:
            logging.exception("icon.run_detached failed: %s", exc)
            raise
        # Verify briefly that it became visible
        for _ in range(10):
            if self.icon.visible:
                break
            time.sleep(0.2)
        logging.debug("icon.visible=%s", getattr(self.icon, "visible", None))

    def stop(
        self,
        icon: Optional[pystray.Icon] = None,
        item: Optional[pystray.MenuItem] = None,
    ) -> None:
        if self.icon:
            self.icon.stop()


def main() -> None:
    print("Starting WinDisplay (press Ctrl+C in this console to exit)")
    print(f"Log: {_LOG_PATH}")
    tray = WinDisplayTray()
    tray.run()
    # Keep the main thread alive and responsive to Ctrl+C
    try:
        while True:
            time.sleep(0.5)
    except KeyboardInterrupt:
        tray.stop()


if __name__ == "__main__":
    main()
