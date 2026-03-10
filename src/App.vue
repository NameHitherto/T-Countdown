<template>
  <div
    ref="widgetContainer"
    class="widget-container"
    :class="{ collapsed: isCollapsed, locked: isLocked }"
    @contextmenu.prevent
    @click="handleContainerClick"
    @mousedown="handleDragStart"
  >
    <!-- Header -->
    <div class="content-wrapper" :style="contentBlurStyle">
    <div class="header">
      <span class="title">我的倒计时</span>
      <div class="header-actions">
        <button class="icon-btn" title="新增" @click="startAdding" v-show="!isCollapsed && currentView === 'list'">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
        <button
          class="icon-btn"
          :class="{ active: isLocked }"
          :title="isLocked ? '解锁位置' : '锁定位置'"
          @click="toggleLock"
          v-show="!isCollapsed"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect v-if="isLocked" x="3" y="6" width="8" height="6" rx="1" stroke="currentColor" stroke-width="1.2"/>
            <path v-if="isLocked" d="M5 6V4.5a2 2 0 014 0V6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
            <rect v-if="!isLocked" x="3" y="6" width="8" height="6" rx="1" stroke="currentColor" stroke-width="1.2"/>
            <path v-if="!isLocked" d="M9 6V4.5a2 2 0 00-4 0" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
        </button>
        <button
          class="icon-btn"
          :class="{ active: currentView === 'settings' }"
          title="设置"
          @click="toggleView('settings')"
          v-show="!isCollapsed"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M5.7 1h2.6l.4 1.5a4.5 4.5 0 011.1.6l1.5-.4 1.3 2.3-1.1 1a4.5 4.5 0 010 1.2l1.1 1-1.3 2.3-1.5-.4a4.5 4.5 0 01-1.1.6L8.3 13H5.7l-.4-1.5a4.5 4.5 0 01-1.1-.6l-1.5.4L1.4 9l1.1-1a4.5 4.5 0 010-1.2l-1.1-1 1.3-2.3 1.5.4a4.5 4.5 0 011.1-.6L5.7 1z" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/>
            <circle cx="7" cy="7" r="1.8" stroke="currentColor" stroke-width="1.1"/>
          </svg>
        </button>

        <button class="icon-btn" :title="isCollapsed ? '展开' : '折叠'" @click="toggleCollapse">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path
              :d="isCollapsed ? 'M3 8l4-4 4 4' : 'M3 6l4 4 4-4'"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button class="icon-btn close-btn" title="关闭" @click="closeWindow">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M2 2l10 10M12 2L2 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Body（折叠时隐藏） -->
    <div class="body" v-show="!isCollapsed">
      <!-- 倒计时列表视图 -->
      <div class="list-container" v-if="currentView === 'list'">
        <!-- 新增表单 -->
        <AddItemForm
          v-if="isAdding"
          @confirm="handleAddConfirm"
          @cancel="isAdding = false"
        />

        <!-- 倒计时条目列表 -->
        <CountdownItem
          v-for="item in sortedItems"
          :key="item.id"
          :item="item"
          :tick="tick"
          @update="handleItemUpdate"
          @dismiss="handleDismiss"
          @delete="handleDelete"
        />

        <!-- 空状态 -->
        <div v-if="countdownItems.length === 0 && !isAdding" class="empty-state">
          点击上方 + 添加倒计时
        </div>
      </div>

      <!-- 设置中心面板 -->
      <div class="list-container" v-else-if="currentView === 'settings'">
        <SettingsPanel
          :items="countdownItems"
          :privacy-settings="privacySettings"
          @synced="handleSynced"
          @config-changed="handleSyncConfigChanged"
          @privacy-changed="handlePrivacyChanged"
        />
      </div>
    </div>
    </div><!-- /content-wrapper -->

    <!-- 隐私模式按钮（右下角） -->
    <button
      ref="privacyTriggerBtn"
      v-if="privacySettings.enabled && !isCollapsed && !isPrivacyActive"
      class="privacy-trigger-btn"
      :class="{ pressing: isLongPressing }"
      title="长按启用隐私模式"
      @mousedown.prevent="onPrivacyBtnDown"
      @mouseup="onPrivacyBtnUp"
      @mouseleave="onPrivacyBtnUp"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.8"/>
        <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
      </svg>
    </button>

    <!-- 隐私模式遮罩 -->
    <PrivacyOverlay
      :active="isPrivacyActive"
      :spreading="isLongPressing"
      :spread-progress="spreadProgress"
      :spread-origin="privacySpreadOrigin"
      :settings="privacySettings"
      @close="closeWindow"
      @deactivate="deactivatePrivacy"
      @reveal-progress="handleRevealProgress"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, availableMonitors, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import CountdownItem from './components/CountdownItem.vue';
