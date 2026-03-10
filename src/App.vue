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
        <button class="icon-btn close-btn" title="关闭" @click="handleCloseWindow">
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
          @confirm="handleAdd"
          @cancel="stopAdding"
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
      @close="handleCloseWindow"
      @deactivate="deactivatePrivacy"
      @reveal-progress="handleRevealProgress"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import CountdownItem from './components/CountdownItem.vue';
import AddItemForm from './components/AddItemForm.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import PrivacyOverlay from './components/PrivacyOverlay.vue';
import { useCountdownItems } from './composables/useCountdownItems';
import { usePrivacyMode } from './composables/usePrivacyMode';
import { useWindowState } from './composables/useWindowState';
import type { CountdownItemData } from './types/countdown';

const {
  isCollapsed,
  isAdding,
  isLocked,
  currentView,
  toggleView,
  handleContainerClick,
  handleDragStart,
  toggleLock,
  restoreWindowState,
  toggleCollapse,
  closeWindow,
  startAdding,
  stopAdding,
  startPersistence,
  stopPersistence,
} = useWindowState();

const {
  countdownItems,
  tick,
  sortedItems,
  loadData,
  persistData,
  refreshSyncConfig,
  startRuntime,
  stopRuntime,
  handleAddConfirm,
  handleItemUpdate,
  handleDismiss,
  handleDelete,
  handleSynced,
} = useCountdownItems();

const {
  privacySettings,
  isPrivacyActive,
  isLongPressing,
  spreadProgress,
  widgetContainer,
  privacyTriggerBtn,
  privacySpreadOrigin,
  contentBlurStyle,
  handlePrivacyChanged,
  onPrivacyBtnDown,
  onPrivacyBtnUp,
  deactivatePrivacy,
  handleRevealProgress,
  stopRuntime: stopPrivacyRuntime,
} = usePrivacyMode();

void widgetContainer;
void privacyTriggerBtn;

const handleAdd = (item: CountdownItemData) => {
  handleAddConfirm(item);
  stopAdding();
};

const handleSyncConfigChanged = () => {
  void refreshSyncConfig();
};

const handleCloseWindow = async () => {
  await closeWindow(persistData);
};

onMounted(async () => {
  await restoreWindowState();
  await loadData();
  await refreshSyncConfig();
  startRuntime();
  startPersistence();
});

onUnmounted(() => {
  stopRuntime();
  stopPersistence();
  stopPrivacyRuntime();
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