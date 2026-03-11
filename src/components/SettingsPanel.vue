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

      <!-- 隐私模式 -->
      <div class="setting-item">
        <div class="setting-info">
          <span class="setting-name">隐私模式</span>
          <span class="setting-desc">快速遮蔽窗口内容</span>
        </div>
        <label class="toggle">
          <input
            type="checkbox"
            v-model="privacySettings.enabled"
            @change="onPrivacyChange"
          />
          <span class="slider"></span>
        </label>
      </div>

      <!-- 隐私模式子设置 -->
      <div v-if="privacySettings.enabled" class="setting-item clickable" @click="togglePrivacyPanel">
        <div class="setting-info">
          <span class="setting-name">隐私设置</span>
          <span class="setting-desc">自定义长按时间与遮罩样式</span>
        </div>
        <svg class="chevron" :class="{ expanded: showPrivacy }" width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M5 3l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>

      <!-- 云同步 -->
      <div class="setting-item clickable" @click="toggleSyncPanel">
        <div class="setting-info">
          <span class="setting-name">云同步</span>
          <span class="setting-desc">通过坚果云 WebDAV 同步数据</span>
        </div>
        <svg class="chevron" :class="{ expanded: showSync }" width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M5 3l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>

      <!-- 检查更新 -->
      <div class="setting-item clickable" @click="toggleUpdatePanel">
        <div class="setting-info">
          <span class="setting-name">检查更新</span>
          <span class="setting-desc">检查新版本并下载安装</span>
        </div>
        <svg class="chevron" :class="{ expanded: showUpdate }" width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M5 3l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>

      <!-- 关于 -->
      <div class="setting-item clickable" @click="toggleAbout">
        <div class="setting-info">
          <span class="setting-name">关于</span>
          <span class="setting-desc">版本信息与作者信息</span>
        </div>
        <svg class="chevron" :class="{ expanded: showAbout }" width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M5 3l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
    </div>

    <!-- 隐私设置面板 -->
    <div v-if="showPrivacy" class="privacy-section">
      <div class="privacy-settings">
        <!-- 长按生效时间 -->
        <div class="privacy-row">
          <span class="privacy-label">长按生效时间</span>
          <div class="duration-control">
            <button class="dur-btn" @click="adjustDuration(-100)" :disabled="privacySettings.longPressDuration <= 300">−</button>
            <span class="dur-value">{{ (privacySettings.longPressDuration / 1000).toFixed(1) }}s</span>
            <button class="dur-btn" @click="adjustDuration(100)" :disabled="privacySettings.longPressDuration >= 5000">+</button>
          </div>
        </div>

        <!-- 遮罩模式 -->
        <div class="privacy-row">
          <span class="privacy-label">遮罩模式</span>
          <div class="mask-mode-group">
            <button
              class="mask-mode-btn"
              :class="{ active: privacySettings.maskMode === 'blur' }"
              @click="setMaskMode('blur')"
            >毛玻璃</button>
            <button
              class="mask-mode-btn"
              :class="{ active: privacySettings.maskMode === 'image' }"
              @click="setMaskMode('image')"
            >图片</button>
          </div>
        </div>

        <!-- 自定义图片 -->
        <div v-if="privacySettings.maskMode === 'image'" class="privacy-row image-row">
          <span class="privacy-label">遮罩图片</span>
          <div class="image-upload-area">
            <div
              v-if="privacySettings.maskImage"
              class="image-preview"
              :style="{ backgroundImage: `url(${privacySettings.maskImage})` }"
              @click="pickImage"
            >
              <div class="image-overlay-hint">点击更换</div>
            </div>
            <button v-else class="pick-image-btn" @click="pickImage">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor" stroke-width="1.5"/>
                <path d="M3 16l5-5 4 4 3-3 6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                <circle cx="8.5" cy="8.5" r="1.5" stroke="currentColor" stroke-width="1.5"/>
              </svg>
              选择图片
            </button>
          </div>
        </div>
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

    <!-- 检查更新面板 -->
    <div v-if="showUpdate" class="update-section">
      <div class="proxy-card">
        <div class="proxy-row">
          <div class="setting-info">
            <span class="setting-name">更新代理</span>
            <span class="setting-desc">
              {{ activeProxyPort ? `当前更新使用 HTTP 代理端口 ${activeProxyPort}` : '未使用代理，且不会继承系统或其它代理软件设置' }}
            </span>
          </div>
          <label class="toggle">
            <input
              type="checkbox"
              v-model="updateProxySettings.enabled"
              @change="onUpdateProxyToggle"
            />
            <span class="slider"></span>
          </label>
        </div>

        <div v-if="updateProxySettings.enabled" class="proxy-row">
          <span class="proxy-label">HTTP 代理端口</span>
          <input
            v-model="updateProxyPortInput"
            class="proxy-input"
            type="text"
            inputmode="numeric"
            maxlength="5"
            placeholder="例如 7890"
            @input="onUpdateProxyPortInput"
            @blur="onUpdateProxyPortBlur"
          />
        </div>
      </div>

      <div class="update-area">
        <div v-if="updateChecking" class="update-status">
          <span class="update-checking">正在检查更新...</span>
        </div>
        <div v-else-if="updateError" class="update-status">
          <span class="update-error">{{ updateError }}</span>
          <button class="btn btn-retry" @click="checkForUpdate">重试</button>
        </div>
        <div v-else-if="updateAvailable" class="update-status">
          <span class="update-new">发现新版本 v{{ updateVersion }}</span>
          <button v-if="!updateInstalling" class="btn btn-update" @click="downloadAndInstall">下载更新</button>
          <span v-else class="update-installing">{{ updateProgress }}</span>
        </div>
        <div v-else class="update-status">
          <span class="update-latest">已是最新版本</span>
          <button class="btn btn-retry" @click="checkForUpdate">重新检查</button>
        </div>
      </div>
    </div>

    <!-- 关于面板 -->
    <div v-if="showAbout" class="about-section">
      <div class="about-info">
        <div class="about-row">
          <span class="about-label">软件名称</span>
          <span class="about-value">T-Countdown</span>
        </div>
        <div class="about-row">
          <span class="about-label">当前版本</span>
          <span class="about-value">v{{ appVersion }}</span>
        </div>
        <div class="about-row">
          <span class="about-label">作者</span>
          <span class="about-value">NameHitherto</span>
        </div>
      </div>
    </div>

    <!-- 内联提示 Toast -->
    <Transition name="toast">
      <div v-if="toastMsg" class="inline-toast">
        {{ toastMsg }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue';
import SyncPanel from './SyncPanel.vue';
import { getAutostartStatus, setAutostartStatus } from '../services/systemService';
import {
  checkForAppUpdate,
  downloadAndInstallUpdate,
  getAppVersion,
  type AppUpdateInfo,
} from '../services/updateService';
import { getActiveProxyPort } from '../services/proxyService';
import { loadUpdateProxySettings, saveUpdateProxySettings } from '../services/storageService';
import type {
  CountdownItemData,
  PrivacySettings,
  PrivacyMaskMode,
  UpdateProxySettings,
} from '../types/countdown';
import { DEFAULT_PRIVACY_SETTINGS, DEFAULT_UPDATE_PROXY_SETTINGS } from '../types/countdown';

const props = defineProps<{
  items: CountdownItemData[];
  privacySettings: PrivacySettings;
}>();

const emit = defineEmits<{
  (e: 'synced', items: CountdownItemData[]): void;
  (e: 'config-changed'): void;
  (e: 'privacy-changed', settings: PrivacySettings): void;
}>();

const showSync = ref(false);
const showUpdate = ref(false);
const showAbout = ref(false);
const showPrivacy = ref(false);
const autostart = ref(false);
const loading = ref(true);
const toastMsg = ref('');
let toastTimer: number | null = null;

const showToast = (msg: string, duration = 2500) => {
  toastMsg.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toastMsg.value = '';
    toastTimer = null;
  }, duration) as unknown as number;
};

