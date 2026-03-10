import { computed, ref, watch } from 'vue';
import { MAX_PRIVACY_BLUR } from '../constants/app';
import {
  loadPrivacySettings as readPrivacySettings,
  savePrivacySettings as writePrivacySettings,
} from '../services/storageService';
import type { PrivacySettings } from '../types/countdown';

export const usePrivacyMode = () => {
  const privacySettings = ref<PrivacySettings>(readPrivacySettings());
  const isPrivacyActive = ref(false);
  const isLongPressing = ref(false);
  const spreadProgress = ref(0);
  const revealProgress = ref(0);
  const widgetContainer = ref<HTMLElement | null>(null);
  const privacyTriggerBtn = ref<HTMLButtonElement | null>(null);
  const privacySpreadOrigin = ref({ x: 0, y: 0 });
  let longPressRaf: number | null = null;
  let longPressStartTime = 0;

  const contentBlurStyle = computed(() => {
    if (isPrivacyActive.value) {
      const blur = MAX_PRIVACY_BLUR * (1 - revealProgress.value);
      return { filter: `blur(${blur}px)` };
    }
    if (isLongPressing.value && spreadProgress.value > 0) {
      const blur = spreadProgress.value * MAX_PRIVACY_BLUR;
      return { filter: `blur(${blur}px)` };
    }
    return {};
  });

  const handlePrivacyChanged = (settings: PrivacySettings) => {
    privacySettings.value = { ...settings };
  };

  const updatePrivacySpreadOrigin = (triggerEl?: HTMLElement | null) => {
    const containerElement = widgetContainer.value;
    const sourceElement = triggerEl ?? privacyTriggerBtn.value;
    if (!containerElement || !sourceElement) {
      return;
    }

    const containerRect = containerElement.getBoundingClientRect();
    const triggerRect = sourceElement.getBoundingClientRect();

    privacySpreadOrigin.value = {
      x: triggerRect.left - containerRect.left + triggerRect.width / 2,
      y: triggerRect.top - containerRect.top + triggerRect.height / 2,
    };
  };

  const onPrivacyBtnDown = (event: MouseEvent) => {
    if (isPrivacyActive.value) {
      return;
    }

    updatePrivacySpreadOrigin(event.currentTarget as HTMLElement | null);
    isLongPressing.value = true;
    spreadProgress.value = 0;
    longPressStartTime = performance.now();

    const animate = (timestamp: number) => {
      if (!isLongPressing.value) {
        return;
      }

      const elapsed = timestamp - longPressStartTime;
      const progress = Math.min(elapsed / privacySettings.value.longPressDuration, 1);
      spreadProgress.value = progress * progress;

      if (progress >= 1) {
        isLongPressing.value = false;
        spreadProgress.value = 0;
        isPrivacyActive.value = true;
        longPressRaf = null;
        return;
      }

      longPressRaf = requestAnimationFrame(animate);
    };

    longPressRaf = requestAnimationFrame(animate);
  };

  const onPrivacyBtnUp = () => {
    if (!isLongPressing.value) {
      return;
    }

    isLongPressing.value = false;
    spreadProgress.value = 0;
    if (longPressRaf) {
      cancelAnimationFrame(longPressRaf);
      longPressRaf = null;
    }
  };

  const deactivatePrivacy = () => {
    isPrivacyActive.value = false;
    revealProgress.value = 0;
  };

  const handleRevealProgress = (progress: number) => {
    revealProgress.value = progress;
  };

  const stopRuntime = () => {
    if (longPressRaf) {
      cancelAnimationFrame(longPressRaf);
      longPressRaf = null;
    }
  };

  watch(
    privacySettings,
    (settings) => {
      writePrivacySettings(settings);
    },
    { deep: true },
  );

  return {
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
    stopRuntime,
  };
};
