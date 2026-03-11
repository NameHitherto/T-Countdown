<template>
  <div class="sync-panel">
    <!-- 未配置：绑定表单 -->
    <template v-if="!isConfigured">
      <div class="panel-header">
        <span class="panel-title">绑定坚果云</span>
      </div>

      <p class="panel-desc">请先在坚果云官网开启第三方应用密码，再填入下方。</p>

      <div class="proxy-settings">
        <div class="proxy-row">
          <div class="proxy-info">
            <span class="proxy-title">云同步代理</span>
            <span class="proxy-desc">
              {{ activeProxyPort ? `当前同步使用 HTTP 代理端口 ${activeProxyPort}` : '未使用代理，且不会继承系统或其它代理软件设置' }}
            </span>
          </div>
          <label class="toggle">
            <input type="checkbox" v-model="proxySettings.enabled" @change="onProxyToggle" />
            <span class="slider"></span>
          </label>
        </div>

        <div v-if="proxySettings.enabled" class="proxy-row proxy-row-input">
          <span class="proxy-label">HTTP 代理端口</span>
          <input
            v-model="proxyPortInput"
            class="proxy-input"
            type="text"
            inputmode="numeric"
            maxlength="5"
            placeholder="例如 7890"
            @input="onProxyPortInput"
            @blur="onProxyPortBlur"
          />
        </div>
      </div>

      <button class="link-btn" @click="openJianguoyun">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <path d="M5.5 2H3a1 1 0 00-1 1v8a1 1 0 001 1h8a1 1 0 001-1V8.5M8 2h4v4M6.5 7.5L12 2"
            stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        打开坚果云安全设置
      </button>

      <div class="form-group">
        <label class="field-label">账号（邮箱）</label>
        <input
          v-model="email"
          class="form-input"
          type="email"
          placeholder="your@email.com"
          @keyup.enter="handleTestAndSave"
        />
      </div>

      <div class="form-group">
        <label class="field-label">应用密码</label>
        <input
          v-model="appPassword"
          class="form-input"
          type="password"
          placeholder="第三方应用授权密码"
          @keyup.enter="handleTestAndSave"
        />
      </div>

      <div class="form-actions">
        <button class="btn btn-cancel" @click="$emit('back')">取消</button>
        <button
          class="btn btn-confirm"
          :disabled="!canTest || testing"
          @click="handleTestAndSave"
        >
          {{ testing ? '测试中...' : '测试并保存' }}
        </button>
      </div>

      <div v-if="message" :class="['msg', messageType]">{{ message }}</div>
    </template>

    <!-- 已配置：同步操作 -->
    <template v-else>
      <div class="panel-header">
        <span class="panel-title">坚果云同步</span>
      </div>

      <div class="account-info">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <circle cx="7" cy="5" r="2.5" stroke="currentColor" stroke-width="1.2"/>
          <path d="M2.5 12.5c0-2.5 2-4 4.5-4s4.5 1.5 4.5 4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
        <span>{{ maskedEmail }}</span>
      </div>

      <div class="proxy-settings">
        <div class="proxy-row">
          <div class="proxy-info">
            <span class="proxy-title">云同步代理</span>
            <span class="proxy-desc">
              {{ activeProxyPort ? `当前同步使用 HTTP 代理端口 ${activeProxyPort}` : '未使用代理，且不会继承系统或其它代理软件设置' }}
            </span>
          </div>
          <label class="toggle">
            <input type="checkbox" v-model="proxySettings.enabled" @change="onProxyToggle" />
            <span class="slider"></span>
          </label>
        </div>

        <div v-if="proxySettings.enabled" class="proxy-row proxy-row-input">
          <span class="proxy-label">HTTP 代理端口</span>
          <input
            v-model="proxyPortInput"
            class="proxy-input"
            type="text"
            inputmode="numeric"
            maxlength="5"
            placeholder="例如 7890"
            @input="onProxyPortInput"
            @blur="onProxyPortBlur"
          />
        </div>
      </div>

      <div class="sync-actions">
        <button class="btn btn-sync" :disabled="!!syncing" @click="handleUpload">
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
            <path d="M7 11V3M4 5.5L7 2.5l3 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          {{ syncing === 'upload' ? '上传中...' : '上传到云端' }}
        </button>
        <button class="btn btn-sync" :disabled="!!syncing" @click="handleDownload">
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
            <path d="M7 3v8M4 8.5L7 11.5l3-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          {{ syncing === 'download' ? '下载中...' : '从云端下载' }}
        </button>
      </div>

      <button class="unbind-btn" @click="handleUnbind">解绑账号</button>

      <div v-if="message" :class="['msg', messageType]">{{ message }}</div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { openUrl } from '@tauri-apps/plugin-opener';