// ---- 关于 & 更新 ----

const appVersion = ref('');
const updateChecking = ref(false);
const updateAvailable = ref(false);
const updateVersion = ref('');
const updateError = ref('');
const updateInstalling = ref(false);
const updateProgress = ref('正在下载...');
let pendingUpdate: AppUpdateInfo | null = null;

// ---- 更新代理设置 ----

const updateProxySettings = ref<UpdateProxySettings>({ ...DEFAULT_UPDATE_PROXY_SETTINGS });
const updateProxyPortInput = ref('');
const activeProxyPort = computed(() => getActiveProxyPort(updateProxySettings.value));

// ---- 隐私模式设置 ----

const privacySettings = ref<PrivacySettings>({ ...DEFAULT_PRIVACY_SETTINGS });

const emitPrivacyChanged = () => {
  emit('privacy-changed', { ...privacySettings.value });
};

const onPrivacyChange = () => {
  emitPrivacyChanged();
};

const togglePrivacyPanel = () => {
  showPrivacy.value = !showPrivacy.value;
  showSync.value = false;
  showUpdate.value = false;
  showAbout.value = false;
};

const toggleSyncPanel = () => {
  showSync.value = !showSync.value;
  showPrivacy.value = false;
  showUpdate.value = false;
  showAbout.value = false;
};

const toggleUpdatePanel = () => {
  showUpdate.value = !showUpdate.value;
  showSync.value = false;
  showPrivacy.value = false;
  showAbout.value = false;
  if (showUpdate.value) {
    void checkForUpdate();
  }
};

