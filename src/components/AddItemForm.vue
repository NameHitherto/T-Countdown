<template>
  <div class="edit-form">
    <!-- 事件名称 -->
    <input
      ref="nameInput"
      v-model="name"
      class="form-input"
      type="text"
      placeholder="事件名称"
      maxlength="20"
      @keyup.enter="handleSubmit"
      @keyup.escape="handleCancel"
    />

    <!-- 天数 / 日期 双向同步 -->
    <div class="date-row">
      <div class="date-field">
        <label class="field-label">剩余天数</label>
        <input
          v-model.number="days"
          class="form-input num-input"
          type="number"
          min="0"
          max="9999"
          @input="onDaysInput"
        />
      </div>
      <span class="sync-icon">⇆</span>
      <div class="date-field">
        <label class="field-label">截止日期</label>
        <input
          v-model="dateStr"
          class="form-input"
          type="date"
          :min="todayStr"
          @input="onDateTimeInput"
        />
        <input
          v-model="timeStr"
          class="form-input time-input"
          type="time"
          @input="onDateTimeInput"
        />
      </div>
    </div>

    <!-- 颜色选择 -->
    <div class="form-colors">
      <div
        v-for="color in PRESET_COLORS"
        :key="color"
        class="color-dot"
        :class="{ active: selectedColor === color }"
        :style="{ backgroundColor: color }"
        @click="selectedColor = color"
      />
    </div>

    <!-- 操作按钮 -->
    <div class="form-actions">
      <button class="btn btn-cancel" @click="handleCancel">取消</button>
      <button class="btn btn-confirm" :disabled="!isValid" @click="handleSubmit">
        {{ isEditMode ? '保存' : '添加' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue';
import type { CountdownItemData } from '../types/countdown';
import { PRESET_COLORS } from '../types/countdown';

const props = defineProps<{
  /** 编辑模式时传入已有数据，新增模式不传 */
  editItem?: CountdownItemData;
}>();

const emit = defineEmits<{
  (e: 'confirm', item: CountdownItemData): void;
  (e: 'cancel'): void;
}>();

const isEditMode = computed(() => !!props.editItem);

// ---- 表单状态 ----

const name = ref('');
const days = ref<number>(1);
const dateStr = ref('');
const timeStr = ref(currentTimeStr());
const selectedColor = ref(PRESET_COLORS[0]);
const nameInput = ref<HTMLInputElement | null>(null);

// 用于阻止循环更新
let syncLock = false;

// ---- 工具方法 ----

const todayStr = computed(() => formatDateStr(new Date()));

function currentTimeStr(): string {
  const now = new Date();
  return `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
}

function formatDateStr(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

function daysFromNow(dateString: string, time: string): number {
  const [h, m] = (time || '00:00').split(':').map(Number);
  const target = new Date(dateString);
  target.setHours(h, m, 0, 0);
  const now = Date.now();
  return Math.max(0, Math.round((target.getTime() - now) / 86400000));
}

function dateTimeFromDays(numDays: number): { date: string; time: string } {
  const d = new Date(Date.now() + numDays * 86400000);
  return {
    date: formatDateStr(d),
    time: `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`,
  };
}

// ---- 天数 → 日期 同步 ----

const onDaysInput = () => {
  if (syncLock) return;
  syncLock = true;
  const d = Math.max(0, Math.floor(days.value || 0));
  days.value = d;
  const result = dateTimeFromDays(d);
  dateStr.value = result.date;
  timeStr.value = result.time;
  nextTick(() => { syncLock = false; });
};

// ---- 日期/时间 → 天数 同步 ----

const onDateTimeInput = () => {
  if (syncLock) return;
  syncLock = true;
  if (dateStr.value) {
    days.value = daysFromNow(dateStr.value, timeStr.value);
  }
  nextTick(() => { syncLock = false; });
};

// ---- 校验 ----

const isValid = computed(() => {
  return name.value.trim().length > 0 && dateStr.value.length > 0;
});

// ---- 提交 ----

const handleSubmit = () => {
  if (!isValid.value) return;

  const target = new Date(dateStr.value);
  const [h, m] = (timeStr.value || '23:59').split(':').map(Number);
  target.setHours(h, m, 0, 0);

  const item: CountdownItemData = {
    id: props.editItem?.id ?? Date.now().toString(),
    name: name.value.trim(),
    targetDate: target.getTime(),
    bgColor: selectedColor.value,
    status: props.editItem?.status ?? 'active',
  };

  emit('confirm', item);
};

const handleCancel = () => {
  emit('cancel');
};

// ---- 初始化（编辑模式回填） ----

const initForm = () => {
  if (props.editItem) {
    const d = new Date(props.editItem.targetDate);
    name.value = props.editItem.name;
    selectedColor.value = props.editItem.bgColor;
    dateStr.value = formatDateStr(d);
    timeStr.value = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
    days.value = daysFromNow(dateStr.value, timeStr.value);
  } else {
    // 新增默认 1 天后
    name.value = '';
    days.value = 1;
    const result = dateTimeFromDays(1);
    dateStr.value = result.date;
    timeStr.value = result.time;
    selectedColor.value = PRESET_COLORS[0];
  }
};

watch(() => props.editItem, () => initForm(), { immediate: true });

onMounted(async () => {
  await nextTick();
  nameInput.value?.focus();
});
</script>

<style scoped>
.edit-form {
  background: rgba(255, 255, 255, 0.08);
  border: 1px dashed rgba(255, 255, 255, 0.25);
  border-radius: 10px;
  padding: 12px 14px;
  margin-bottom: 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-6px); }
  to   { opacity: 1; transform: translateY(0); }
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
  color: rgba(255, 255, 255, 0.4);
}

.form-input:focus {
  border-color: rgba(255, 255, 255, 0.4);
}

.form-input[type="date"]::-webkit-calendar-picker-indicator,
.form-input[type="time"]::-webkit-calendar-picker-indicator {
  filter: invert(1);
  cursor: pointer;
}

.time-input {
  margin-top: 3px;
}

/* 隐藏 number 输入框的上下箭头 */
.num-input::-webkit-outer-spin-button,
.num-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
.num-input {
  -moz-appearance: textfield;
  appearance: textfield;
}

/* ---- 天数 / 日期 行 ---- */

.date-row {
  display: flex;
  align-items: start;
  gap: 8px;
}

.date-field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.field-label {
  font-size: 10px;
  opacity: 0.45;
  padding-left: 2px;
}

.sync-icon {
  font-size: 14px;
  opacity: 0.3;
  padding-top: 21px;
  flex-shrink: 0;
}

/* ---- 颜色 ---- */

.form-colors {
  display: flex;
  gap: 8px;
}

.color-dot {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid transparent;
  transition: border-color 0.15s, transform 0.15s;
}

.color-dot:hover {
  transform: scale(1.15);
}

.color-dot.active {
  border-color: white;
}

/* ---- 按钮 ---- */

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn {
  padding: 4px 14px;
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
</style>