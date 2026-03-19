import { invoke } from '@tauri-apps/api/core';
import type { CountdownItemData, WebDavProxySettings } from '../types/countdown';
import { DEFAULT_WEBDAV_PROXY_SETTINGS } from '../types/countdown';
import { normalizeCountdownItems } from './dataService';
import { getProxyPortNumber } from './proxyService';

export const JIANGUOYUN_SERVER = 'https://dav.jianguoyun.com/dav/';

interface RustProxyConfig {
  enabled: boolean;
  port: number | null;
}

export const loadWebDavConfig = async () => {
  return invoke<[string, string] | null>('load_webdav_config');
};

export const hasWebDavConfig = async () => {
  const config = await loadWebDavConfig();
  return !!config;
};

export const testWebDav = async (username: string, password: string) => {
  await invoke('test_webdav', {
    server: JIANGUOYUN_SERVER,
    username,
    password,
  });
};

export const saveWebDavConfig = async (username: string, password: string) => {
  await invoke('save_webdav_config', {
    server: JIANGUOYUN_SERVER,
    username,
    password,
  });
};

export const clearWebDavConfig = async () => {
  await invoke('clear_webdav_config');
};

export const loadWebDavProxyConfig = async (): Promise<WebDavProxySettings> => {
  const config = await invoke<RustProxyConfig>('load_webdav_proxy_config');
  return {
    ...DEFAULT_WEBDAV_PROXY_SETTINGS,
    enabled: config.enabled,
    port: config.port ? String(config.port) : '',
  };
};

export const saveWebDavProxyConfig = async (settings: WebDavProxySettings) => {
  await invoke('save_webdav_proxy_config', {
    enabled: settings.enabled,
    port: getProxyPortNumber(settings),
  });
};

export const uploadCountdownItems = async (items: CountdownItemData[]) => {
  await invoke('webdav_upload', { json: JSON.stringify(items) });
};

export const downloadCountdownItems = async (): Promise<CountdownItemData[]> => {
  const json = await invoke<string>('webdav_download');
  const items = JSON.parse(json) as CountdownItemData[];
  return normalizeCountdownItems(items);
};
