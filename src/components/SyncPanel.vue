<template>
  <div class="sync-panel">
    <!-- 未配置：绑定表单 -->
    <template v-if="!isConfigured">
      <div class="panel-header">
        <span class="panel-title">绑定坚果云</span>
      </div>

      <p class="panel-desc">请先在坚果云官网开启第三方应用密码，再填入下方。</p>

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
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { CountdownItemData } from '../types/countdown';

const props = defineProps<{
  items: CountdownItemData[];
}>();

const emit = defineEmits<{
  (e: 'synced', items: CountdownItemData[]): void;
  (e: 'back'): void;
}>();

// ---- 状态 ----

const isConfigured = ref(false);
const email = ref('');
const appPassword = ref('');
const testing = ref(false);
const syncing = ref<false | 'upload' | 'download'>(false);
const message = ref('');
const messageType = ref<'success' | 'error'>('success');
const savedServer = ref('');
const savedEmail = ref('');

const JIANGUOYUN_SERVER = 'https://dav.jianguoyun.com/dav/';

// ---- 计算属性 ----

const canTest = computed(() => email.value.trim().length > 0 && appPassword.value.length > 0);

const maskedEmail = computed(() => {
  const e = savedEmail.value;
  const at = e.indexOf('@');
  if (at <= 1) return e;
  return e[0] + '***' + e.substring(at);
});

// ---- 初始化 ----

onMounted(async () => {
  try {
    const result = await invoke<[string, string] | null>('load_webdav_config');
    if (result) {
      savedServer.value = result[0];
      savedEmail.value = result[1];
      isConfigured.value = true;
    }
  } catch {
    isConfigured.value = false;
  }
});

// ---- 操作方法 ----

const showMessage = (text: string, type: 'success' | 'error') => {
  message.value = text;
  messageType.value = type;
  setTimeout(() => { message.value = ''; }, 4000);
};

const openJianguoyun = async () => {
  try {
    await openUrl('https://www.jianguoyun.com/d/home#/safety');
  } catch {
    showMessage('无法打开浏览器', 'error');
  }
};

const handleTestAndSave = async () => {
  if (!canTest.value || testing.value) return;
  testing.value = true;
  message.value = '';

  try {
    await invoke('test_webdav', {
      server: JIANGUOYUN_SERVER,
      username: email.value.trim(),
      password: appPassword.value,
    });

    // 测试成功，保存配置
    await invoke('save_webdav_config', {
      server: JIANGUOYUN_SERVER,
      username: email.value.trim(),
      password: appPassword.value,
    });

    savedServer.value = JIANGUOYUN_SERVER;
    savedEmail.value = email.value.trim();
    isConfigured.value = true;
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
    const json = JSON.stringify(props.items);
    await invoke('webdav_upload', { json });
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
    const json = await invoke<string>('webdav_download');
    const remoteItems = JSON.parse(json) as CountdownItemData[];

    // 兼容旧数据
    for (const item of remoteItems) {
      if (!item.status) item.status = 'active';
    }

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
    await invoke('clear_webdav_config');
    isConfigured.value = false;
    savedServer.value = '';
    savedEmail.value = '';
    email.value = '';
    appPassword.value = '';
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
  gap: 12px;
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
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
}

.panel-desc {
  font-size: 11px;
  opacity: 0.55;
  margin: 0;
  line-height: 1.5;
}

.link-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  color: rgba(120, 180, 255, 0.9);
  font-size: 12px;
  cursor: pointer;
  padding: 4px 0;
  transition: color 0.2s;
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
  font-size: 10px;
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
  font-size: 13px;
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
  gap: 8px;
}

.btn {
  padding: 5px 14px;
  border: none;
  border-radius: 8px;
  font-size: 12px;
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
  font-size: 12px;
  opacity: 0.7;
  padding: 6px 10px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
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
  font-size: 12px;
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
  font-size: 11px;
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
  font-size: 11px;
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
</style>
