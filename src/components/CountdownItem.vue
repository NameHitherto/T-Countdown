<template>
  <div class="countdown-item-wrapper">
    <!-- ===== 编辑模式 ===== -->
    <AddItemForm
      v-if="isEditing"
      :edit-item="item"
      @confirm="handleEditConfirm"
      @cancel="isEditing = false"
    />

    <!-- ===== 滑动删除容器 ===== -->
    <div v-else class="swipe-container" @contextmenu.prevent.stop="showContextMenu">
      <!-- 背景删除按钮（滑动时露出） -->
      <div class="swipe-action" :class="{ visible: swipeOffset < -40 }">
        <button class="swipe-delete-btn" @click="confirmDelete">
          <svg width="16" height="16" viewBox="0 0 14 14" fill="none">
            <path d="M3 4h8M5.5 4V3a.5.5 0 01.5-.5h2a.5.5 0 01.5.5v1M4.5 4v7.5a1 1 0 001 1h3a1 1 0 001-1V4"
              stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span>删除</span>
        </button>
      </div>

      <!-- 可滑动的内容层 -->
      <div
        class="swipe-content"
        :class="{ dragging: isDragging }"
        :style="{ transform: `translateX(${swipeOffset}px)` }"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      >
        <!-- ===== 待删除（dismissed）状态 ===== -->
        <div
          v-if="item.status === 'dismissed'"
          class="countdown-item dismissed"
        >
          <div class="item-left">
            <div class="item-name line-through">{{ item.name }}</div>
            <div class="item-target">已确认 · 点击删除</div>
          </div>
          <button class="action-btn delete-btn" title="确认删除" @click="$emit('delete', item.id)">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M3 4h8M5.5 4V3a.5.5 0 01.5-.5h2a.5.5 0 01.5.5v1M4.5 4v7.5a1 1 0 001 1h3a1 1 0 001-1V4"
                stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </button>
        </div>

        <!-- ===== 到期（expired）状态 ===== -->
        <div
          v-else-if="item.status === 'expired'"
          class="countdown-item expired"
        >
          <div class="item-left">
            <div class="item-name">{{ item.name }}</div>
            <div class="item-target">已到期 · {{ formatDate(item.targetDate) }}</div>
          </div>
          <button class="action-btn ack-btn" title="已读" @click="$emit('dismiss', item.id)">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M2.5 7.5l3 3 6-6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </button>
        </div>

        <!-- ===== 正常展示模式 ===== -->
        <div
          v-else
          class="countdown-item active"
          :style="{ backgroundColor: item.bgColor }"
          @dblclick="startEdit"
          title="双击编辑"
        >
          <div class="item-left">
            <div class="item-name">{{ item.name }}</div>
            <div class="item-target">{{ formatDate(item.targetDate) }}</div>
          </div>
          <div class="item-right">
            <div class="time-main">
              <span class="days-number">{{ timeLeft.days }}</span><span class="days-unit">天</span>
            </div>
            <div class="time-sub">{{ timeLeft.hours }}<span class="sub-unit">小时</span>{{ timeLeft.minutes }}<span class="sub-unit">分钟</span></div>
          </div>
        </div>
      </div>

      <!-- 右键菜单 -->
      <Teleport to="body">
        <div
          v-if="contextMenuVisible"
          class="context-menu"
          :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
        >
          <button class="context-menu-item danger" @click="handleContextDelete">
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
              <path d="M3 4h8M5.5 4V3a.5.5 0 01.5-.5h2a.5.5 0 01.5.5v1M4.5 4v7.5a1 1 0 001 1h3a1 1 0 001-1V4"
                stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            删除
          </button>
        </div>
      </Teleport>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import type { CountdownItemData } from '../types/countdown';
import AddItemForm from './AddItemForm.vue';

const props = defineProps<{
  item: CountdownItemData;
  tick: number;
}>();

const emit = defineEmits<{
  (e: 'update', item: CountdownItemData): void;
  (e: 'dismiss', id: string): void;
  (e: 'delete', id: string): void;
}>();

const isEditing = ref(false);

// ---- 滑动删除 ----

const swipeOffset = ref(0);
const isDragging = ref(false);
let startX = 0;
let startY = 0;
let swiping = false;
let directionLocked = false;
let isHorizontal = false;
let activePointerId = -1;

const onPointerDown = (e: PointerEvent) => {
  if (e.button !== 0) return; // 仅响应主按钮
  startX = e.clientX;
  startY = e.clientY;
  swiping = true;
  directionLocked = false;
  isHorizontal = false;
  activePointerId = e.pointerId;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
};

const onPointerMove = (e: PointerEvent) => {
  if (!swiping || e.pointerId !== activePointerId) return;
  const dx = e.clientX - startX;
  const dy = e.clientY - startY;

  if (!directionLocked) {
    // 需要至少 10px 移动才判定方向（增大死区防误触）
    if (Math.abs(dx) < 10 && Math.abs(dy) < 10) return;
    directionLocked = true;
    // 水平移动量需超过垂直的 1.5 倍才视为滑动
    isHorizontal = Math.abs(dx) > Math.abs(dy) * 1.5;
    if (!isHorizontal) {
      swiping = false;
      return;
    }
    isDragging.value = true;
  }

  // 仅允许向左滑，范围 [-80, 0]
  swipeOffset.value = Math.min(0, Math.max(-80, dx));
};

const onPointerUp = (e: PointerEvent) => {
  if (e.pointerId !== activePointerId) return;
  swiping = false;
  isDragging.value = false;
  // 超过一半距离则停留展开，否则弹回
  swipeOffset.value = swipeOffset.value < -40 ? -80 : 0;
};

const confirmDelete = () => {
  swipeOffset.value = 0;
  emit('delete', props.item.id);
};

