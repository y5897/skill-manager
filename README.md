# Skill Manager

> AI Agent Skill 集中管理桌面应用  
> Centralized Skill Management for AI Coding Agents

---

## 概述 | Overview

Skill Manager 是一款基于 **Tauri v2 + React 19 + TypeScript** 构建的桌面应用，用于统一管理多个 AI 编码 Agent（如 OpenCode、Claude Code、Codex 等）的 Skills 目录。

### 核心功能 | Features

| 功能 | 说明 |
|------|------|
| **多 Agent 管理** | 支持 8+ 种 AI Agent，可自定义添加，路径支持 `{{USERNAME}}` 动态变量 |
| **Skill 资源库** | 统一 CRUD 管理，搜索筛选，Agent 绑定关系可视化 |
| **SKILL.md 编辑** | 内联编辑 + Markdown 实时渲染预览 |
| **文件管理** | 文件树浏览、拖拽上传、内容预览、删除管理 |
| **扫描发现** | 自动扫描 Agent 目录，发现新 Skill 并建立关联 |
| **同步追踪** | 将 Skill 同步到所有 Agent 目录，变更自动标记 `pending` 状态 |
| **文件校验** | SHA-256 哈希检测变更，仅增量同步 |
| **黑暗模式** | 亮/暗主题切换，自动跟随系统偏好 |
| **通知系统** | Toast 弹窗通知，替代原生 `alert()` |

### 架构 | Architecture

```
Skill Manager
├── Frontend (React 19 + TypeScript + Tailwind CSS)
│   ├── pages/          # 路由页面 (4个)
│   ├── components/     # UI 组件库 + Markdown 预览
│   ├── context/        # 主题 + Toast 上下文
│   ├── lib/api.ts      # Tauri IPC 桥接层
│   └── types/          # TypeScript 类型定义
├── Backend (Rust + Tauri v2)
│   ├── commands/       # Tauri IPC 命令 (agents/scanner/skills)
│   ├── db/             # SQLite 数据库 + migration
│   ├── models/         # 数据模型
│   └── sync/           # 同步引擎 (哈希校验 + 目录拷贝)
└── Database (SQLite)
    ├── agents          # Agent 配置表
    ├── skills          # Skill 元数据表
    ├── agent_skills    # 多对多关联 + 同步状态
    ├── skill_files     # 文件清单 + 哈希索引
    └── activities      # 操作日志
```

---

## 快速开始 | Quick Start

### 环境要求 | Prerequisites

- **Node.js** >= 18
- **Rust** >= 1.77.2
- **Visual Studio Build Tools** (Windows, 含 MSVC C++ 组件)

### 安装运行 | Install & Run

```bash
# 1. 安装前端依赖
npm install

# 2. 开发模式启动 (前端 + Tauri 桌面窗口)
npm run tauri dev

# 3. 仅启动前端 (浏览器访问 http://localhost:5173)
npm run dev

# 4. 生产构建
npm run tauri build
```

### 构建产物 | Build Output

- 可执行文件: `src-tauri/target/release/skill-manager.exe`
- 安装包(MSI): `src-tauri/target/release/bundle/msi/`

---

## 支持的 Agent | Supported Agents

| Agent | 标识 | 默认 Skills 路径 |
|-------|------|-----------------|
| OpenCode | `opencode` | `C:\Users\{{USERNAME}}\.config\opencode\skills` |
| Claude Code | `claude-code` | `C:\Users\{{USERNAME}}\.claude\skills` |
| Codex | `codex` | `C:\Users\{{USERNAME}}\.codex\skills` |
| Qoder | `qoder` | `C:\Users\{{USERNAME}}\.qoder\skills` |
| Trae CN | `trae-cn` | `C:\Users\{{USERNAME}}\.trae-cn\skills` |
| CodeBuddy | `codebuddy` | `C:\Users\{{USERNAME}}\.codebuddy\skills` |
| WorkBuddy | `workbuddy` | `C:\Users\{{USERNAME}}\.workbuddy\skills` |
| Hermes | `hermes` | `C:\Users\{{USERNAME}}\.hermes\skills` |

> `{{USERNAME}}` 在运行时自动替换为当前系统用户名。

---

## Skill 在线市场 | Marketplace

Skill Manager 内置在线市场，可以从 GitHub 发现并一键安装社区共享的 Skill。

### 安装 Skill

1. 打开应用左侧导航栏的 **🏪 Skill 市场**
2. 搜索或浏览可用的 Skill
3. 点击 **安装** 按钮，自动下载并部署到所有 Agent 目录

### 发布你的 Skill

想让自己的 Skill 出现在市场中？只需三步：

```bash
# 1. 创建 GitHub 仓库，放入 SKILL.md 和技能文件
# 2. 给仓库添加 topic: skill-manager-skill
# 3. 发布 GitHub Release，附件为 Skill 文件的 .zip 包
```

> 建议仓库名 = Skill slug，Release 的 zip 中 SKILL.md 应在根目录。

---

## 技术栈 | Tech Stack

| 层 | 技术 |
|-----|------|
| 桌面框架 | [Tauri v2](https://v2.tauri.app/) |
| 前端框架 | [React 19](https://react.dev/) |
| 类型系统 | [TypeScript 6](https://www.typescriptlang.org/) |
| 样式 | [Tailwind CSS 4](https://tailwindcss.com/) |
| 路由 | [React Router v7](https://reactrouter.com/) |
| 后端语言 | [Rust](https://www.rust-lang.org/) |
| 数据库 | [SQLite (rusqlite)](https://github.com/rusqlite/rusqlite) |
| 构建工具 | [Vite 8](https://vitejs.dev/) |

---

## License

MIT
