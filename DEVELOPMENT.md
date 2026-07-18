# Knockd Client 全流程开发指南

> Tauri v2 + Rust + React + TypeScript + Tailwind CSS + SQLite 跨平台桌面应用开发手册

---

## 目录

1. [环境搭建](#1-环境搭建)
2. [项目结构](#2-项目结构)
3. [日常开发](#3-日常开发)
4. [Rust 后端开发](#4-rust-后端开发)
5. [React 前端开发](#5-react-前端开发)
6. [Tauri IPC 通信](#6-tauri-ipc-通信)
7. [测试](#7-测试)
8. [构建与打包](#8-构建与打包)
9. [CI/CD](#9-cicd)
10. [发布与分发](#10-发布与分发)
11. [常见问题](#11-常见问题)

---

## 1. 环境搭建

### 1.1 系统要求

| 平台 | 最低版本 |
|------|---------|
| Ubuntu/Debian | 22.04+ |
| Windows | 10+ (Build 19041+) |
| macOS | 12+ (Monterey) |
| Rust | 1.88+ |
| Node.js | 18+ |

### 1.2 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
rustup update
```

### 1.3 安装 Node.js & pnpm

```bash
# 使用 nvm (推荐)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.0/install.sh | bash
nvm install 18
nvm use 18

# 安装 pnpm
npm install -g pnpm
```

### 1.4 安装 Tauri 系统依赖

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libjavascriptcoregtk-4.1-dev \
  libsoup-3.0-dev \
  curl wget file
```

**Windows:**
- 安装 [Microsoft Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（勾选 "Desktop development with C++"）
- 或者安装 [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)（Windows 11 已内置）

**macOS:**
```bash
xcode-select --install
```

### 1.5 克隆项目

```bash
git clone <repo-url> knockd-client
cd knockd-client
pnpm install
```

### 1.6 验证环境

```bash
# 检查 Rust
rustc --version  # >= 1.88
cargo --version

# 检查 Node
node --version   # >= 18
pnpm --version

# 检查 Tauri 环境
pnpm tauri info
```

---

## 2. 项目结构

```
knockd/
├── src/                          # React 前端 (TypeScript)
│   ├── main.tsx                  # 入口：挂载 React
│   ├── App.tsx                   # 主组件：布局、状态管理、过滤
│   ├── api.ts                    # Tauri invoke 封装层
│   ├── types.ts                  # TypeScript 接口定义
│   ├── index.css                 # Tailwind + 全局样式
│   ├── vite-env.d.ts             # Vite 类型声明
│   └── components/
│       ├── ConnectionCard.tsx     # 连接卡片 (展示+操作)
│       ├── ConnectionForm.tsx     # 新增/编辑表单 (模态框)
│       └── SettingsPanel.tsx      # 设置面板 (模态框)
│
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口：启动应用
│   │   ├── lib.rs                # Tauri 配置、插件注册、命令注册
│   │   ├── models.rs             # 数据结构 (Serialize/Deserialize)
│   │   ├── db.rs                 # SQLite 数据库操作 (CRUD + 建表)
│   │   ├── knock.rs              # UDP/TCP 端口敲门引擎
│   │   ├── launcher.rs           # SSH 客户端检测 + 启动器
│   │   └── commands.rs           # Tauri IPC 命令处理器
│   ├── capabilities/
│   │   └── default.json          # 权限声明
│   ├── icons/                    # 应用图标 (多尺寸)
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 应用配置
│   └── build.rs                  # Tauri 构建脚本
│
├── index.html                    # HTML 入口
├── vite.config.ts                # Vite 配置
├── tsconfig.json                 # TypeScript 配置
├── tailwind.config.js            # Tailwind 配置
├── postcss.config.js             # PostCSS 配置
├── package.json                  # Node 依赖 + 脚本
├── releases/                     # 构建产物
│   ├── knockd-client_*.deb       # Debian 包
│   └── knockd-client_*.exe       # Windows 可执行文件
└── README.md
```

---

## 3. 日常开发

### 3.1 启动开发模式

```bash
cd knockd-client
pnpm tauri dev
```

这会同时启动：
- **Vite 开发服务器** (localhost:1420) — 热更新前端
- **Rust 后端编译** — 每次修改自动重编译
- **Tauri 窗口** — 展示应用

### 3.2 仅启动前端（脱离 Tauri 调试 UI）

```bash
pnpm dev
# 浏览器打开 http://localhost:1420
# 注意：Tauri API 在浏览器中不可用，会报错
```

### 3.3 Rust 热重载 (cargo watch)

```bash
cargo install cargo-watch
cd src-tauri
cargo watch -x run
```

### 3.4 代码检查

```bash
# Rust
cargo clippy -- -D warnings
cargo fmt --check

# TypeScript
pnpm exec tsc --noEmit
pnpm exec eslint src/
```

### 3.5 项目常用命令

| 命令 | 说明 |
|------|------|
| `pnpm tauri dev` | 开发模式（热重载） |
| `pnpm tauri build` | 生产构建（当前平台） |
| `pnpm build` | 仅前端构建 |
| `cargo build` | 仅 Rust 构建 |
| `cargo check` | Rust 类型检查（快） |
| `cargo clippy` | Rust 代码检查 |
| `cargo fmt` | Rust 代码格式化 |
| `pnpm tauri icon <file>` | 生成应用图标 |

---

## 4. Rust 后端开发

### 4.1 依赖说明 (`Cargo.toml`)

```toml
[dependencies]
tauri = { version = "2", features = [] }        # Tauri 框架
tauri-plugin-dialog = "2"                        # 原生文件选择器
rusqlite = { version = "0.31", features = ["bundled"] }  # SQLite
serde = { version = "1", features = ["derive"] }  # 序列化
serde_json = "1"                                  # JSON
dirs = "5"                                        # 系统目录
glob = "0.3"                                      # 文件通配
ed25519-dalek = "2"                               # Ed25519 签名
curve25519-dalek = "4"                            # X25519 ECDH
aes-gcm = "0.10"                                  # AES-256-GCM
sha2 = "0.10"                                     # SHA-256
hmac = "0.12"                                     # HMAC-SHA256
hex = "0.4"                                       # Hex 编解码
rand = "0.8"                                      # 随机数
hostname = "0.4"                                  # 主机名 (设备指纹)
log = "0.4"                                       # 日志
env_logger = "0.11"                               # 环境变量日志
```

### 4.2 模块设计

```
lib.rs ─── 注册插件、初始化数据库、注册命令、启动应用
  ├── commands.rs   ←── Tauri IPC (前端调用)
  │     ├── db.rs           (SQLite CRUD + 加密迁移)
  │     ├── knock.rs        (端口敲门)
  │     ├── knockpass.rs    (SPA: Ed25519+X25519+AES+动态端口)
  │     ├── crypto_store.rs (AES-256-GCM 加密存储)
  │     ├── spa_cmds.rs     (SPA 加密/解密命令)
  │     └── launcher.rs     (SSH客户端检测 + 启动)
  └── models.rs      ←── 共享数据结构
```

**设计原则**:
- knockpass.rs 只做加密运算，不直接访问存储
- 所有密钥通过 crypto_store.rs 加密后存入 SQLite settings 表
- 加密密钥由 `SHA256(设备指纹 + pepper)` 派生
- 前台 Save 时调用 `spaEncrypt` → `storeEncryptedKey` 写入 DB
- SPA 包格式与 [knockpass-server](https://github.com/0xHertz/KnockSPApass) 完全兼容：
  Ed25519 签名 + AES-256-GCM(priv_key) + HMAC-SHA256 动态端口

```
lib.rs ─── 注册插件、初始化数据库、注册命令、启动应用
  ├── commands.rs  ←── Tauri IPC 入口 (前端调用)
  │     ├── db.rs          (SQLite CRUD)
  │     ├── knock.rs       (端口敲门)
  │     └── launcher.rs    (启动 SSH/浏览器)
  └── models.rs     ←── 共享数据结构
```

**设计原则：**
- 每个模块独立职责，通过 `models.rs` 共享类型
- 数据库使用 `Mutex<SqliteConnection>` 保证线程安全
- 敲门操作是同步的（阻塞），因为需要等待包序列
- 启动外部程序使用 `std::process::Command::spawn()`（非阻塞）

### 4.3 添加新 Tauri 命令

1. 在 `commands.rs` 中定义函数并加 `#[tauri::command]`:
```rust
#[tauri::command]
pub fn my_command(state: State<AppState>, arg: String) -> Result<String, String> {
    // 访问数据库: state.db.get_setting(...)
    Ok(format!("Got: {}", arg))
}
```

2. 在 `lib.rs` 注册:
```rust
.invoke_handler(tauri::generate_handler![
    commands::my_command,  // 新增
    commands::list_connections,
    // ...
])
```

3. 在前端 `api.ts` 添加调用:
```typescript
export async function myCommand(arg: string): Promise<string> {
  return invoke("my_command", { arg });
}
```

### 4.4 添加新的 Rust 依赖

```bash
cd src-tauri
cargo add <crate-name>
```

### 4.5 日志

使用 `log` crate，配合 `env_logger`。在代码中：

```rust
log::info!("Knocking {} on port {}", host, port);
log::warn!("Knock failed: {}", e);
```

运行时设置日志级别：
```bash
RUST_LOG=debug pnpm tauri dev
```

---

## 5. React 前端开发

### 5.1 技术选型

| 层面 | 技术 | 理由 |
|------|------|------|
| 框架 | React 18 | 生态成熟，与 Tauri 配合良好 |
| 语言 | TypeScript 5 (strict) | 类型安全，减少运行时错误 |
| 构建 | Vite 6 | 快速 HMR，支持 Tauri |
| 样式 | Tailwind CSS 3 | utility-first，开发效率高 |
| 状态 | React useState | 简单场景无需 Redux |

### 5.2 组件树

```
App
├── Header (搜索框 + 过滤按钮 + 新增按钮)
├── StatusBar (操作反馈消息)
├── SettingsPanel (设置模态框)
├── ConnectionGrid
│   └── ConnectionCard[] (卡片列表)
│       └── [Knock & Connect] [Edit] [Delete]
└── ConnectionForm (新增/编辑模态框)
```

### 5.3 添加新组件

```bash
# 创建文件
touch src/components/NewComponent.tsx
```

遵循现有模式：
- 使用 TypeScript interface 定义 Props
- 使用 Tailwind 类名，暗色主题 (`bg-slate-800`, `text-slate-300` 等)
- 通过 `api.ts` 调用后端

### 5.4 样式约定

```tsx
// 主背景:  bg-slate-900
// 卡片:    bg-slate-800/60 border border-slate-700/50
// 输入框:  bg-slate-800 border border-slate-600/50
// 主按钮:  bg-emerald-600 hover:bg-emerald-500
// 文字:    text-slate-300 (主要) text-slate-400 (次要) text-slate-500 (辅助)
// SSH标签: bg-blue-900/50 text-blue-300
// Web标签: bg-purple-900/50 text-purple-300
```

---

## 6. Tauri IPC 通信

### 6.1 架构

```
React (TypeScript)          Rust
─────────────────    IPC    ─────────────────
api.ts ──invoke()──→       commands.rs
                            ├── db.rs (SQLite)
                            ├── knock.rs
                            └── launcher.rs
                   ←──Result
```

### 6.2 前端调用模式

```typescript
// api.ts
import { invoke } from "@tauri-apps/api/core";

// 查询 (返回数据)
export async function listConnections(): Promise<Connection[]> {
  return invoke("list_connections");
}

// 命令 (有副作用)
export async function knockAndConnect(id: number): Promise<string> {
  return invoke("knock_and_connect", { connectionId: id });
}

// 在组件中使用
const [connections, setConnections] = useState<Connection[]>([]);
useEffect(() => {
  listConnections().then(setConnections).catch(console.error);
}, []);
```

### 6.3 后端命令模式

```rust
// commands.rs
#[tauri::command]
pub fn list_connections(state: State<AppState>) -> Result<Vec<Connection>, String> {
    state.db.list_connections().map_err(|e| e.to_string())
}
```

**关键规则：**
- 访问数据库：`state.db.<method>()` — 内部 Mutex 自动加锁
- 返回类型必须是 `Result<T, String>`（前端自动转 Promise）
- 复杂参数用 struct（带 Serialize/Deserialize）

---

## 7. 测试

### 7.1 Rust 单元测试

```rust
// 在 db.rs 底部添加
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knock_sequence_parsing() {
        let ports = r#"[{"protocol":"udp","port":7000}]"#;
        let result = perform_knock("127.0.0.1", ports, "udp", 10);
        assert!(result.success);
    }
}
```

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_knock

# 显示输出
cargo test -- --nocapture
```

### 7.2 前端测试 (Vitest)

```bash
# 安装
pnpm add -D vitest @testing-library/react @testing-library/jest-dom jsdom

# 配置 vite.config.ts 添加 test 配置
```

```tsx
// src/components/__tests__/ConnectionCard.test.tsx
import { render, screen } from "@testing-library/react";
import ConnectionCard from "../ConnectionCard";

test("renders connection name", () => {
  render(<ConnectionCard connection={mockConnection} ... />);
  expect(screen.getByText("My Server")).toBeInTheDocument();
});
```

```bash
pnpm vitest
```

### 7.3 端到端测试

Tauri 的 E2E 测试框架还在发展中。当前推荐手动测试清单：

**功能测试清单：**
- [ ] 添加 SSH 连接 → 保存成功 → 列表中可见
- [ ] 添加 Web 连接 → 保存成功 → 列表中可见
- [ ] 编辑连接 → 修改字段 → 保存 → 更新显示
- [ ] 删除连接 → 确认 → 从列表消失
- [ ] 搜索过滤 → 输入关键词 → 正确过滤
- [ ] 类型过滤 (SSH/Web/All) → 正确过滤
- [ ] 敲门顺序 → 多端口序列正确发送
- [ ] SSH 启动 → 正确调用系统 SSH 客户端
- [ ] Web 启动 → 正确调用系统浏览器
- [ ] 设置保存 → 关闭重开 → 设置持久化
- [ ] 数据库持久化 → 重启应用 → 数据完整

---

## 8. 构建与打包

### 8.1 本地平台构建

```bash
pnpm tauri build
```

**Bundles 配置** (`tauri.conf.json`):
```json
"bundle": {
    "active": true,
    "targets": "all",
    "linux": { "deb": { "depends": [...] } },
    "windows": { "webviewInstallMode": { "type": "embedBootstrapper" } }
}
```

| 平台 | 产物 |
|------|------|
| **Linux** | `.deb` + `.rpm` + `.AppImage` |
| **Windows** | `.msi` + `.exe` NSIS 安装包（含 WebView2 引导） |
| **macOS** | `.dmg` + `.app` |

### 8.2 Linux 手动打包 .deb

```bash
mkdir -p packaging/deb/DEBIAN packaging/deb/usr/bin \
  packaging/deb/usr/share/applications \
  packaging/deb/usr/share/icons/hicolor/256x256/apps

cat > packaging/deb/DEBIAN/control << 'EOF'
Package: knockd-client
Version: 0.1.0
Architecture: amd64
Depends: libwebkit2gtk-4.1-0, libgtk-3-0, libayatana-appindicator3-1
Description: Cross-platform port knocking client with SSH/web launcher
EOF

cp src-tauri/target/release/knockd-client packaging/deb/usr/bin/
cp src-tauri/icons/128x128.png packaging/deb/usr/share/icons/hicolor/256x256/apps/knockd-client.png
chmod 755 packaging/deb/usr/bin/knockd-client
dpkg-deb --build packaging/deb knockd-client_0.1.0_amd64.deb
```

### 8.3 Windows 交叉编译

```bash
# 安装 MinGW + Rust target
sudo apt install gcc-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu

# 使用完整 Tauri 流水线（嵌入前端）
pnpm build
pnpm tauri build --target x86_64-pc-windows-gnu
# NSIS 打包会失败（Linux 无 makensis），但二进制已正确嵌入前端
# 产物: src-tauri/target/x86_64-pc-windows-gnu/release/knockd-client.exe
```

> **注意**: 裸 `cargo build --release --target` 不会执行 `beforeBuildCommand`，前端不嵌入 → 运行时出现 `localhost 拒绝连接`。必须用 `pnpm tauri build --target`。

### 8.4 macOS 构建

必须在 macOS 上构建（Apple 代码签名要求）：
```bash
pnpm tauri build  # 自动生成 .dmg
```

### 8.5 浏览器扩展安装

```bash
# 安装 Native Messaging Host manifest
sudo ./install/install-linux.sh <extension-id> [chrome|edge|chromium|brave]

# 然后加载扩展: chrome://extensions → Dev mode → Load unpacked → extension/
```

扩展通过 `chrome.runtime.sendNativeMessage("com.knockd.client", ...)` 与二进制通信。
二进制检测到 `chrome-extension://` 参数自动进入 Native Host 模式。

### 8.6 版本号管理

在三个地方同步更新版本号：

```json
// tauri.conf.json
{ "version": "0.2.0" }

// Cargo.toml
[package]
version = "0.2.0"

// package.json
{ "version": "0.2.0" }
```

---

## 9. CI/CD

### 9.1 触发方式

| 触发 | 行为 |
|------|------|
| `git push --tags` (tag 以 `v` 开头) | 三平台构建 + 自动创建 GitHub Release |
| workflow_dispatch (手动) | 三平台构建，产物在 Actions Artifacts 下载 |

### 9.2 工作流架构

```
push tag v0.1.0
    ├── build-linux   (ubuntu-24.04) ── .deb + .rpm + .AppImage
    ├── build-windows (windows-latest) ── .msi + NSIS .exe (含 WebView2)
    └── build-macos   (macos-latest) ── .dmg + .app
              │
              └── release job ── 收集所有产物 → GitHub Release
```

### 9.3 产物清单

| Job | 产物 |
|-----|------|
| **linux-deb** | `Knockd Client_*.deb`, `Knockd Client_*.rpm`, `Knockd Client_*.AppImage`, `knockd-client` |
| **windows-installer** | `Knockd Client_*.msi`, `Knockd Client_*-setup.exe` (NSIS), `knockd-client.exe` |
| **macos-dmg** | `Knockd Client_*.dmg`, `Knockd Client.app`, `knockd-client` |

### 9.4 分支策略

```
main              # 稳定版本
├── develop       # 开发主分支
│   ├── feat/xxx  # 功能分支
│   └── fix/xxx   # 修复分支
└── release/x.y.z # 发布分支
```

### 9.3 提交规范

```
feat: add connection export feature
fix: knock sequence timeout on slow networks
docs: update installation guide
refactor: extract knock engine to separate module
chore: update dependencies
```

---

## 10. 发布与分发

### 10.1 发布检查清单

- [ ] 所有测试通过 (`cargo test`, `pnpm vitest`)
- [ ] 无 lint 错误 (`cargo clippy`, `pnpm tsc --noEmit`)
- [ ] 版本号已更新（三处：`tauri.conf.json`, `Cargo.toml`, `package.json`）
- [ ] CHANGELOG 已更新
- [ ] Git tag 已创建 (`git tag v0.2.0`)
- [ ] 所有平台构建成功
- [ ] 安装包已验证（安装→运行→卸载）
- [ ] README 截图已更新

### 10.2 自动更新

Tauri v2 内置更新支持。配置 `tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://releases.example.com/{{target}}/{{arch}}/{{current_version}}"
      ],
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

生成密钥对：
```bash
pnpm tauri signer generate -w ~/.tauri/knockd.key
```

### 10.3 分发渠道

| 渠道 | 格式 | 适用 |
|------|------|------|
| GitHub Releases | .deb, .msi, .dmg | 公开分发 |
| 自建 APT 仓库 | .deb | Debian/Ubuntu 用户 |
| Microsoft Store | .msix | Windows 用户 |
| Homebrew Cask | .dmg | macOS 用户 |
| Snap/Flatpak | snap/flatpak | Linux 通用 |

### 10.4 APT 仓库搭建

```bash
# 在服务器上
mkdir -p /var/www/apt/pool/main
cp knockd-client_0.1.0_amd64.deb /var/www/apt/pool/main/

# 生成索引
cd /var/www/apt
dpkg-scanpackages pool/main > dists/stable/main/binary-amd64/Packages
gzip -k dists/stable/main/binary-amd64/Packages

# 用户添加源
echo "deb [trusted=yes] https://apt.example.com/ stable main" \
  | sudo tee /etc/apt/sources.list.d/knockd.list
sudo apt update && sudo apt install knockd-client
```

---

## 11. 常见问题

### 11.1 构建错误

**`error: linker 'cc' not found`**
```bash
sudo apt install build-essential
```

**`could not find `libwebkit2gtk-4.1`**
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

**Rust 版本不兼容**
```bash
rustup update stable
```

**`npx tauri init` 交互式不可用**
在非交互式环境中手动创建项目结构（参考本项目的文件布局）。

### 11.2 运行时错误

**应用启动闪退**
```bash
RUST_LOG=debug ./knockd-client  # 查看日志
```

**数据库权限问题**
数据库文件创建在用户数据目录，通常不会遇到权限问题。如遇到：
```bash
chmod 755 ~/.local/share/knockd-client
```

**WebView 问题 (Linux)**
确保安装了 webkit2gtk:
```bash
sudo apt install libwebkit2gtk-4.1-0
```

### 11.3 调试技巧

**Rust 后端调试：**
```bash
# 添加更多日志
RUST_LOG=trace pnpm tauri dev

# 使用 dbg! 宏打印变量
dbg!(&connection);
```

**前端调试：**
在 Tauri 窗口中按 `Ctrl+Shift+I` (Linux/Windows) 或 `Cmd+Option+I` (macOS) 打开 DevTools。

**IPC 调试：**
在 `api.ts` 中添加拦截日志：
```typescript
const invokeWithLog = async (cmd: string, args?: any) => {
  console.log(`IPC → ${cmd}`, args);
  const result = await invoke(cmd, args);
  console.log(`IPC ← ${cmd}`, result);
  return result;
};
```

### 11.4 跨平台差异处理

```rust
// 在 Rust 中使用条件编译
#[cfg(target_os = "windows")]
fn platform_specific() { /* Windows 逻辑 */ }

#[cfg(target_os = "linux")]
fn platform_specific() { /* Linux 逻辑 */ }

#[cfg(target_os = "macos")]
fn platform_specific() { /* macOS 逻辑 */ }
```

### 11.5 性能优化

- **Rust**: 使用 `cargo build --release`（生产构建）
- **前端**: Tailwind purge 只保留使用的类
- **数据库**: 使用索引、批量操作
- **二进制大小**: 使用 `strip` 去除调试符号

```bash
# 精简二进制
strip src-tauri/target/release/knockd-client
```

---

## 附录 A: 开发环境速查

```bash
# 首次设置
git clone <repo> && cd knockd-client
pnpm install
rustup target add x86_64-pc-windows-gnu  # 如需 Windows 交叉编译

# 日常开发
pnpm tauri dev          # 启动
cargo check             # 快速 Rust 检查
pnpm tsc --noEmit       # TypeScript 检查

# 构建
pnpm tauri build        # 当前平台
cargo build --release --target x86_64-pc-windows-gnu  # Windows 交叉编译

# 测试
cargo test
pnpm vitest

# 版本发布
git tag v0.2.0
git push --tags
```

## 附录 B: 依赖版本矩阵

| 包 | 版本 | 用途 |
|----|------|------|
| tauri | 2.11 | 桌面框架 |
| tauri-build | 2.6 | 构建脚本 |
| tauri-plugin-dialog | 2.7 | 文件选择器 |
| rusqlite | 0.31 | SQLite (bundled-sqlcipher) |
| ed25519-dalek | 2 | Ed25519 签名 |
| curve25519-dalek | 4 | X25519 ECDH |
| aes-gcm | 0.10 | AES-256-GCM |
| sha2 | 0.10 | SHA-256 |
| hmac | 0.12 | HMAC-SHA256 |
| hex | 0.4 | Hex 编解码 |
| serde | 1.0 | 序列化 |
| react | 18.3 | UI 框架 |
| typescript | 5.6+ | 类型系统 |
| vite | 6.0+ | 构建工具 |
| tailwindcss | 3.4 | CSS 框架 |

## 附录 B: 调试加密数据库

```bash
# 安装 sqlcipher
sudo apt install sqlcipher

# 获取密钥并查询
FP=$(knockd-client --activate)
KEY=$(echo -n "${FP}|knockd-sqlcipher-v1" | sha256sum | cut -d' ' -f1)
sqlcipher ~/.local/share/knockd-client/knockd.db \
  "PRAGMA key=\"x'${KEY}'\"; SELECT name,conn_type,auth_method,host FROM connections;"
```

> 注意：必须用 `echo -n`，`<<<` 会带换行符导致 hash 不一致。
