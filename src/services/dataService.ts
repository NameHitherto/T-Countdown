import { invoke } from '@tauri-apps/api/core';
import type { CountdownItemData } from '../types/countdown';

export const normalizeCountdownItems = (items: CountdownItemData[]): CountdownItemData[] => {
  return items.map((item) => ({
    ...item,
    status: item.status ?? 'active',
  }));
};

export const loadCountdownItems = async (): Promise<CountdownItemData[]> => {
  const json = await invoke<string>('load_data');
  const items = JSON.parse(json) as CountdownItemData[];
  return normalizeCountdownItems(items);
};

export const saveCountdownItems = async (items: CountdownItemData[]) => {
  await invoke('save_data', { json: JSON.stringify(items) });
};
