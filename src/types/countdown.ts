/** 条目状态：active=正常 | expired=已到期待确认 | dismissed=已读待删除 */
export type ItemStatus = 'active' | 'expired' | 'dismissed';

export interface CountdownItemData {
  id: string;
  name: string;
  targetDate: number;
  bgColor: string;
  status: ItemStatus;
}

export const PRESET_COLORS = [
  'rgba(43, 45, 66, 0.8)',
  'rgba(217, 4, 41, 0.8)',
  'rgba(141, 153, 174, 0.8)',
  'rgba(45, 106, 79, 0.8)',
  'rgba(230, 140, 30, 0.8)',
  'rgba(100, 60, 180, 0.8)',
];

/** 隐私遮罩模式 */
export type PrivacyMaskMode = 'blur' | 'image';

/** 隐私模式设置 */
export interface PrivacySettings {
  /** 是否启用隐私模式功能 */
  enabled: boolean;
  /** 长按生效时长（毫秒） */
  longPressDuration: number;
  /** 遮罩模式 */
  maskMode: PrivacyMaskMode;
  /** 自定义图片（base64） */
  maskImage: string;
}

export interface ProxySettings {
  /** 是否启用本地 HTTP 代理 */
  enabled: boolean;
  /** 本地 HTTP 代理端口 */
  port: string;
}

export type UpdateProxySettings = ProxySettings;

export type WebDavProxySettings = ProxySettings;

export interface SyncSettings {
  /** 自动云同步间隔（毫秒） */
  autoSyncIntervalMs: number;
}

export const DEFAULT_PRIVACY_SETTINGS: PrivacySettings = {
  enabled: false,
  longPressDuration: 500,
  maskMode: 'blur',
  maskImage: '',
};

export const DEFAULT_PROXY_SETTINGS: ProxySettings = {
  enabled: false,
  port: '',
};

export const DEFAULT_UPDATE_PROXY_SETTINGS: UpdateProxySettings = {
  ...DEFAULT_PROXY_SETTINGS,
};

export const DEFAULT_WEBDAV_PROXY_SETTINGS: WebDavProxySettings = {
  ...DEFAULT_PROXY_SETTINGS,
};

export const DEFAULT_SYNC_SETTINGS: SyncSettings = {
  autoSyncIntervalMs: 60 * 1000,
};

export type AppView = 'list' | 'settings';

export interface WindowState {
  x: number;
  y: number;
  width: number;
  height: number;
}