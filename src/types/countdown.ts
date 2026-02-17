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