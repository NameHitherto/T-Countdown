# T-Countdown

一个轻量、透明、常驻桌面的倒计时挂件工具。基于 Tauri 2 + Vue 3 + TypeScript 构建，适合放在桌面底层随时查看重要日期的倒计时。

![version](https://img.shields.io/badge/version-0.1.0-blue)
![platform](https://img.shields.io/badge/platform-Windows-lightgrey)

## ✨ 功能特性

- **倒计时管理** — 添加、编辑、删除倒计时条目，支持天数与日期双向同步输入
- **状态流转** — 条目自动检测到期（active → expired → dismissed），分状态展示
- **滑动删除** — 向左滑动条目露出删除按钮，方向锁定防误触
- **右键菜单** — 右键条目快速删除
- **自定义颜色** — 六种预设背景色，自由标记不同事项
- **窗口记忆** — 自动保存/恢复窗口位置与大小
- **折叠模式** — 一键折叠为仅标题栏，不占桌面空间
- **透明无边框** — 背景半透明毛玻璃效果，始终置于桌面底层、不出现在任务栏
- **坚果云 WebDAV 同步** — 绑定坚果云账号，一键上传/下载数据到云端，跨设备同步
- **设置中心** — 可扩展的设置面板，当前支持 Windows 开机自启

## 🏗️ 技术栈

| 层 | 技术 |
|---|---|
| 框架 | [Tauri 2](https://v2.tauri.app/) |
| 前端 | Vue 3 + TypeScript + Vite + SCSS |
| 后端 | Rust |
| 数据存储 | 本地 JSON (`Documents/T-Countdown/data.json`) |
| 云同步 | WebDAV (坚果云) via `ureq` |
| 开机自启 | Windows 注册表 (`winreg`) |

## 📦 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70
- [Tauri CLI](https://v2.tauri.app/start/create-project/) v2

### 开发

```bash
# 安装前端依赖
npm install

# 启动开发环境（前端 + Tauri 窗口）
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 📁 项目结构

```
T-Countdown/
├── src/                      # 前端源码
│   ├── App.vue               # 主视图（列表 / 同步 / 设置 三个面板）
│   ├── components/
│   │   ├── AddItemForm.vue   # 新增 / 编辑表单
│   │   ├── CountdownItem.vue # 倒计时条目（滑动删除 + 右键菜单）
│   │   ├── SyncPanel.vue     # 坚果云 WebDAV 同步面板
│   │   └── SettingsPanel.vue # 设置中心
│   └── types/
│       └── countdown.ts      # 类型定义 & 预设颜色
├── src-tauri/                # Rust 后端
│   └── src/
│       └── lib.rs            # Tauri 命令（数据读写 / WebDAV / 自启管理）
└── package.json
```

## 🔧 使用说明

| 操作 | 方式 |
|------|------|
| 拖动窗口 | 拖拽标题栏区域 |
| 新增条目 | 点击标题栏 **+** 按钮 |
| 编辑条目 | 双击条目 |
| 删除条目 | 左滑露出删除按钮，或右键选择删除 |
| 折叠/展开 | 点击标题栏 **∧/∨** 按钮 |
| 云同步 | 点击标题栏 **↻** 按钮，首次需绑定坚果云应用密码 |
| 设置 | 点击标题栏 **⚙** 按钮 |

## 📋 更新日志

### v0.1.0

> 首个正式版本

- 倒计时条目的增删改查与状态流转（active / expired / dismissed）
- 天数与日期时间双向同步输入
- 六色自定义背景
- 滑动删除 & 右键菜单删除
- 窗口位置与尺寸记忆
- 折叠模式
- 透明无边框窗口，桌面底层常驻
- 坚果云 WebDAV 云同步（绑定/上传/下载/解绑）
- 设置中心 & Windows 开机自启

## 📄 许可证

MIT