import AddItemForm from './components/AddItemForm.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import PrivacyOverlay from './components/PrivacyOverlay.vue';
import type { CountdownItemData, PrivacySettings } from './types/countdown';
import { DEFAULT_PRIVACY_SETTINGS } from './types/countdown';

// ========== 常量 ==========

const COLLAPSED_HEIGHT = 52;
const MIN_WIDTH = 280;
const MIN_HEIGHT = 200;
const STORAGE_KEY = 't-countdown-window-state'; // position + size
const LOCK_STORAGE_KEY = 't-countdown-lock-state'; // 锁定状态
const PRIVACY_STORAGE_KEY = 't-countdown-privacy-settings'; // 隐私设置

// ========== 状态 ==========

const isCollapsed = ref(false);
const isAdding = ref(false);
const isLocked = ref(false);
const tick = ref(0);
const currentView = ref<'list' | 'settings'>('list');
let timer: number | null = null;
let stateTimer: number | null = null;
let expandedSize = { width: 320, height: 420 }; // 记忆展开时的大小

const countdownItems = ref<CountdownItemData[]>([]);
let webdavConfigured = false;
let syncDebounceTimer: number | null = null;
let autoSyncTimer: number | null = null;
const AUTO_SYNC_INTERVAL = 1 * 60 * 1000; // 1 分钟定时同步
const SYNC_DEBOUNCE = 500; // 数据变化后 0.5 秒触发同步

// ========== 隐私模式状态 ==========

const privacySettings = ref<PrivacySettings>({ ...DEFAULT_PRIVACY_SETTINGS });
const isPrivacyActive = ref(false);
const isLongPressing = ref(false);
const spreadProgress = ref(0);
const revealProgress = ref(0);
const widgetContainer = ref<HTMLElement | null>(null);
const privacyTriggerBtn = ref<HTMLButtonElement | null>(null);
const privacySpreadOrigin = ref({ x: 0, y: 0 });
const maxBlur = 4; // 最大模糊半径（px）
let longPressRaf: number | null = null;
let longPressStartTime = 0;

// 内容模糊样式：直接对内容 DOM 应用 filter:blur，而非依赖 backdrop-filter
const contentBlurStyle = computed(() => {
  // 隐私模式完全激活时：根据揭示进度平滑消退模糊
  if (isPrivacyActive.value) {
    const blur = maxBlur * (1 - revealProgress.value);
    return { filter: `blur(${blur}px)` };
  }
  // 长按蔓延中：逐渐增加模糊
  if (isLongPressing.value && spreadProgress.value > 0) {
    const blur = spreadProgress.value * maxBlur;
    return { filter: `blur(${blur}px)` };
  }
  return {};
});

// ========== 到期检测 ==========

/** 每次 tick 时自动将到期的 active 条目切换为 expired */
const checkExpired = () => {
  const now = Date.now();
  let changed = false;
  for (const item of countdownItems.value) {
    if (item.status === 'active' && item.targetDate <= now) {
      item.status = 'expired';
      changed = true;
    }
  }
  if (changed) saveData();
};

// ========== 排序：active 按剩余天数升序 → expired → dismissed ==========

const sortedItems = computed(() => {
  void tick.value;
  const now = Date.now();
  const statusOrder: Record<string, number> = { active: 0, expired: 1, dismissed: 2 };
  return [...countdownItems.value].sort((a, b) => {
    const sa = statusOrder[a.status] ?? 3;
    const sb = statusOrder[b.status] ?? 3;
    if (sa !== sb) return sa - sb;
    // 同状态按 targetDate 升序
    return (a.targetDate - now) - (b.targetDate - now);
  });
});

// ========== 操作 ==========

const toggleView = (view: 'settings') => {
  currentView.value = currentView.value === view ? 'list' : view;
  isAdding.value = false;
};

const handleContainerClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (!target.closest('.countdown-item-wrapper') && !target.closest('.header-actions')) {
    document.dispatchEvent(new CustomEvent('close-all-swipes', { detail: { id: null } }));
  }
};

const handleDragStart = async (e: MouseEvent) => {
  if (isLocked.value) return;
  // 不从按钮、输入框等可交互元素发起拖拽
  const target = e.target as HTMLElement;
  if (target.closest('button, input, textarea, select, a, .countdown-item-wrapper, .list-container')) return;
  const appWindow = getCurrentWindow();
  await appWindow.startDragging();
};

const toggleLock = async () => {
  isLocked.value = !isLocked.value;
  const appWindow = getCurrentWindow();
  if (isLocked.value) {
    // 锁定：禁止拉伸
    await appWindow.setResizable(false);
  } else if (!isCollapsed.value) {
    // 解锁且非折叠：恢复拉伸
    await appWindow.setResizable(true);
  }
  localStorage.setItem(LOCK_STORAGE_KEY, JSON.stringify(isLocked.value));
};

const handleSynced = (items: CountdownItemData[]) => {
  countdownItems.value = items;
  saveData();
};

const handleSyncConfigChanged = () => {
  checkWebDavConfig();
};

const toggleCollapse = async () => {
  const appWindow = getCurrentWindow();

  if (!isCollapsed.value) {
    // 折叠前保存当前窗口逻辑尺寸（innerSize 返回物理像素，需除以缩放因子）
    const factor = await appWindow.scaleFactor();
    const size = await appWindow.innerSize();
    expandedSize = { width: size.width / factor, height: size.height / factor };
    isCollapsed.value = true;
    isAdding.value = false;
    currentView.value = 'list';
    await appWindow.setResizable(false);
    await appWindow.setSize(new LogicalSize(expandedSize.width, COLLAPSED_HEIGHT));
  } else {
    // 展开
    isCollapsed.value = false;
    await appWindow.setSize(new LogicalSize(expandedSize.width, expandedSize.height));
    // 解锁状态才允许拉伸
    await appWindow.setResizable(!isLocked.value);
  }
};

const closeWindow = async () => {
  await saveWindowState();
  await saveData();
  const appWindow = getCurrentWindow();
  await appWindow.close();
};

const startAdding = () => {
  isAdding.value = true;
};

const handleAddConfirm = (item: CountdownItemData) => {
  countdownItems.value.push(item);
  isAdding.value = false;
  saveData();
};

const handleItemUpdate = (updated: CountdownItemData) => {
  const idx = countdownItems.value.findIndex(i => i.id === updated.id);
  if (idx !== -1) {
    // 编辑后如果日期变成未来，恢复 active
    if (updated.targetDate > Date.now()) {
      updated.status = 'active';
    }
    countdownItems.value[idx] = updated;
    saveData();
  }
};

const handleDismiss = (id: string) => {
  const item = countdownItems.value.find(i => i.id === id);
  if (item) {
    item.status = 'dismissed';
    saveData();
  }
};

const handleDelete = (id: string) => {
  countdownItems.value = countdownItems.value.filter(i => i.id !== id);
  saveData();
};

// ========== 隐私模式 ==========

const loadPrivacySettings = () => {
  try {
    const saved = localStorage.getItem(PRIVACY_STORAGE_KEY);
    if (saved) {
      const parsed = JSON.parse(saved) as Partial<PrivacySettings>;
      privacySettings.value = { ...DEFAULT_PRIVACY_SETTINGS, ...parsed };
    }
  } catch { /* ignore */ }
};

const savePrivacySettings = () => {
  localStorage.setItem(PRIVACY_STORAGE_KEY, JSON.stringify(privacySettings.value));
};

const handlePrivacyChanged = (settings: PrivacySettings) => {
  privacySettings.value = { ...settings };
};

const updatePrivacySpreadOrigin = (triggerEl?: HTMLElement | null) => {
  const containerEl = widgetContainer.value;
  const sourceEl = triggerEl ?? privacyTriggerBtn.value;
  if (!containerEl || !sourceEl) return;

  const containerRect = containerEl.getBoundingClientRect();
  const triggerRect = sourceEl.getBoundingClientRect();

  privacySpreadOrigin.value = {
    x: triggerRect.left - containerRect.left + triggerRect.width / 2,
    y: triggerRect.top - containerRect.top + triggerRect.height / 2,
  };
};