import {
  clearWebDavConfig,
  downloadCountdownItems,
  loadWebDavConfig,
  loadWebDavProxyConfig,
  saveWebDavConfig,
  saveWebDavProxyConfig,
  testWebDav,
  uploadCountdownItems,
} from '../services/syncService';
import { getActiveProxyPort } from '../services/proxyService';
import type { CountdownItemData, WebDavProxySettings } from '../types/countdown';
import { DEFAULT_WEBDAV_PROXY_SETTINGS } from '../types/countdown';

const props = defineProps<{
  items: CountdownItemData[];
}>();

const emit = defineEmits<{
  (e: 'synced', items: CountdownItemData[]): void;
  (e: 'back'): void;
  (e: 'config-changed'): void;
}>();

// ---- 状态 ----

const isConfigured = ref(false);
const email = ref('');
const appPassword = ref('');
const testing = ref(false);
const syncing = ref<false | 'upload' | 'download'>(false);
const message = ref('');
const messageType = ref<'success' | 'error'>('success');
const savedEmail = ref('');
const proxySettings = ref<WebDavProxySettings>({ ...DEFAULT_WEBDAV_PROXY_SETTINGS });
const proxyPortInput = ref('');

// ---- 计算属性 ----

const canTest = computed(() => email.value.trim().length > 0 && appPassword.value.length > 0);

const maskedEmail = computed(() => {
  const e = savedEmail.value;
  const at = e.indexOf('@');
  if (at <= 1) return e;
  return e[0] + '***' + e.substring(at);
});

const activeProxyPort = computed(() => getActiveProxyPort(proxySettings.value));

// ---- 初始化 ----

onMounted(async () => {
  try {
    const result = await loadWebDavConfig();
    if (result) {
      savedEmail.value = result[1];
      isConfigured.value = true;
    }
  } catch {
    isConfigured.value = false;
  }

  try {
    const proxyConfig = await loadWebDavProxyConfig();
    proxySettings.value = proxyConfig;
    proxyPortInput.value = proxyConfig.port;
  } catch {
    proxySettings.value = { ...DEFAULT_WEBDAV_PROXY_SETTINGS };
    proxyPortInput.value = '';
  }
});

// ---- 操作方法 ----

const showMessage = (text: string, type: 'success' | 'error') => {
  message.value = text;
  messageType.value = type;
  setTimeout(() => { message.value = ''; }, 4000);
};

const persistProxySettings = async () => {
  await saveWebDavProxyConfig(proxySettings.value);
};

const openJianguoyun = async () => {
  try {
    await openUrl('https://www.jianguoyun.com/d/home#/safety');
  } catch {
    showMessage('无法打开浏览器', 'error');
  }
};

const onProxyToggle = async () => {
  try {
    await persistProxySettings();
  } catch (e: any) {
    showMessage(e?.toString() || '保存代理设置失败', 'error');
  }
};

const onProxyPortInput = () => {
  proxyPortInput.value = proxyPortInput.value.replace(/\D/g, '').slice(0, 5);
  proxySettings.value.port = proxyPortInput.value;
};

const onProxyPortBlur = async () => {
  if (proxyPortInput.value && !activeProxyPort.value) {
    showMessage('请输入 1 到 65535 之间的代理端口', 'error');
  }

  proxySettings.value.port = proxyPortInput.value;

  try {
    await persistProxySettings();
  } catch (e: any) {
    showMessage(e?.toString() || '保存代理设置失败', 'error');
  }
};

const handleTestAndSave = async () => {
  if (!canTest.value || testing.value) return;
  testing.value = true;
  message.value = '';

  try {
    await testWebDav(email.value.trim(), appPassword.value);

    // 测试成功，保存配置
    await saveWebDavConfig(email.value.trim(), appPassword.value);

    savedEmail.value = email.value.trim();
    isConfigured.value = true;
    emit('config-changed');
    showMessage('连接成功，已保存配置', 'success');
  } catch (e: any) {
    showMessage(e?.toString() || '连接失败', 'error');
  } finally {
    testing.value = false;
  }
};

const handleUpload = async () => {
  if (syncing.value) return;
  syncing.value = 'upload';
  message.value = '';

  try {
    await uploadCountdownItems(props.items);
    showMessage('上传成功', 'success');
  } catch (e: any) {
    showMessage(e?.toString() || '上传失败', 'error');
  } finally {
    syncing.value = false;
  }
};

const handleDownload = async () => {
  if (syncing.value) return;
  syncing.value = 'download';
  message.value = '';

  try {
    const remoteItems = await downloadCountdownItems();
    emit('synced', remoteItems);
    showMessage('下载成功，已更新本地数据', 'success');
  } catch (e: any) {
    showMessage(e?.toString() || '下载失败', 'error');
  } finally {
    syncing.value = false;
  }
};

