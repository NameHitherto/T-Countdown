import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import type { UpdateProxySettings } from '../types/countdown';
import { getActiveProxyPort } from './proxyService';

export interface AppUpdateInfo {
  version: string;
  body?: string | null;
}

const getProxyPort = (settings: UpdateProxySettings) => {
  const port = getActiveProxyPort(settings);
  return port ? Number(port) : null;
};

export const getAppVersion = async () => {
  return getVersion();
};

export const checkForAppUpdate = async (
  proxySettings: UpdateProxySettings,
): Promise<AppUpdateInfo | null> => {
  return invoke<AppUpdateInfo | null>('check_update', {
    proxyPort: getProxyPort(proxySettings),
  });
};

export const downloadAndInstallUpdate = async (
  proxySettings: UpdateProxySettings,
  onProgress: (message: string) => void,
) => {
  onProgress('正在下载并安装...');
  await invoke('download_and_install_update', {
    proxyPort: getProxyPort(proxySettings),
  });
};