const onPrivacyBtnDown = (event: MouseEvent) => {
  if (isPrivacyActive.value) return;
  updatePrivacySpreadOrigin(event.currentTarget as HTMLElement | null);
  isLongPressing.value = true;
  spreadProgress.value = 0;
  longPressStartTime = performance.now();

  const duration = privacySettings.value.longPressDuration;

  const animate = (now: number) => {
    if (!isLongPressing.value) return;
    const elapsed = now - longPressStartTime;
    const t = Math.min(elapsed / duration, 1);
    // easeInQuad for accelerating spread
    spreadProgress.value = t * t;

    if (t >= 1) {
      // 长按完成，激活隐私模式
      isLongPressing.value = false;
      spreadProgress.value = 0;
      isPrivacyActive.value = true;
      longPressRaf = null;
      return;
    }

    longPressRaf = requestAnimationFrame(animate);
  };

  longPressRaf = requestAnimationFrame(animate);
};

const onPrivacyBtnUp = () => {
  if (!isLongPressing.value) return;
  // 未达到生效时间，取消
  isLongPressing.value = false;
  spreadProgress.value = 0;
  if (longPressRaf) {
    cancelAnimationFrame(longPressRaf);
    longPressRaf = null;
  }
};

const deactivatePrivacy = () => {
  isPrivacyActive.value = false;
  revealProgress.value = 0;
};

const handleRevealProgress = (progress: number) => {
  revealProgress.value = progress;
};

// ========== 数据持久化（通过 Rust 命令存到 Documents/T-Countdown/data.json） ==========

const loadData = async () => {
  try {
    const json = await invoke<string>('load_data');
    const arr = JSON.parse(json) as CountdownItemData[];
    // 兼容旧数据：没有 status 字段的条目默认 active
    for (const item of arr) {
      if (!item.status) item.status = 'active';
    }
    countdownItems.value = arr;
    checkExpired();
  } catch {
    countdownItems.value = [];
  }
};

const saveData = async () => {
  try {
    await invoke('save_data', { json: JSON.stringify(countdownItems.value) });
  } catch {
    // 忽略保存失败
  }
};

// ========== 云自动同步 ==========

const checkWebDavConfig = async () => {
  try {
    const result = await invoke<[string, string] | null>('load_webdav_config');
    webdavConfigured = !!result;
  } catch {
    webdavConfigured = false;
  }
};

const doCloudSync = async () => {
  if (!webdavConfigured) return;
  try {
    await invoke('webdav_upload', { json: JSON.stringify(countdownItems.value) });
  } catch {
    // 静默失败，不打扰用户
  }
};

const scheduleDebouncedSync = () => {
  if (!webdavConfigured) return;
  if (syncDebounceTimer) clearTimeout(syncDebounceTimer);
  syncDebounceTimer = setTimeout(() => {
    doCloudSync();
    syncDebounceTimer = null;
  }, SYNC_DEBOUNCE) as unknown as number;
};

// ========== 窗口状态记忆（位置 + 大小） ==========

const saveWindowState = async () => {
  try {
    const appWindow = getCurrentWindow();
    const factor = await appWindow.scaleFactor();
    const pos = await appWindow.outerPosition();
    const size = await appWindow.innerSize();
    const state = {
      x: pos.x / factor,
      y: pos.y / factor,
      width: isCollapsed.value ? expandedSize.width : size.width / factor,
      height: isCollapsed.value ? expandedSize.height : size.height / factor,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // 忽略
  }
};

const restoreWindowState = async () => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (!saved) return;
    const { x, y, width, height } = JSON.parse(saved);
    const appWindow = getCurrentWindow();

    if (typeof width === 'number' && typeof height === 'number') {
      const w = Math.max(width, MIN_WIDTH);
      const h = Math.max(height, MIN_HEIGHT);
      expandedSize = { width: w, height: h };
      await appWindow.setSize(new LogicalSize(w, h));
    }

    if (typeof x === 'number' && typeof y === 'number') {
      // 检查保存的位置是否仍在可用显示器范围内
      const monitors = await availableMonitors();
      let isOnScreen = false;
      for (const m of monitors) {
        const mx = m.position.x;
        const my = m.position.y;
        const mw = m.size.width;
        const mh = m.size.height;
        // 窗口至少有 50px 在某个显示器区域内即视为可见
        if (x + 50 > mx && x < mx + mw && y + 50 > my && y < my + mh) {
          isOnScreen = true;
          break;
        }
      }
      if (isOnScreen) {
        await appWindow.setPosition(new LogicalPosition(x, y));
      } else {
        // 显示器已不可用，将窗口移到主显示器中央
        await appWindow.center();
      }
    }

    // 恢复锁定状态
    const lockSaved = localStorage.getItem(LOCK_STORAGE_KEY);
    if (lockSaved) {
      isLocked.value = JSON.parse(lockSaved) === true;
      if (isLocked.value) {
        await appWindow.setResizable(false);
      }
    }
  } catch {
    // 忽略
  }
};