const handleUnbind = async () => {
  try {
    await clearWebDavConfig();
    isConfigured.value = false;
    savedEmail.value = '';
    email.value = '';
    appPassword.value = '';
    emit('config-changed');
    showMessage('已解绑账号', 'success');
  } catch (e: any) {
    showMessage(e?.toString() || '解绑失败', 'error');
  }
};
</script>

<style scoped>
.sync-panel {
  display: flex;
  flex-direction: column;
  gap: clamp(10px, 4vw, 12px);
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
  gap: 6px;
  min-width: 0;
}

.panel-title {
  font-size: clamp(13px, 4.4vw, 14px);
  font-weight: 600;
}

.panel-desc {
  font-size: clamp(10px, 3.6vw, 11px);
  opacity: 0.55;
  margin: 0;
  line-height: 1.5;
  overflow-wrap: anywhere;
}

.proxy-settings {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.proxy-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: clamp(8px, 3vw, 10px);
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
}

.proxy-row-input {
  padding-top: 10px;
  padding-bottom: 10px;
}

.proxy-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.proxy-title {
  font-size: clamp(11px, 3.8vw, 12px);
  font-weight: 500;
}

.proxy-desc,
.proxy-label {
  font-size: clamp(10px, 3.4vw, 11px);
  opacity: 0.5;
  overflow-wrap: anywhere;
}

.proxy-input {
  width: min(100%, 96px);
  min-width: 0;
  padding: 6px 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  color: white;
  font-size: clamp(11px, 3.8vw, 12px);
  text-align: right;
}

.proxy-input::placeholder {
  color: rgba(255, 255, 255, 0.28);
}

.proxy-input:focus {
  border-color: rgba(120, 180, 255, 0.6);
  outline: none;
}

.toggle {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  inset: 0;
  cursor: pointer;
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
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.7);
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.toggle input:checked + .slider {
  background: rgba(45, 106, 79, 0.7);
}

.toggle input:checked + .slider::before {
  transform: translateX(16px);
  background: white;
}

.link-btn {
  display: inline-flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  background: none;
  border: none;
  color: rgba(120, 180, 255, 0.9);
  font-size: clamp(11px, 3.8vw, 12px);
  cursor: pointer;
  padding: 4px 0;
  transition: color 0.2s;
  text-align: left;
}

.link-btn:hover {
  color: rgba(160, 200, 255, 1);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.field-label {
  font-size: clamp(10px, 3.4vw, 11px);
  opacity: 0.45;
  padding-left: 2px;
}

.form-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  color: white;
  font-size: clamp(12px, 4vw, 13px);
  transition: border-color 0.2s;
}

.form-input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.form-input:focus {
  border-color: rgba(255, 255, 255, 0.4);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 8px;
}

.btn {
  padding: 5px 14px;
  border: none;
  border-radius: 8px;
  font-size: clamp(11px, 3.8vw, 12px);
  cursor: pointer;
  transition: background 0.2s;
}

.btn-cancel {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
}

.btn-cancel:hover {
  background: rgba(255, 255, 255, 0.18);
}

.btn-confirm {
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.btn-confirm:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.35);
}

.btn-confirm:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

/* ---- 已配置 ---- */

.account-info {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: clamp(11px, 3.8vw, 12px);
  opacity: 0.7;
  padding: 6px 10px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  overflow-wrap: anywhere;
}

.sync-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.btn-sync {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 8px 14px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.85);
  font-size: clamp(11px, 3.8vw, 12px);
  cursor: pointer;
  transition: background 0.2s, border-color 0.2s;
}

.btn-sync:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.25);
}

.btn-sync:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.unbind-btn {
  background: none;
  border: none;
  color: rgba(255, 120, 120, 0.7);
  font-size: clamp(10px, 3.6vw, 11px);
  cursor: pointer;
  padding: 4px 0;
  text-align: left;
  transition: color 0.2s;
}

.unbind-btn:hover {
  color: rgba(255, 120, 120, 1);
}

/* ---- 消息 ---- */

.msg {
  font-size: clamp(10px, 3.6vw, 11px);
  padding: 6px 10px;
  border-radius: 6px;
  animation: fadeIn 0.15s ease;
}

.msg.success {
  background: rgba(45, 106, 79, 0.25);
  color: rgba(130, 220, 170, 0.95);
}

.msg.error {
  background: rgba(220, 50, 50, 0.15);
  color: rgba(255, 150, 150, 0.95);
}

@media (max-width: 300px) {
  .proxy-row,
  .account-info,
  .form-actions {
    align-items: stretch;
    flex-direction: column;
  }

  .toggle,
  .proxy-input,
  .btn,
  .link-btn {
    align-self: stretch;
  }

  .proxy-input,
  .btn {
    width: 100%;
    max-width: none;
    text-align: left;
  }
}
</style>
