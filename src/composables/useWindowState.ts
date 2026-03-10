import { ref } from 'vue';
import { availableMonitors, getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import {
  COLLAPSED_HEIGHT,
  DEFAULT_WINDOW_SIZE,
  MIN_HEIGHT,
  MIN_WIDTH,
} from '../constants/app';
import {
  loadLockState,
  loadWindowState,
  saveLockState,
  saveWindowState as writeWindowState,
} from '../services/storageService';
import type { AppView, WindowState } from '../types/countdown';

export const useWindowState = () => {
  const isCollapsed = ref(false);
  const isAdding = ref(false);
  const isLocked = ref(loadLockState());
  const currentView = ref<AppView>('list');
  let stateTimer: number | null = null;
  let expandedSize = { ...DEFAULT_WINDOW_SIZE };

  const toggleView = (view: 'settings') => {
    currentView.value = currentView.value === view ? 'list' : view;
    isAdding.value = false;
  };

  const handleContainerClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement;
    if (!target.closest('.countdown-item-wrapper') && !target.closest('.header-actions')) {
      document.dispatchEvent(new CustomEvent('close-all-swipes', { detail: { id: null } }));
    }
  };

  const handleDragStart = async (event: MouseEvent) => {
    if (isLocked.value) {
      return;
    }
    const target = event.target as HTMLElement;
    if (target.closest('button, input, textarea, select, a, .countdown-item-wrapper, .list-container')) {
      return;
    }
    await getCurrentWindow().startDragging();
  };

  const toggleLock = async () => {
    isLocked.value = !isLocked.value;
    const appWindow = getCurrentWindow();
    if (isLocked.value) {
      await appWindow.setResizable(false);
    } else if (!isCollapsed.value) {
      await appWindow.setResizable(true);
    }
    saveLockState(isLocked.value);
  };

  const persistWindowState = async () => {
    try {
      const appWindow = getCurrentWindow();
      const factor = await appWindow.scaleFactor();
      const position = await appWindow.outerPosition();
      const size = await appWindow.innerSize();
      const state: WindowState = {
        x: position.x / factor,
        y: position.y / factor,
        width: isCollapsed.value ? expandedSize.width : size.width / factor,
        height: isCollapsed.value ? expandedSize.height : size.height / factor,
      };
      writeWindowState(state);
    } catch {
      // 忽略窗口状态保存失败
    }
  };

  const restoreWindowState = async () => {
    try {
      const savedState = loadWindowState();
      const appWindow = getCurrentWindow();

      if (savedState) {
        const width = Math.max(savedState.width, MIN_WIDTH);
        const height = Math.max(savedState.height, MIN_HEIGHT);
        expandedSize = { width, height };
        await appWindow.setSize(new LogicalSize(width, height));

        const monitors = await availableMonitors();
        const isOnScreen = monitors.some((monitor) => {
          const monitorX = monitor.position.x;
          const monitorY = monitor.position.y;
          const monitorWidth = monitor.size.width;
          const monitorHeight = monitor.size.height;
          return (
            savedState.x + 50 > monitorX &&
            savedState.x < monitorX + monitorWidth &&
            savedState.y + 50 > monitorY &&
            savedState.y < monitorY + monitorHeight
          );
        });

        if (isOnScreen) {
          await appWindow.setPosition(new LogicalPosition(savedState.x, savedState.y));
        } else {
          await appWindow.center();
        }
      }

      if (isLocked.value) {
        await appWindow.setResizable(false);
      }
    } catch {
      // 忽略窗口状态恢复失败
    }
  };

  const toggleCollapse = async () => {
    const appWindow = getCurrentWindow();

    if (!isCollapsed.value) {
      const factor = await appWindow.scaleFactor();
      const size = await appWindow.innerSize();
      expandedSize = { width: size.width / factor, height: size.height / factor };
      isCollapsed.value = true;
      isAdding.value = false;
      currentView.value = 'list';
      await appWindow.setResizable(false);
      await appWindow.setSize(new LogicalSize(expandedSize.width, COLLAPSED_HEIGHT));
      return;
    }

    isCollapsed.value = false;
    await appWindow.setSize(new LogicalSize(expandedSize.width, expandedSize.height));
    await appWindow.setResizable(!isLocked.value);
  };

  const closeWindow = async (beforeClose?: () => Promise<void>) => {
    await persistWindowState();
    if (beforeClose) {
      await beforeClose();
    }
    await getCurrentWindow().close();
  };

  const startAdding = () => {
    isAdding.value = true;
  };

  const stopAdding = () => {
    isAdding.value = false;
  };

  const startPersistence = () => {
    stateTimer = window.setInterval(() => {
      void persistWindowState();
    }, 5000);
  };

  const stopPersistence = () => {
    if (stateTimer) {
      clearInterval(stateTimer);
      stateTimer = null;
    }
  };

  return {
    isCollapsed,
    isAdding,
    isLocked,
    currentView,
    toggleView,
    handleContainerClick,
    handleDragStart,
    toggleLock,
    persistWindowState,
    restoreWindowState,
    toggleCollapse,
    closeWindow,
    startAdding,
    stopAdding,
    startPersistence,
    stopPersistence,
  };
};
