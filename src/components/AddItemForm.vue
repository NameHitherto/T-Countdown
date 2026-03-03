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
        <VueDatePicker
          v-model="pickerDate"
          :min-date="new Date()"
          :enable-time-picker="true"
          :dark="true"
          :teleport="true"
          auto-apply
          :locale="zhCN"
          :format="formatDisplay"
          position="left"
          :offset="4"
          text-input
          six-weeks="fair"
          @update:model-value="onPickerChange"
        >
          <template #trigger>
            <button class="picker-trigger" type="button">
              <span class="picker-text">{{ displayDateStr }}</span>
              <svg class="picker-icon" width="14" height="14" viewBox="0 0 14 14" fill="none">
                <rect x="1.5" y="2.5" width="11" height="9" rx="1.5" stroke="currentColor" stroke-width="1.1"/>
                <path d="M1.5 5.5h11" stroke="currentColor" stroke-width="1.1"/>
                <path d="M4.5 1v2M9.5 1v2" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/>
              </svg>
            </button>
          </template>
        </VueDatePicker>
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
import { VueDatePicker } from '@vuepic/vue-datepicker';
import '@vuepic/vue-datepicker/dist/main.css';
import { zhCN } from 'date-fns/locale';
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
const pickerDate = ref<Date>(new Date(Date.now() + 86400000));
const selectedColor = ref(PRESET_COLORS[0]);
const nameInput = ref<HTMLInputElement | null>(null);

// 用于阻止循环更新
let syncLock = false;

// ---- 工具方法 ----

function daysFromNow(target: Date): number {
  return Math.max(0, Math.round((target.getTime() - Date.now()) / 86400000));
}

function dateFromDays(numDays: number): Date {
  return new Date(Date.now() + numDays * 86400000);
}

/** 格式化日期用于 datepicker 显示 */
function formatDisplay(date: Date): string {
  if (!date) return '';
  const y = date.getFullYear();
  const M = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  const h = String(date.getHours()).padStart(2, '0');
  const m = String(date.getMinutes()).padStart(2, '0');
  return `${y}-${M}-${d} ${h}:${m}`;
}

/** 用于触发按钮的显示文本 */
const displayDateStr = computed(() => {
  return formatDisplay(pickerDate.value);
});

// ---- 天数 → 日期 同步 ----

const onDaysInput = () => {
  if (syncLock) return;
  syncLock = true;
  const d = Math.max(0, Math.floor(days.value || 0));
  days.value = d;
  pickerDate.value = dateFromDays(d);
  nextTick(() => { syncLock = false; });
};

// ---- 日期 → 天数 同步 ----

const onPickerChange = (val: Date | null) => {
  if (syncLock || !val) return;
  syncLock = true;
  pickerDate.value = val;
  days.value = daysFromNow(val);
  nextTick(() => { syncLock = false; });
};

// ---- 校验 ----

const isValid = computed(() => {
  return name.value.trim().length > 0 && pickerDate.value != null;
});

// ---- 提交 ----

const handleSubmit = () => {
  if (!isValid.value) return;

  const target = new Date(pickerDate.value);
  target.setSeconds(0, 0);

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
    pickerDate.value = d;
    days.value = daysFromNow(d);
  } else {
    // 新增默认 1 天后
    name.value = '';
    days.value = 1;
    pickerDate.value = dateFromDays(1);
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

/* ---- 日期选择器触发按钮 ---- */

.picker-trigger {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  color: white;
  font-size: 13px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  transition: border-color 0.2s;
  text-align: left;
}

.picker-trigger:hover {
  border-color: rgba(255, 255, 255, 0.4);
}

.picker-text {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.picker-icon {
  opacity: 0.5;
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

<!-- VueDatePicker 全局样式覆盖（teleport 到 body，不能用 scoped） -->
<style>
.dp__theme_dark {
  --dp-background-color: rgba(30, 30, 30, 0.95);
  --dp-text-color: #e0e0e0;
  --dp-hover-color: rgba(255, 255, 255, 0.1);
  --dp-hover-text-color: #fff;
  --dp-primary-color: rgba(100, 140, 255, 0.85);
  --dp-primary-text-color: #fff;
  --dp-secondary-color: rgba(255, 255, 255, 0.15);
  --dp-border-color: rgba(255, 255, 255, 0.12);
  --dp-menu-border-color: rgba(255, 255, 255, 0.1);
  --dp-disabled-color: rgba(255, 255, 255, 0.05);
  --dp-disabled-color-text: rgba(255, 255, 255, 0.25);
  --dp-highlight-color: rgba(100, 140, 255, 0.15);
  --dp-range-between-dates-background-color: rgba(100, 140, 255, 0.1);
  --dp-range-between-dates-text-color: #e0e0e0;
  --dp-icon-color: rgba(255, 255, 255, 0.5);
}

/* 紧凑化日期选择面板 */
.dp__menu {
  border-radius: 10px !important;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5) !important;
  font-size: 13px !important;
}

.dp__cell_inner {
  width: 32px !important;
  height: 32px !important;
  font-size: 12px !important;
}

.dp__calendar_header_item {
  width: 32px !important;
  height: 28px !important;
  font-size: 11px !important;
}

.dp__month_year_wrap {
  font-size: 13px !important;
}

/* 时间选择器紧凑化 */
.dp__time_display {
  font-size: 13px !important;
}

.dp__overlay_cell {
  font-size: 12px !important;
}
</style>