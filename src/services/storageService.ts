import {
  LOCK_STATE_STORAGE_KEY,
  PRIVACY_SETTINGS_STORAGE_KEY,
  UPDATE_PROXY_STORAGE_KEY,
  WINDOW_STATE_STORAGE_KEY,
} from '../constants/app';
import { invoke } from '@tauri-apps/api/core';
import type { PrivacySettings, UpdateProxySettings, WindowState } from '../types/countdown';
import { DEFAULT_PRIVACY_SETTINGS, DEFAULT_UPDATE_PROXY_SETTINGS } from '../types/countdown';

interface RustPrivacySettings {
  enabled: boolean;
  long_press_duration: number;
  mask_mode: string;
  mask_image: string;
}

const normalizePrivacySettings = (settings: Partial<PrivacySettings>): PrivacySettings => {
  const normalizedDuration = Number.isFinite(settings.longPressDuration)
    ? Number(settings.longPressDuration)
    : DEFAULT_PRIVACY_SETTINGS.longPressDuration;
  return {
    ...DEFAULT_PRIVACY_SETTINGS,
    ...settings,
    longPressDuration: Math.max(300, Math.min(5000, normalizedDuration)),
    maskMode: settings.maskMode === 'image' ? 'image' : 'blur',
    maskImage: settings.maskImage ?? '',
  };
};

const readJson = <T>(key: string): T | null => {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch {
    return null;
  }
};

const writeJson = (key: string, value: unknown) => {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // 忽略本地存储失败
  }
};

export const loadWindowState = () => readJson<WindowState>(WINDOW_STATE_STORAGE_KEY);

export const saveWindowState = (state: WindowState) => {
  writeJson(WINDOW_STATE_STORAGE_KEY, state);
};

export const loadLockState = () => readJson<boolean>(LOCK_STATE_STORAGE_KEY) === true;

export const saveLockState = (locked: boolean) => {
  writeJson(LOCK_STATE_STORAGE_KEY, locked);
};

export const loadPrivacySettings = async (): Promise<PrivacySettings> => {
  try {
    const saved = await invoke<RustPrivacySettings>('load_privacy_settings');
    return normalizePrivacySettings({
      enabled: saved.enabled,
      longPressDuration: saved.long_press_duration,
      maskMode: saved.mask_mode === 'image' ? 'image' : 'blur',
      maskImage: saved.mask_image,
    });
  } catch {
    const localSaved = readJson<Partial<PrivacySettings>>(PRIVACY_SETTINGS_STORAGE_KEY);
    return normalizePrivacySettings(localSaved ?? {});
  }
};

export const savePrivacySettings = async (settings: PrivacySettings) => {
  const normalized = normalizePrivacySettings(settings);
  writeJson(PRIVACY_SETTINGS_STORAGE_KEY, normalized);
  await invoke('save_privacy_settings', {
    enabled: normalized.enabled,
    longPressDuration: normalized.longPressDuration,
    maskMode: normalized.maskMode,
    maskImage: normalized.maskImage,
  });
};

export const loadUpdateProxySettings = (): UpdateProxySettings => {
  const saved = readJson<Partial<UpdateProxySettings>>(UPDATE_PROXY_STORAGE_KEY);
  return { ...DEFAULT_UPDATE_PROXY_SETTINGS, ...saved };
};

export const saveUpdateProxySettings = (settings: UpdateProxySettings) => {
  writeJson(UPDATE_PROXY_STORAGE_KEY, settings);
};