// ---- 右键菜单 ----

const contextMenuVisible = ref(false);
const contextMenuPos = ref({ x: 0, y: 0 });

const showContextMenu = (e: MouseEvent) => {
  contextMenuPos.value = { x: e.clientX, y: e.clientY };
  contextMenuVisible.value = true;
};

const handleContextDelete = () => {
  contextMenuVisible.value = false;
  emit('delete', props.item.id);
};

const closeContextMenu = () => {
  contextMenuVisible.value = false;
};

onMounted(() => {
  document.addEventListener('click', closeContextMenu);
});

onUnmounted(() => {
  document.removeEventListener('click', closeContextMenu);
});

// ---- 倒计时 ----

const timeLeft = computed(() => {
  void props.tick;
  const now = Date.now();
  const diff = props.item.targetDate - now;
  if (diff <= 0) return { days: 0, hours: 0, minutes: 0 };
  const days = Math.floor(diff / 86400000);
  const hours = Math.floor((diff % 86400000) / 3600000);
  const minutes = Math.floor((diff % 3600000) / 60000);
  return { days, hours, minutes };
});

const formatDate = (timestamp: number) => {
  const d = new Date(timestamp);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
};

const startEdit = () => {
  isEditing.value = true;
};

const handleEditConfirm = (updated: CountdownItemData) => {
  isEditing.value = false;
  emit('update', updated);
};
</script>

<style scoped>
.countdown-item-wrapper {
  margin-bottom: 10px;
}

/* ---- 滑动容器 ---- */

.swipe-container {
  position: relative;
  overflow: hidden;
  border-radius: 10px;
}

.swipe-content {
  position: relative;
  z-index: 1;
  will-change: transform;
  touch-action: pan-y;
}

.swipe-content:not(.dragging) {
  transition: transform 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.swipe-action {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(220, 50, 50, 0.65);
  border-radius: 0 10px 10px 0;
  opacity: 0;
  transition: opacity 0.2s;
}

.swipe-action.visible {
  opacity: 1;
}

.swipe-delete-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  background: none;
  border: none;
  color: white;
  cursor: pointer;
  font-size: 11px;
  padding: 6px 10px;
  border-radius: 6px;
  transition: background 0.15s;
}

.swipe-delete-btn:hover {
  background: rgba(255, 255, 255, 0.15);
}

/* ---- 右键菜单 ---- */

.context-menu {
  position: fixed;
  z-index: 10000;
  background: rgba(35, 35, 35, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  padding: 4px;
  min-width: 100px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(12px);
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 6px 10px;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: rgba(255, 255, 255, 0.85);
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
}

.context-menu-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.context-menu-item.danger {
  color: rgba(255, 120, 120, 0.95);
}

.context-menu-item.danger:hover {
  background: rgba(220, 50, 50, 0.2);
}

/* ---- 基础条目样式 ---- */

.countdown-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-radius: 10px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  transition: box-shadow 0.2s, background-color 0.2s, opacity 0.2s, filter 0.2s;
  min-height: 56px;
}

/* ---- active 条目 ---- */

.countdown-item.active {
  cursor: pointer;
}

.countdown-item.active:hover {
  box-shadow: inset 3px 0 0 rgba(255, 255, 255, 0.35), 0 4px 12px rgba(0, 0, 0, 0.25);
  filter: brightness(1.08);
}

/* ---- expired 条目 ---- */

.countdown-item.expired {
  background: rgba(220, 50, 50, 0.15);
  border: 1px solid rgba(220, 50, 50, 0.3);
  animation: pulseGlow 2s ease-in-out infinite;
}

@keyframes pulseGlow {
  0%, 100% { box-shadow: 0 0 0 0 rgba(220, 50, 50, 0); }
  50% { box-shadow: 0 0 12px 2px rgba(220, 50, 50, 0.2); }
}

/* ---- dismissed 条目 ---- */

.countdown-item.dismissed {
  background: rgba(255, 255, 255, 0.04);
  opacity: 0.5;
}

.countdown-item.dismissed:hover {
  opacity: 0.7;
}

.line-through {
  text-decoration: line-through;
  opacity: 0.7;
}

/* ---- 内容 ---- */

.item-left {
  display: flex;
  flex-direction: column;
  gap: 3px;
  overflow: hidden;
  flex: 1;
  min-width: 0;
}

.item-name {
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-target {
  font-size: 11px;
  opacity: 0.6;
  font-variant-numeric: tabular-nums;
}

.item-right {
  flex-shrink: 0;
  text-align: right;
  margin-left: 12px;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.time-main {
  display: flex;
  align-items: baseline;
}

.time-main .days-number {
  font-size: 26px;
  font-weight: bold;
  font-variant-numeric: tabular-nums;
  line-height: 1;
}

.time-main .days-unit {
  font-size: 12px;
  opacity: 0.7;
  margin-left: 2px;
}

.time-sub {
  font-size: 11px;
  opacity: 0.5;
  font-variant-numeric: tabular-nums;
  margin-top: 2px;
}

.time-sub .sub-unit {
  font-size: 10px;
  opacity: 0.7;
  margin: 0 1px;
}

/* ---- 操作按钮 ---- */

.action-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 50%;
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: background 0.2s, transform 0.15s;
  flex-shrink: 0;
  margin-left: 12px;
  padding: 0;
}

.ack-btn {
  background: rgba(220, 50, 50, 0.25);
  color: rgba(255, 150, 150, 0.9);
}

.ack-btn:hover {
  background: rgba(220, 50, 50, 0.45);
  color: white;
  transform: scale(1.1);
}

.delete-btn {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.4);
}

.delete-btn:hover {
  background: rgba(220, 50, 50, 0.35);
  color: rgba(255, 150, 150, 0.9);
  transform: scale(1.1);
}
</style>