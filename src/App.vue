<template>
  <div
    class="widget-container"
    :class="{ collapsed: isCollapsed }"
    data-tauri-drag-region
    @contextmenu.prevent
  >
    <!-- Header -->
    <div class="header" data-tauri-drag-region>
      <span class="title" data-tauri-drag-region>我的倒计时</span>
      <div class="header-actions">
        <button class="icon-btn" title="新增" @click="startAdding" v-show="!isCollapsed && currentView === 'list'">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
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
        <button
          class="icon-btn"
          :class="{ active: currentView === 'sync' }"
          title="云同步"
          @click="toggleView('sync')"
          v-show="!isCollapsed"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M2.5 8.5a4.5 4.5 0 018.2-2.5M11.5 5.5a4.5 4.5 0 01-8.2 2.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M10 5.5h2v-2M4 8.5H2v2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
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

      <!-- 云同步面板 -->
      <div class="list-container" v-else-if="currentView === 'sync'">
        <SyncPanel
          :items="countdownItems"
          @synced="handleSynced"
          @back="currentView = 'list'"
        />
      </div>

      <!-- 设置中心面板 -->
      <div class="list-container" v-else-if="currentView === 'settings'">
        <SettingsPanel />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import CountdownItem from './components/CountdownItem.vue';
import AddItemForm from './components/AddItemForm.vue';
import SyncPanel from './components/SyncPanel.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import type { CountdownItemData } from './types/countdown';

// ========== 常量 ==========

const COLLAPSED_HEIGHT = 52;
const MIN_WIDTH = 280;
const MIN_HEIGHT = 200;
const STORAGE_KEY = 't-countdown-window-state'; // position + size

// ========== 状态 ==========

const isCollapsed = ref(false);
const isAdding = ref(false);
const tick = ref(0);
const currentView = ref<'list' | 'sync' | 'settings'>('list');
let timer: number | null = null;
let stateTimer: number | null = null;
let expandedSize = { width: 320, height: 420 }; // 记忆展开时的大小

const countdownItems = ref<CountdownItemData[]>([]);

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

const toggleView = (view: 'sync' | 'settings') => {
  currentView.value = currentView.value === view ? 'list' : view;
  isAdding.value = false;
};

const handleSynced = (items: CountdownItemData[]) => {
  countdownItems.value = items;
  saveData();
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
    await appWindow.setResizable(true);
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
      await appWindow.setPosition(new LogicalPosition(x, y));
    }
  } catch {
    // 忽略
  }
};

// ========== 自动保存：数据变更时保存 ==========

watch(countdownItems, () => saveData(), { deep: true });

// ========== 定时刷新 & 生命周期 ==========

onMounted(async () => {
  await restoreWindowState();
  await loadData();

  // 每秒刷新倒计时以支持分钟级精度，每 10 秒检测到期
  timer = setInterval(() => {
    tick.value++;
    if (tick.value % 10 === 0) checkExpired();
  }, 1000) as unknown as number;

  // 每 5 秒保存窗口状态
  stateTimer = setInterval(() => {
    saveWindowState();
  }, 5000) as unknown as number;
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
  if (stateTimer) clearInterval(stateTimer);
});
</script>

<style scoped>
.widget-container {
  width: 100vw;
  height: 100vh;
  padding: 15px;
  background: rgba(20, 20, 20, 0.35);
  border-radius: 6px;
  color: white;
  display: flex;
  flex-direction: column;
  border: 1px solid rgba(255, 255, 255, 0.1);
  overflow: hidden;
}

/* ---- Header ---- */
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
</style>