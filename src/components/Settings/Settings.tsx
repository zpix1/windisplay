import { Dialog } from "../ui/Dialog/Dialog";
import { useRef, useEffect, useMemo, useState, type KeyboardEvent } from "react";
import {
  useSettings,
  type KeyboardBrightnessShortcut,
  type Settings,
  type ThemeMode,
} from "../../hooks/useSettings";
import { relaunch } from "@tauri-apps/plugin-process";
import "./Settings.css";
import { CheckUpdatesButton } from "./CheckUpdatesButton";

type SettingsProps = {
  isOpen: boolean;
  onClose: () => void;
};

const RESTART_REQUIRED_KEYS: Array<keyof Settings> = [
  "keyboardBrightnessShortcut",
  "brightnessDownHotkey",
  "brightnessUpHotkey",
];

export function Settings({ isOpen, onClose }: SettingsProps) {
  const { settings, updateSettings, loading } = useSettings();
  const [shortcutError, setShortcutError] = useState("");
  const initialSettingsRef = useRef<Settings | null>(null);
  useEffect(() => {
    if (!loading && initialSettingsRef.current === null) {
      initialSettingsRef.current = settings;
    }
  }, [loading, settings]);

  const restartRequired = useMemo(() => {
    if (loading || !initialSettingsRef.current) return false;
    return RESTART_REQUIRED_KEYS.some(
      (key) => initialSettingsRef.current![key] !== settings[key]
    );
  }, [loading, settings]);

  const handleCheckboxChange = (checked: boolean) => {
    updateSettings({ showUIOnMonitorChange: checked });
  };

  const handleFocusOutCheckboxChange = (checked: boolean) => {
    updateSettings({ shouldHideUIOnFocusOut: checked });
  };

  const handleRadioChange = (value: KeyboardBrightnessShortcut) => {
    updateSettings({ keyboardBrightnessShortcut: value });
  };

  const handleThemeChange = (value: ThemeMode) => {
    updateSettings({ theme: value });
  };

  const captureShortcut = (
    event: KeyboardEvent<HTMLInputElement>,
    direction: "brightnessDownHotkey" | "brightnessUpHotkey"
  ) => {
    if (event.key === "Tab") return;
    event.preventDefault();
    if (/^(Control|Shift|Alt|Meta)(Left|Right)$/.test(event.code)) return;
    if (!/^(F([1-9]|1[0-9]|2[0-4])|Key[A-Z]|Digit[0-9]|Arrow(Up|Down|Left|Right))$/.test(event.code)) {
      setShortcutError("Use a letter, number, arrow, or F1–F24 key, with optional modifiers.");
      return;
    }
    const shortcut = [
      event.ctrlKey && "Ctrl",
      event.altKey && "Alt",
      event.shiftKey && "Shift",
      event.metaKey && "Super",
      event.code,
    ].filter(Boolean).join("+");
    const other = direction === "brightnessDownHotkey" ? settings.brightnessUpHotkey : settings.brightnessDownHotkey;
    if (shortcut.toLowerCase() === other.toLowerCase()) {
      setShortcutError("Brightness up and down must use different shortcuts.");
      return;
    }
    setShortcutError("");
    void updateSettings({ [direction]: shortcut });
  };

  return (
    <Dialog isOpen={isOpen} onClose={onClose} className="settings-dialog">
      <div className="settings-container">
        {loading ? (
          <div className="settings-loading">Loading settings...</div>
        ) : (
          <div className="settings-content">
            <div className="settings-section">
              <div className="settings-section-title">Theme</div>
              <div className="settings-radio-group">
                <label className="settings-radio-label">
                  <input
                    type="radio"
                    name="theme"
                    value="system"
                    checked={settings.theme === "system"}
                    onChange={() => handleThemeChange("system")}
                    className="settings-radio"
                  />
                  <span className="settings-radio-text">System default</span>
                </label>
                <label className="settings-radio-label">
                  <input
                    type="radio"
                    name="theme"
                    value="light"
                    checked={settings.theme === "light"}
                    onChange={() => handleThemeChange("light")}
                    className="settings-radio"
                  />
                  <span className="settings-radio-text">Light</span>
                </label>
                <label className="settings-radio-label">
                  <input
                    type="radio"
                    name="theme"
                    value="dark"
                    checked={settings.theme === "dark"}
                    onChange={() => handleThemeChange("dark")}
                    className="settings-radio"
                  />
                  <span className="settings-radio-text">Dark</span>
                </label>
              </div>
            </div>
            <div className="settings-section">
              <div className="settings-section-title">
                Brightness Keys Affect:{" "}
                <span
                  className="settings-section-title-note"
                  title="Choose shortcuts below and map your keyboard to send them. Monitor brightness changes may be slow."
                >
                  (?)
                </span>
              </div>
              <div className="settings-radio-group">
                <label className="settings-radio-label">
                  <input
                    type="radio"
                    name="keyboardBrightnessShortcut"
                    value="all_screens"
                    checked={
                      settings.keyboardBrightnessShortcut === "all_screens"
                    }
                    onChange={() => handleRadioChange("all_screens")}
                    className="settings-radio"
                  />
                  <span className="settings-radio-text">All screens</span>
                </label>

                {/* <label className="settings-radio-label">
                  <input
                    type="radio"
                    name="keyboardBrightnessShortcut"
                    value="screen_with_mouse"
                    checked={
                      settings.keyboardBrightnessShortcut ===
                      "screen_with_mouse"
                    }
                    onChange={() => handleRadioChange("screen_with_mouse")}
                    className="settings-radio"
                  />
                  <span className="settings-radio-text">
                    Screen with mouse pointer
                  </span>
                </label> */}

                <label className="settings-radio-label">
                  <input
                    type="radio"
                    name="keyboardBrightnessShortcut"
                    value="system"
                    checked={settings.keyboardBrightnessShortcut === "system"}
                    onChange={() => handleRadioChange("system")}
                    className="settings-radio"
                  />
                  <span className="settings-radio-text">
                    Use system behavior (no action)
                  </span>
                </label>
              </div>
              {settings.keyboardBrightnessShortcut === "all_screens" && (
                <div className="settings-shortcuts">
                  <label className="settings-shortcut-row">
                    <span>Brightness down</span>
                    <input className="settings-shortcut-input" aria-label="Brightness down shortcut" title="Click and press a shortcut" readOnly value={settings.brightnessDownHotkey} onKeyDown={(event) => captureShortcut(event, "brightnessDownHotkey")} />
                  </label>
                  <label className="settings-shortcut-row">
                    <span>Brightness up</span>
                    <input className="settings-shortcut-input" aria-label="Brightness up shortcut" title="Click and press a shortcut" readOnly value={settings.brightnessUpHotkey} onKeyDown={(event) => captureShortcut(event, "brightnessUpHotkey")} />
                  </label>
                  {shortcutError && <p role="alert">{shortcutError}</p>}
                </div>
              )}
            </div>
            <div className="settings-section">
              <label className="settings-checkbox-label">
                <input
                  type="checkbox"
                  checked={settings.showUIOnMonitorChange}
                  onChange={(e) => handleCheckboxChange(e.target.checked)}
                  className="settings-checkbox"
                />
                <span className="settings-checkbox-text">
                  Show controls when a monitor is connected or disconnected
                </span>
              </label>
            </div>
            <div className="settings-section">
              <label className="settings-checkbox-label">
                <input
                  type="checkbox"
                  checked={settings.shouldHideUIOnFocusOut}
                  onChange={(e) =>
                    handleFocusOutCheckboxChange(e.target.checked)
                  }
                  className="settings-checkbox"
                />
                <span className="settings-checkbox-text">
                  Hide controls when focus is lost
                </span>
              </label>
            </div>
            <div className="settings-section">
              <label className="settings-checkbox-label">
                <input
                  type="checkbox"
                  checked={settings.showStartupNotification}
                  onChange={(e) =>
                    updateSettings({ showStartupNotification: e.target.checked })
                  }
                  className="settings-checkbox"
                />
                <span className="settings-checkbox-text">
                  Show notification when app starts
                </span>
              </label>
            </div>
            <div className="settings-section">
              <CheckUpdatesButton />
            </div>
            {restartRequired && (
              <div className="settings-section">
                <button
                  className="button"
                  onClick={() => {
                    void relaunch();
                  }}
                >
                  Restart required
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </Dialog>
  );
}