const adjustDuration = (delta: number) => {
  const v = privacySettings.value.longPressDuration + delta;
  privacySettings.value.longPressDuration = Math.max(300, Math.min(5000, v));
  emitPrivacyChanged();
};

const setMaskMode = (mode: PrivacyMaskMode) => {
  privacySettings.value.maskMode = mode;
  emitPrivacyChanged();
};

const pickImage = () => {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = 'image/*';
  input.onchange = () => {
    const file = input.files?.[0];
    if (!file) return;
    // 限制 2MB
    if (file.size > 2 * 1024 * 1024) {
      showToast('图片大小不能超过 2MB');
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      privacySettings.value.maskImage = reader.result as string;
      emitPrivacyChanged();
    };
    reader.readAsDataURL(file);
  };
  input.click();
};

const toggleAbout = () => {
  showAbout.value = !showAbout.value;
  showSync.value = false;
  showUpdate.value = false;
  showPrivacy.value = false;
};

const checkForUpdate = async () => {
  updateChecking.value = true;
  updateAvailable.value = false;
  updateError.value = '';
  updateVersion.value = '';
  pendingUpdate = null;

  try {
    const update = await checkForAppUpdate(updateProxySettings.value);
    if (update) {
      updateAvailable.value = true;
      updateVersion.value = update.version;
      pendingUpdate = update;
    }
  } catch (e: any) {
    updateError.value = '检查更新失败';
    console.error('检查更新失败:', e);
  } finally {
    updateChecking.value = false;
  }
};

const downloadAndInstall = async () => {
  if (!pendingUpdate) return;
  updateInstalling.value = true;
  updateProgress.value = '正在下载...';

  try {
    await downloadAndInstallUpdate(updateProxySettings.value, (message) => {
      updateProgress.value = message;
    });

    // 安装完成后需要重启
    updateProgress.value = '更新完成，即将重启...';
  } catch (e: any) {
    updateInstalling.value = false;
    updateError.value = '更新失败，请稍后重试';
    updateAvailable.value = false;
    console.error('更新失败:', e);
  }
};

onMounted(async () => {
  const savedProxySettings = loadUpdateProxySettings();
  updateProxySettings.value = savedProxySettings;
  updateProxyPortInput.value = savedProxySettings.port;

  try {
    appVersion.value = await getAppVersion();
  } catch {
    appVersion.value = '未知';
  }
  try {
    autostart.value = await getAutostartStatus();
  } catch {
    autostart.value = false;
  }
  loading.value = false;
});

watch(() => props.privacySettings, (settings) => {
  privacySettings.value = { ...DEFAULT_PRIVACY_SETTINGS, ...settings };
}, { deep: true, immediate: true });

const onAutostartChange = async () => {
  const target = autostart.value;
  try {
    await setAutostartStatus(target);
  } catch (e) {
    // 设置失败，回滚状态
    autostart.value = !target;
    console.error('设置开机自启失败:', e);
  }
};

const persistUpdateProxySettings = () => {
  saveUpdateProxySettings(updateProxySettings.value);
};

const onUpdateProxyToggle = () => {
  if (!updateProxySettings.value.enabled) {
    updateProxySettings.value.port = '';
    updateProxyPortInput.value = '';
  }

  persistUpdateProxySettings();
};

const onUpdateProxyPortInput = () => {
  updateProxyPortInput.value = updateProxyPortInput.value.replace(/\D/g, '').slice(0, 5);
  updateProxySettings.value.port = updateProxyPortInput.value;
  persistUpdateProxySettings();
};

const onUpdateProxyPortBlur = () => {
  const currentPort = getActiveProxyPort({
    ...updateProxySettings.value,
    enabled: true,
  });

  if (updateProxyPortInput.value && !currentPort) {
    showToast('请输入 1 到 65535 之间的代理端口');
  }

  updateProxySettings.value.port = updateProxyPortInput.value;
  persistUpdateProxySettings();
};
</script>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: clamp(10px, 4vw, 14px);
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
  min-width: 0;
}

.panel-title {
  font-size: clamp(13px, 4.4vw, 14px);
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
  gap: 12px;
  padding: clamp(8px, 3vw, 10px) clamp(10px, 3.6vw, 12px);
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
  flex: 1;
  min-width: 0;
}

.setting-name {
  font-size: clamp(12px, 4vw, 13px);
  font-weight: 500;
  line-height: 1.25;
}

.setting-desc {
  font-size: clamp(10px, 3.4vw, 11px);
  opacity: 0.45;
  line-height: 1.35;
  overflow-wrap: anywhere;
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

.update-section {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  animation: fadeIn 0.15s ease;
}

.proxy-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.proxy-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 6px;
}

.proxy-row + .proxy-row {
  margin-top: 0;
}

.proxy-label {
  font-size: clamp(11px, 3.8vw, 12px);
  opacity: 0.7;
  min-width: 0;
}

