<template>
  <div class="settings-panel">
    <div class="panel-header">
      <span class="panel-title">设置中心</span>
    </div>

    <div class="settings-list">
      <!-- 开机自启 -->
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-name">开机自启</span>
          <span class="setting-desc">Windows 启动时自动打开此工具</span>
        </div>
        <label class="toggle" :class="{ disabled: loading }">
          <input
            type="checkbox"
            v-model="autostart"
            :disabled="loading"
            @change="onAutostartChange"
          />
          <span class="slider"></span>
        </label>
      </div>

      <!-- 云同步 -->
      <div class="setting-item clickable" @click="showSync = !showSync">
        <div class="setting-info">
          <span class="setting-name">云同步</span>
          <span class="setting-desc">通过坚果云 WebDAV 同步数据</span>
        </div>
        <svg class="chevron" :class="{ expanded: showSync }" width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M5 3l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
    </div>

    <!-- 云同步面板 -->
    <div v-if="showSync" class="sync-section">
      <SyncPanel
        :items="items"
        @synced="$emit('synced', $event)"
        @back="showSync = false"
        @config-changed="$emit('config-changed')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import SyncPanel from './SyncPanel.vue';
import type { CountdownItemData } from '../types/countdown';

defineProps<{
  items: CountdownItemData[];
}>();

defineEmits<{
  (e: 'synced', items: CountdownItemData[]): void;
  (e: 'config-changed'): void;
}>();

const showSync = ref(false);
const autostart = ref(false);
const loading = ref(true);

onMounted(async () => {
  try {
    autostart.value = await invoke<boolean>('get_autostart');
  } catch {
    autostart.value = false;
  }
  loading.value = false;
});

const onAutostartChange = async () => {
  const target = autostart.value;
  try {
    await invoke('set_autostart', { enable: target });
  } catch (e) {
    // 设置失败，回滚状态
    autostart.value = !target;
    console.error('设置开机自启失败:', e);
  }
};
</script>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 4px 2px;
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-6px); }
  to   { opacity: 1; transform: translateY(0); }
}

.panel-header {
  display: flex;
  align-items: center;
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
}

.settings-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  transition: background 0.15s;
}

.setting-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-name {
  font-size: 13px;
  font-weight: 500;
}

.setting-desc {
  font-size: 10px;
  opacity: 0.45;
}

/* ---- Toggle 开关 ---- */

.toggle {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
}

.toggle.disabled {
  opacity: 0.4;
  pointer-events: none;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 20px;
  transition: background 0.25s;
}

.slider::before {
  content: "";
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  bottom: 2px;
  background: rgba(255, 255, 255, 0.7);
  border-radius: 50%;
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.toggle input:checked + .slider {
  background: rgba(45, 106, 79, 0.7);
}

.toggle input:checked + .slider::before {
  transform: translateX(16px);
  background: white;
}

/* ---- 可点击设置项 ---- */

.setting-item.clickable {
  cursor: pointer;
}

.chevron {
  color: rgba(255, 255, 255, 0.4);
  transition: transform 0.2s;
  flex-shrink: 0;
}

.chevron.expanded {
  transform: rotate(90deg);
}

/* ---- 云同步区域 ---- */

.sync-section {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 12px;
}
</style>
