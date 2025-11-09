import { Dialog } from "../ui/Dialog/Dialog";
import { useRef, useEffect, useMemo } from "react";
import {
  useSettings,
  type KeyboardBrightnessShortcut,
  type Settings,
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
];

export function Settings({ isOpen, onClose }: SettingsProps) {
  const { settings, updateSettings, loading } = useSettings();
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

  const handleRadioChange = (value: KeyboardBrightnessShortcut) => {
    updateSettings({ keyboardBrightnessShortcut: value });
  };

  return (
    <Dialog isOpen={isOpen} onClose={onClose} className="settings-dialog">
      <div className="settings-container">
        {loading ? (
          <div className="settings-loading">Loading settings...</div>
        ) : (
          <div className="settings-content">
            <div className="settings-section">
              <div className="settings-section-title">
                Brightness Keys Affect:{" "}
                <span
                  className="settings-section-title-note"
                  title="We use F14/F15 for brightness adjustment. You can bind these keys to the brightness up/down keys in your keyboard settings. Please note that brighness Windows API is very slow, so it might feel laggy."
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
                  Automatically show UI when a monitor is connected or
                  disconnected
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