.proxy-input {
  width: min(100%, 96px);
  min-width: 0;
  padding: 6px 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  color: white;
  font-size: clamp(11px, 3.8vw, 12px);
  text-align: right;
  outline: none;
}

.proxy-input::placeholder {
  color: rgba(255, 255, 255, 0.28);
}

.proxy-input:focus {
  border-color: rgba(120, 180, 255, 0.6);
}

/* ---- 关于区域 ---- */

.about-section {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  animation: fadeIn 0.15s ease;
}

.about-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.about-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 6px;
}

.about-label {
  font-size: clamp(11px, 3.8vw, 12px);
  opacity: 0.5;
}

.about-value {
  font-size: clamp(11px, 3.8vw, 12px);
  font-weight: 500;
  min-width: 0;
  text-align: right;
  overflow-wrap: anywhere;
}

.update-area {
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 8px;
}

.update-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  font-size: clamp(11px, 3.8vw, 12px);
}

.update-checking {
  opacity: 0.5;
}

.update-latest {
  color: rgba(120, 200, 140, 0.85);
}

.update-new {
  color: rgba(120, 180, 255, 0.9);
  font-weight: 500;
}

.update-error {
  color: rgba(255, 120, 120, 0.85);
}

.update-installing {
  opacity: 0.6;
}

.btn-update,
.btn-retry {
  padding: 4px 12px;
  border: none;
  border-radius: 6px;
  font-size: clamp(10px, 3.6vw, 11px);
  cursor: pointer;
  transition: background 0.15s;
  flex-shrink: 0;
}

.btn-update {
  background: rgba(45, 106, 79, 0.6);
  color: white;
}

.btn-update:hover {
  background: rgba(45, 106, 79, 0.85);
}

.btn-retry {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.75);
}

.btn-retry:hover {
  background: rgba(255, 255, 255, 0.18);
}

/* ---- 隐私设置区域 ---- */

.privacy-section {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  padding-top: 12px;
  animation: fadeIn 0.15s ease;
}

.privacy-settings {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.privacy-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 6px;
}

.privacy-row.image-row {
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
}

.privacy-label {
  font-size: clamp(11px, 3.8vw, 12px);
  opacity: 0.7;
  flex-shrink: 0;
}

/* 时长控制 */
.duration-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dur-btn {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s;
  padding: 0;
}

.dur-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.dur-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.dur-value {
  font-size: clamp(11px, 3.8vw, 12px);
  font-weight: 500;
  min-width: 36px;
  text-align: center;
}

/* 遮罩模式选择 */
.mask-mode-group {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 4px;
}

.mask-mode-btn {
  padding: 4px 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  background: transparent;
  color: rgba(255, 255, 255, 0.6);
  font-size: clamp(10px, 3.6vw, 11px);
  cursor: pointer;
  transition: all 0.15s;
}

.mask-mode-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.8);
}

.mask-mode-btn.active {
  background: rgba(45, 106, 79, 0.5);
  border-color: rgba(45, 106, 79, 0.8);
  color: white;
}

/* 图片上传 */
.image-upload-area {
  width: 100%;
}

.image-preview {
  width: 100%;
  height: 80px;
  background-size: cover;
  background-position: center;
  border-radius: 6px;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.image-overlay-hint {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  opacity: 0;
  transition: opacity 0.2s;
  font-size: clamp(10px, 3.6vw, 11px);
  color: white;
}

.image-preview:hover .image-overlay-hint {
  opacity: 1;
}

.pick-image-btn {
  width: 100%;
  padding: 12px;
  border: 1px dashed rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.03);
  color: rgba(255, 255, 255, 0.5);
  font-size: clamp(11px, 3.8vw, 12px);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: all 0.15s;
}

.pick-image-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.35);
  color: rgba(255, 255, 255, 0.75);
}

@media (max-width: 300px) {
  .setting-item,
  .proxy-row,
  .privacy-row,
  .about-row,
  .update-status {
    align-items: stretch;
    flex-direction: column;
  }

  .toggle,
  .chevron,
  .proxy-input,
  .duration-control,
  .mask-mode-group,
  .btn-update,
  .btn-retry {
    align-self: stretch;
  }

  .proxy-input,
  .btn-update,
  .btn-retry {
    width: 100%;
    max-width: none;
    text-align: left;
  }

  .duration-control,
  .mask-mode-group {
    justify-content: space-between;
  }

  .about-value {
    text-align: left;
  }
}

/* ---- 内联 Toast ---- */
.inline-toast {
  position: fixed;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(220, 60, 60, 0.85);
  color: white;
  font-size: 12px;
  padding: 6px 16px;
  border-radius: 6px;
  z-index: 9999;
  pointer-events: none;
  white-space: nowrap;
}

.toast-enter-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.toast-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(8px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-4px);
}
</style>
