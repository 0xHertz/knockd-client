# Knockd Client

A cross-platform desktop GUI client for managing SSH connections and web URLs with [port knocking](https://en.wikipedia.org/wiki/Port_knocking) support.

Built with **Tauri v2** (Rust + React + TypeScript + Tailwind CSS).

## Features

- **Port Knocking** — Send UDP/TCP knock sequences before connecting (standard Linux knockd protocol)
- **SSH Connection Manager** — Store and manage SSH connections with one-click knock + connect
- **Web URL Launcher** — Knock then open websites in your default browser
- **Multi-SSH-Client Support** — Auto-detects via filesystem + Windows Registry + manual override
- **Custom SSH Clients** — File picker to add any SSH client not in default paths
- **Cross-Platform** — Windows (native), macOS, Linux (.deb / .rpm / .AppImage)
- **SQLite Storage** — All connections stored locally
- **Dark Theme** — Modern UI with search, filter, and quick actions
- **CI/CD** — GitHub Actions native builds for all three platforms

## Screenshots

![main](https://raw.githubusercontent.com/0xHertz/img/main/img/main.PNG)

## Installation

### Download Pre-built Packages

| Platform | Package | Source |
|----------|---------|--------|
| **Ubuntu/Debian** | `.deb` | [Releases](https://github.com/0xHertz/knockd-client/releases) |
| **Fedora/RHEL** | `.rpm` | [Releases](https://github.com/0xHertz/knockd-client/releases) |
| **Linux (any)** | `.AppImage` | [Releases](https://github.com/0xHertz/knockd-client/releases) |
| **Windows** | `.exe` NSIS installer (with WebView2) | [Releases](https://github.com/0xHertz/knockd-client/releases) |
| **macOS** | `.dmg` | [Releases](https://github.com/0xHertz/knockd-client/releases) |

> **Windows note**: Requires [WebView2 Runtime](https://go.microsoft.com/fwlink/p/?LinkId=2124703) (pre-installed on Windows 11). The NSIS installer includes an embedded bootstrapper.

### Build from Source

#### Prerequisites

- **Rust** 1.88+ (via [rustup](https://rustup.rs))
- **Node.js** 18+ and **pnpm**

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev \
  libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev \
  libjavascriptcoregtk-4.1-dev libsoup-3.0-dev
```

```bash
git clone https://github.com/0xHertz/knockd-client.git
cd knockd-client
pnpm install
pnpm tauri build
# Binary at: src-tauri/target/release/knockd-client
```

#### Development

```bash
pnpm tauri dev
```

## Usage

### Adding a Connection

1. Click **+ New Connection**
2. Choose **SSH** or **Web**
3. Fill in host, port, username (SSH) or launch URL (Web)
4. Configure the **knock sequence** as JSON:

```json
[{"protocol":"udp","port":7000},{"protocol":"tcp","port":8000},{"protocol":"udp","port":9000}]
```

5. Click **Save**

### Knock Port JSON Format

| Field | Type | Description |
|-------|------|-------------|
| `protocol` | `"udp"` or `"tcp"` | Per-step protocol (falls back to connection default) |
| `port` | `number` | Target port (1-65535) |

```json
[{"protocol":"udp","port":12345}]
[{"protocol":"tcp","port":4444},{"protocol":"udp","port":5555}]
```

### SSH Clients

Auto-detection priority (Windows):

1. **Known paths** — `C:\Program Files\...`, `%LOCALAPPDATA%\...`, `Downloads\...`
2. **Windows Registry** — `HKLM\SOFTWARE\...\App Paths` and vendor-specific keys
3. **PATH** — `where.exe` lookup (suppressed console window)

| Platform | Auto-detected Clients |
|----------|----------------------|
| **Windows** | Windows Terminal, OpenSSH, PuTTY, KiTTY, MobaXterm, Xshell, NxShell, Bitvise SSH, SecureCRT, Termius |
| **Linux** | OpenSSH, PuTTY, Termius |
| **macOS** | Terminal + ssh, iTerm2, Termius |

### Custom SSH Clients

If your SSH tool is not auto-detected (e.g. portable version, custom location):

1. Open **Settings** → **Custom SSH Clients**
2. Click **+ Add**
3. Click **📂** to browse and select the `.exe`
4. The name auto-fills from the filename (editable)
5. Click **Save**

The custom client then appears in the dropdown when creating/editing an SSH connection.

## Configuration

| Platform | Database Path |
|----------|--------------|
| **Linux** | `~/.local/share/knockd-client/knockd.db` |
| **macOS** | `~/Library/Application Support/knockd-client/knockd.db` |
| **Windows** | `%APPDATA%\knockd-client\knockd.db` |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | [Tauri v2](https://v2.tauri.app) |
| Backend | Rust (rusqlite, serde, dirs, glob) |
| Frontend | React 18 + TypeScript + Vite |
| Styling | Tailwind CSS 3 |
| Database | SQLite (bundled, no server) |
| Dialogs | tauri-plugin-dialog (native file picker) |
| CI/CD | GitHub Actions (native builds per platform) |

## Project Structure

```
knockd/
├── .github/workflows/build.yml    # CI/CD pipeline
├── src/                           # Frontend (React)
│   ├── App.tsx                    # Main layout + state
│   ├── api.ts                     # Tauri IPC bindings
│   ├── types.ts                   # TypeScript interfaces
│   └── components/
│       ├── ConnectionCard.tsx      # Card: knock/connect/edit/delete
│       ├── ConnectionForm.tsx      # Modal: add/edit connection
│       └── SettingsPanel.tsx       # Settings + custom SSH file picker
├── src-tauri/                     # Backend (Rust)
│   ├── src/
│   │   ├── main.rs                # Entry point
│   │   ├── lib.rs                 # Plugin setup + command registration
│   │   ├── models.rs              # Connection, SshClient, KnockStep
│   │   ├── db.rs                  # SQLite CRUD + schema init
│   │   ├── knock.rs               # UDP/TCP port knocking engine
│   │   ├── launcher.rs            # SSH detection (filesystem + registry) + launch
│   │   └── commands.rs            # Tauri IPC handlers
│   ├── capabilities/default.json  # Permissions (core, dialog)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── DEVELOPMENT.md                 # Full development guide
├── README.md
├── package.json
└── vite.config.ts
```

## License

MIT

## Acknowledgments

- [knockd](https://github.com/jvinet/knock) — Original port knocking daemon
- [Tauri](https://tauri.app) — Cross-platform desktop framework
- [Knock on Ports](https://github.com/impalex/knockonports) — Multi-protocol knock format inspiration
