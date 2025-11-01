import { useSyncExternalStore, useCallback } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { error as logError } from "@tauri-apps/plugin-log";

export type KeyboardBrightnessShortcut =
  | "all_screens"
  | "screen_with_mouse"
  | "system";

export type Settings = {
  showUIOnMonitorChange: boolean;
  keyboardBrightnessShortcut: KeyboardBrightnessShortcut;
};

const DEFAULT_SETTINGS: Settings = {
  showUIOnMonitorChange: false,
  keyboardBrightnessShortcut: "system",
};

const STORE_KEY = "settings";

class SettingsStore {
  private store: Store | null = null;
  private settings: Settings = DEFAULT_SETTINGS;
  private loading: boolean = true;
  private listeners: Set<() => void> = new Set();

  async init() {
    try {
      this.store = await Store.load("settings.json");
      const stored = await this.store.get<Settings>(STORE_KEY);
      this.settings = stored
        ? { ...DEFAULT_SETTINGS, ...stored }
        : DEFAULT_SETTINGS;
    } catch (err) {
      logError(`Failed to load settings: ${err}`);
    } finally {
      this.loading = false;
      this.notify();
    }
  }

  getSettings(): Settings {
    return this.settings;
  }

  isLoading(): boolean {
    return this.loading;
  }

  async updateSettings(partial: Partial<Settings>): Promise<void> {
    this.settings = { ...this.settings, ...partial };
    this.notify();

    try {
      if (!this.store) {
        this.store = await Store.load("settings.json");
      }
      await this.store.set(STORE_KEY, this.settings);
      await this.store.save();
    } catch (err) {
      logError(`Failed to save settings: ${err}`);
      throw err;
    }
  }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify() {
    this.listeners.forEach((listener) => listener());
  }
}

const settingsStore = new SettingsStore();
settingsStore.init();

export function useSettings() {
  const settings = useSyncExternalStore(
    (listener) => settingsStore.subscribe(listener),
    () => settingsStore.getSettings(),
    () => DEFAULT_SETTINGS
  );

  const loading = useSyncExternalStore(
    (listener) => settingsStore.subscribe(listener),
    () => settingsStore.isLoading(),
    () => true
  );

  const updateSettings = useCallback(async (partial: Partial<Settings>) => {
    await settingsStore.updateSettings(partial);
  }, []);

  return {
    settings,
    updateSettings,
    loading,
  };
}
