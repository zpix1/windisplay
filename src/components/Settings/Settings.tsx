import { Dialog } from "../ui/Dialog/Dialog";
import {
  useSettings,
  type KeyboardBrightnessShortcut,
} from "../../hooks/useSettings";
import "./Settings.css";

type SettingsProps = {
  isOpen: boolean;
  onClose: () => void;
};

export function Settings({ isOpen, onClose }: SettingsProps) {
  const { settings, updateSettings, loading } = useSettings();

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
                Brightness Keys Affect:
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

                <label className="settings-radio-label">
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
                </label>

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
                    Use system behavior
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
          </div>
        )}
      </div>
    </Dialog>
  );
}
