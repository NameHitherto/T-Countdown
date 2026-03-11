import {
  LOCK_STATE_STORAGE_KEY,
  PRIVACY_SETTINGS_STORAGE_KEY,
  UPDATE_PROXY_STORAGE_KEY,
  WINDOW_STATE_STORAGE_KEY,
} from '../constants/app';
import type { PrivacySettings, UpdateProxySettings, WindowState } from '../types/countdown';
import { DEFAULT_PRIVACY_SETTINGS, DEFAULT_UPDATE_PROXY_SETTINGS } from '../types/countdown';

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

export const loadPrivacySettings = (): PrivacySettings => {
  const saved = readJson<Partial<PrivacySettings>>(PRIVACY_SETTINGS_STORAGE_KEY);
  return { ...DEFAULT_PRIVACY_SETTINGS, ...saved };
};

export const savePrivacySettings = (settings: PrivacySettings) => {
  writeJson(PRIVACY_SETTINGS_STORAGE_KEY, settings);
};

export const loadUpdateProxySettings = (): UpdateProxySettings => {
  const saved = readJson<Partial<UpdateProxySettings>>(UPDATE_PROXY_STORAGE_KEY);
  return { ...DEFAULT_UPDATE_PROXY_SETTINGS, ...saved };
};

export const saveUpdateProxySettings = (settings: UpdateProxySettings) => {
  writeJson(UPDATE_PROXY_STORAGE_KEY, settings);
};
