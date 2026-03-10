import { invoke } from '@tauri-apps/api/core';

export const getAutostartStatus = async () => {
  return invoke<boolean>('get_autostart');
};

export const setAutostartStatus = async (enable: boolean) => {
  await invoke('set_autostart', { enable });
};
