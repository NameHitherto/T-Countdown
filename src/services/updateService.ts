import { getVersion } from '@tauri-apps/api/app';
import { check, type Update } from '@tauri-apps/plugin-updater';

export const getAppVersion = async () => {
  return getVersion();
};

export const checkForAppUpdate = async (): Promise<Update | null> => {
  return check();
};

export const downloadAndInstallUpdate = async (
  update: Update,
  onProgress: (message: string) => void,
) => {
  let downloaded = 0;
  let contentLength = 0;

  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case 'Started':
        contentLength = event.data.contentLength ?? 0;
        break;
      case 'Progress':
        downloaded += event.data.chunkLength;
        if (contentLength > 0) {
          const percentage = Math.round((downloaded / contentLength) * 100);
          onProgress(`下载中 ${percentage}%`);
        }
        break;
      case 'Finished':
        onProgress('正在安装...');
        break;
    }
  });
};
