import { computed, ref, watch } from 'vue';
import { SYNC_DEBOUNCE } from '../constants/app';
import { loadCountdownItems, saveCountdownItems } from '../services/dataService';
import {
  downloadCountdownItems,
  hasWebDavConfig,
  loadSyncSettings,
  uploadCountdownItems,
} from '../services/syncService';
import { DEFAULT_SYNC_SETTINGS } from '../types/countdown';
import type { CountdownItemData, SyncSettings } from '../types/countdown';

export const useCountdownItems = () => {
  const countdownItems = ref<CountdownItemData[]>([]);
  const syncSettings = ref<SyncSettings>({ ...DEFAULT_SYNC_SETTINGS });
  const tick = ref(0);
  let webdavConfigured = false;
  let timer: number | null = null;
  let syncDebounceTimer: number | null = null;
  let autoSyncTimer: number | null = null;

  const checkExpired = () => {
    const now = Date.now();
    for (const item of countdownItems.value) {
      if (item.status === 'active' && item.targetDate <= now) {
        item.status = 'expired';
      }
    }
  };

  const sortedItems = computed(() => {
    void tick.value;
    const statusOrder: Record<string, number> = { active: 0, expired: 1, dismissed: 2 };
    return [...countdownItems.value].sort((left, right) => {
      const leftOrder = statusOrder[left.status] ?? 3;
      const rightOrder = statusOrder[right.status] ?? 3;
      if (leftOrder !== rightOrder) {
        return leftOrder - rightOrder;
      }
      return left.targetDate - right.targetDate;
    });
  });

  const loadData = async () => {
    try {
      countdownItems.value = await loadCountdownItems();
      checkExpired();
    } catch {
      countdownItems.value = [];
    }
  };

  const persistData = async () => {
    try {
      await saveCountdownItems(countdownItems.value);
    } catch {
      // 忽略保存失败
    }
  };

  const refreshSyncConfig = async () => {
    try {
      webdavConfigured = await hasWebDavConfig();
    } catch {
      webdavConfigured = false;
    }

    try {
      syncSettings.value = await loadSyncSettings();
    } catch {
      syncSettings.value = { ...DEFAULT_SYNC_SETTINGS };
    }

    restartAutoSyncTimer();
  };

  const doCloudSync = async () => {
    if (!webdavConfigured) {
      return;
    }
    try {
      await uploadCountdownItems(countdownItems.value);
    } catch {
      // 静默失败，不打扰用户
    }
  };

  const scheduleDebouncedSync = () => {
    if (!webdavConfigured) {
      return;
    }
    if (syncDebounceTimer) {
      clearTimeout(syncDebounceTimer);
    }
    syncDebounceTimer = window.setTimeout(() => {
      void doCloudSync();
      syncDebounceTimer = null;
    }, SYNC_DEBOUNCE);
  };

  const restartAutoSyncTimer = () => {
    if (autoSyncTimer) {
      clearInterval(autoSyncTimer);
      autoSyncTimer = null;
    }

    if (!webdavConfigured) {
      return;
    }

    autoSyncTimer = window.setInterval(() => {
      void doCloudSync();
    }, syncSettings.value.autoSyncIntervalMs);
  };

  const syncFromCloudOnLaunch = async () => {
    if (!webdavConfigured) {
      return;
    }

    try {
      const remoteItems = await downloadCountdownItems();
      countdownItems.value = remoteItems;
      checkExpired();
      await persistData();
    } catch {
      // 首次自动同步失败时静默降级，保留本地数据
    }
  };

  const startRuntime = () => {
    timer = window.setInterval(() => {
      tick.value += 1;
      if (tick.value % 10 === 0) {
        checkExpired();
      }
    }, 1000);

    restartAutoSyncTimer();
  };

  const stopRuntime = () => {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
    if (autoSyncTimer) {
      clearInterval(autoSyncTimer);
      autoSyncTimer = null;
    }
    if (syncDebounceTimer) {
      clearTimeout(syncDebounceTimer);
      syncDebounceTimer = null;
    }
  };

  const handleAddConfirm = (item: CountdownItemData) => {
    countdownItems.value.push(item);
  };

  const handleItemUpdate = (updated: CountdownItemData) => {
    const index = countdownItems.value.findIndex((item) => item.id === updated.id);
    if (index === -1) {
      return;
    }
    if (updated.targetDate > Date.now()) {
      updated.status = 'active';
    }
    countdownItems.value[index] = updated;
  };

  const handleDismiss = (id: string) => {
    const item = countdownItems.value.find((entry) => entry.id === id);
    if (item) {
      item.status = 'dismissed';
    }
  };

  const handleDelete = (id: string) => {
    countdownItems.value = countdownItems.value.filter((item) => item.id !== id);
  };

  const handleSynced = (items: CountdownItemData[]) => {
    countdownItems.value = items;
  };

  watch(
    countdownItems,
    () => {
      void persistData();
      scheduleDebouncedSync();
    },
    { deep: true },
  );

  return {
    countdownItems,
    syncSettings,
    tick,
    sortedItems,
    loadData,
    persistData,
    refreshSyncConfig,
    syncFromCloudOnLaunch,
    startRuntime,
    stopRuntime,
    handleAddConfirm,
    handleItemUpdate,
    handleDismiss,
    handleDelete,
    handleSynced,
  };
};