// ========== 自动保存 & 自动同步：数据变更时触发 ==========

watch(countdownItems, () => {
  saveData();
  scheduleDebouncedSync();
}, { deep: true });

watch(privacySettings, () => {
  savePrivacySettings();
}, { deep: true });

// ========== 定时刷新 & 生命周期 ==========

onMounted(async () => {
  await restoreWindowState();
  await loadData();
  await checkWebDavConfig();
  loadPrivacySettings();

  // 每秒刷新倒计时以支持分钟级精度，每 10 秒检测到期
  timer = setInterval(() => {
    tick.value++;
    if (tick.value % 10 === 0) checkExpired();
  }, 1000) as unknown as number;

  // 每 5 秒保存窗口状态
  stateTimer = setInterval(() => {
    saveWindowState();
  }, 5000) as unknown as number;

  // 定时云同步（每 5 分钟）
  autoSyncTimer = setInterval(() => {
    doCloudSync();
  }, AUTO_SYNC_INTERVAL) as unknown as number;
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
  if (stateTimer) clearInterval(stateTimer);
  if (autoSyncTimer) clearInterval(autoSyncTimer);
  if (syncDebounceTimer) clearTimeout(syncDebounceTimer);
  if (longPressRaf) cancelAnimationFrame(longPressRaf);
});
</script>

<style scoped>
.widget-container {
  position: relative;
  width: 100vw;
  height: 100vh;
  padding: 12px;
  background: rgba(20, 20, 20, 0.35);
  border-radius: 6px;
  color: white;
  display: flex;
  flex-direction: column;
  border: 1px solid rgba(255, 255, 255, 0.1);
  overflow: hidden;
}

/* ---- Header ---- */
.content-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
  transition: filter 0.05s linear;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
  cursor: grab;
}

.header:active {
  cursor: grabbing;
}

/* 锁定时禁用拖拽光标 */
.locked .header {
  cursor: default;
}

.locked .header:active {
  cursor: default;
}

.title {
  font-weight: bold;
  font-size: 16px;
}

.header-actions {
  display: flex;
  gap: 6px;
  align-items: center;
}

.icon-btn {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.75);
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
  padding: 0;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.25);
  color: white;
}

.icon-btn.active {
  background: rgba(255, 255, 255, 0.22);
  color: white;
}

.close-btn:hover {
  background: rgba(220, 50, 50, 0.55);
  color: white;
}

/* ---- Body ---- */
.body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  margin-top: 15px;
}

.list-container {
  flex: 1;
  overflow-y: auto;
  scrollbar-width: none;
}

.list-container::-webkit-scrollbar {
  display: none;
}

.empty-state {
  text-align: center;
  padding: 40px 0;
  font-size: 13px;
  opacity: 0.4;
}

/* ---- 隐私模式触发按钮 ---- */
.privacy-trigger-btn {
  position: absolute;
  bottom: 10px;
  right: 10px;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.45);
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: background 0.2s, color 0.2s, transform 0.15s;
  padding: 0;
  z-index: 10;
  user-select: none;
  -webkit-user-select: none;
}

.privacy-trigger-btn:hover {
  background: rgba(255, 255, 255, 0.18);
  color: rgba(255, 255, 255, 0.75);
}

.privacy-trigger-btn.pressing {
  transform: scale(0.9);
  background: rgba(255, 255, 255, 0.22);
  color: white;
}
</style>