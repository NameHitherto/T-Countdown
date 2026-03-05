<template>
  <!-- 长按蔓延动画遮罩（激活中但尚未完全生效） -->
  <div
    v-if="spreading"
    class="privacy-spread-overlay"
  >
    <template v-if="settings.maskMode === 'blur'">
      <!-- 半透明磨砂蓖覆层，用 mask-image 裁圆 -->
      <div class="spread-blur-layer" :style="spreadMaskImageStyle"></div>
    </template>
    <!-- 图片模式：圆形 mask-image 裁切图片 -->
    <div
      v-else
      class="spread-image-mask"
      :style="spreadImageMaskStyle"
    ></div>
  </div>

  <!-- 隐私模式激活后的完整遮罩 -->
  <div
    v-if="active"
    class="privacy-overlay"
    :style="overlayMaskStyle"
  >
    <!-- 遮罩内容 -->
    <div class="privacy-mask">
      <div v-if="settings.maskMode === 'blur'" class="blur-mask"></div>
      <div v-else class="image-mask" :style="{ backgroundImage: `url(${settings.maskImage})` }"></div>
    </div>

    <!-- 右上角关闭按钮（鼠标靠近时显示） -->
    <div
      class="corner-zone top-right"
      @mouseenter="showClose = true"
      @mouseleave="showClose = false"
    >
      <Transition name="fade">
        <button v-if="showClose" class="corner-btn close-btn" title="关闭" @click="$emit('close')">
          <svg width="16" height="16" viewBox="0 0 14 14" fill="none">
            <path d="M2 2l10 10M12 2L2 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      </Transition>
    </div>

    <!-- 右下角眼睛按钮（鼠标靠近时显示） -->
    <div
      class="corner-zone bottom-right"
      @mouseenter="showEye = true"
      @mouseleave="showEye = false"
    >
      <Transition name="fade">
        <button v-if="showEye" class="corner-btn eye-btn" title="退出隐私模式" @click="startReveal">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8S1 12 1 12z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
            <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.8"/>
          </svg>
        </button>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import type { PrivacySettings } from '../types/countdown';

const props = defineProps<{
  active: boolean;
  spreading: boolean;
  spreadProgress: number; // 0~1
  settings: PrivacySettings;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'deactivate'): void;
  (e: 'reveal-progress', progress: number): void;
}>();

const showClose = ref(false);
const showEye = ref(false);
const isRevealing = ref(false);
const revealProgress = ref(0);
let revealRaf: number | null = null;

// 容器尺寸（用于计算对角线）
const containerW = ref(320);
const containerH = ref(420);

const updateContainerSize = () => {
  const el = document.querySelector('.widget-container');
  if (el) {
    const rect = el.getBoundingClientRect();
    containerW.value = rect.width;
    containerH.value = rect.height;
  }
};

onMounted(updateContainerSize);

// 对角线长度 = 从右下角到左上角的距离（确保圆能覆盖整个窗口）
const diagonal = computed(() => {
  return Math.sqrt(containerW.value ** 2 + containerH.value ** 2);
});

// 蔓延圆心 px
const anchorX = computed(() => containerW.value - 24);
const anchorY = computed(() => containerH.value - 24);
const spreadR = computed(() => props.spreadProgress * diagonal.value);

// ========== 蔓延动画样式 ==========

// mask-image 圆（用于模糊蓖覆层和图片蔓延）
const spreadMaskImageStyle = computed(() => {
  const r = spreadR.value;
  const x = anchorX.value;
  const y = anchorY.value;
  return {
    WebkitMaskImage: `radial-gradient(circle ${r}px at ${x}px ${y}px, black 100%, transparent 100%)`,
    maskImage: `radial-gradient(circle ${r}px at ${x}px ${y}px, black 100%, transparent 100%)`,
  };
});

// 图片蔓延
const spreadImageMaskStyle = computed(() => {
  const r = spreadR.value;
  const x = anchorX.value;
  const y = anchorY.value;
  return {
    WebkitMaskImage: `radial-gradient(circle ${r}px at ${x}px ${y}px, black 100%, transparent 100%)`,
    maskImage: `radial-gradient(circle ${r}px at ${x}px ${y}px, black 100%, transparent 100%)`,
    backgroundImage: `url(${props.settings.maskImage})`,
  };
});

// ========== 揭示动画（从中心扩散） ==========
const overlayMaskStyle = computed(() => {
  if (!isRevealing.value) return {};
  const maxRadius = diagonal.value;
  const r = revealProgress.value * maxRadius;
  return {
    maskImage: `radial-gradient(circle ${r}px at 50% 50%, transparent 100%, black 100%)`,
    WebkitMaskImage: `radial-gradient(circle ${r}px at 50% 50%, transparent 100%, black 100%)`,
  };
});

const REVEAL_DURATION = 500;

const startReveal = () => {
  updateContainerSize();
  isRevealing.value = true;
  revealProgress.value = 0;
  showEye.value = false;

  const startTime = performance.now();

  const animate = (now: number) => {
    const elapsed = now - startTime;
    const t = Math.min(elapsed / REVEAL_DURATION, 1);
    revealProgress.value = 1 - Math.pow(1 - t, 3);
    emit('reveal-progress', revealProgress.value);

    if (t < 1) {
      revealRaf = requestAnimationFrame(animate);
    } else {
      isRevealing.value = false;
      revealProgress.value = 0;
      emit('deactivate');
    }
  };

  revealRaf = requestAnimationFrame(animate);
};

onUnmounted(() => {
  if (revealRaf) cancelAnimationFrame(revealRaf);
});
</script>

<style scoped>
/* 长按蔓延遮罩 */
.privacy-spread-overlay {
  position: absolute;
  inset: 0;
  z-index: 1000;
  pointer-events: none;
  border-radius: 6px;
  overflow: hidden;
}

/*
 * 毛玻璃蔓延 —— 半透明磨砂蒙版
 * 真正的模糊由 App.vue 中 content-wrapper 的 filter:blur() 实现
 * 此层只提供半透明深色蓖覆增强磨砂质感
 */
.spread-blur-layer {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  background: rgba(15, 15, 25, 0.2);
}

/* 图片蔓延 */
.spread-image-mask {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
}

/* 完整遮罩 */
.privacy-overlay {
  position: absolute;
  inset: 0;
  z-index: 1000;
  border-radius: 6px;
  overflow: hidden;
}

.privacy-mask {
  position: absolute;
  inset: 0;
}

.blur-mask {
  width: 100%;
  height: 100%;
  background: rgba(15, 15, 25, 0.35);
}

.image-mask {
  width: 100%;
  height: 100%;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
}

/* 角落检测区域 */
.corner-zone {
  position: absolute;
  width: 60px;
  height: 60px;
  z-index: 1001;
}

.corner-zone.top-right {
  top: 0;
  right: 0;
}

.corner-zone.bottom-right {
  bottom: 0;
  right: 0;
}

/* 角落按钮 */
.corner-btn {
  position: absolute;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.75);
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: background 0.2s, color 0.2s, transform 0.15s;
  padding: 0;
}

.corner-btn:hover {
  background: rgba(255, 255, 255, 0.25);
  color: white;
  transform: scale(1.1);
}

.close-btn {
  top: 10px;
  right: 10px;
}

.close-btn:hover {
  background: rgba(220, 50, 50, 0.55);
}

.eye-btn {
  bottom: 10px;
  right: 10px;
}

/* 淡入淡出 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
