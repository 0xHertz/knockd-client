# Knockd Client

A cross-platform desktop GUI client for managing SSH connections and web URLs with [port knocking](https://en.wikipedia.org/wiki/Port_knocking) support.

Built with **Tauri v2** (Rust + React + TypeScript + Tailwind CSS).

## Features

- **Port Knocking** — Send UDP/TCP knock sequences before connecting (standard Linux knockd protocol)
- **SSH Connection Manager** — Store and manage SSH connections with one-click knock + connect
- **Web URL Launcher** — Knock then open websites in your default browser
- **Multi-SSH-Client Support** — Auto-detects 10+ SSH clients: Windows Terminal, OpenSSH, PuTTY, KiTTY, MobaXterm, Xshell, NxShell, Bitvise, SecureCRT, Termius, iTerm2
- **Cross-Platform** — Windows, macOS, and Linux
- **SQLite Storage** — All connections stored locally in SQLite
- **Dark Theme** — Modern dark UI with search, filter, and quick actions

## Screenshots

*(Add screenshots here after running the app)*

## Installation

### Download Pre-built Packages

| Platform | Package |
|----------|---------|
| **Ubuntu/Debian** | `knockd-client_0.1.0_amd64.deb` |
| **Windows** | `knockd-client_0.1.0_x64-setup.exe` (NSIS installer) |
| **macOS** | *(coming soon)* |

### Build from Source

#### Prerequisites

- **Rust** 1.88+ (via [rustup](https://rustup.rs))
- **Node.js** 18+ and **pnpm**
- **Tauri system dependencies** (see [Tauri docs](https://v2.tauri.app/start/prerequisites/))

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev \
  libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev \
  libjavascriptcoregtk-4.1-dev libsoup-3.0-dev
```

#### Build

```bash
git clone <repo-url> knockd-client
cd knockd-client

# Install frontend dependencies
pnpm install

# Build for production
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
3. Fill in host, port, username (for SSH) or launch URL (for Web)
4. Configure the **knock sequence** as JSON:

```json
[{"protocol":"udp","port":7000},{"protocol":"tcp","port":8000},{"protocol":"udp","port":9000}]
```

5. Click **Save**

### Connecting

1. Find your connection in the list (use search/filter if needed)
2. Click **🚀 Knock & Connect**
3. The app will: send knock packets → launch your SSH client or browser

### Knock Port JSON Format

Each step specifies a protocol and port:

| Field | Type | Description |
|-------|------|-------------|
| `protocol` | `"udp"` or `"tcp"` | Knock protocol (defaults to connection's default protocol if empty) |
| `port` | `number` | Target port (1-65535) |

**Examples:**

Single UDP knock:
```json
[{"protocol":"udp","port":12345}]
```

Mixed TCP/UDP:
```json
[{"protocol":"tcp","port":4444},{"protocol":"udp","port":5555}]
```

Three-step sequence (standard Linux knockd):
```json
[{"protocol":"udp","port":7000},{"protocol":"tcp","port":8000},{"protocol":"udp","port":9000}]
```

### SSH Clients

The app auto-detects available SSH clients on your system:

| Platform | Detected Clients |
|----------|-----------------|
| **Windows** | Windows Terminal, OpenSSH (ssh), PuTTY, KiTTY, MobaXterm, Xshell, NxShell, Bitvise SSH, SecureCRT, Termius |
| **Linux** | OpenSSH (ssh), PuTTY, Termius |
| **macOS** | Terminal + ssh, iTerm2, Termius |

You can set a preferred client in **Settings**, or let it auto-detect each time.

## Configuration

All data is stored in SQLite at:

| Platform | Path |
|----------|------|
| **Linux** | `~/.local/share/knockd-client/knockd.db` |
| **macOS** | `~/Library/Application Support/knockd-client/knockd.db` |
| **Windows** | `%APPDATA%\knockd-client\knockd.db` |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | [Tauri v2](https://v2.tauri.app) |
| Backend | Rust (rusqlite, serde, tokio) |
| Frontend | React 18 + TypeScript + Vite |
| Styling | Tailwind CSS 3 |
| Database | SQLite (embedded, no server needed) |
| Packaging | .deb (Linux), NSIS .exe (Windows) |

## Project Structure

```
knockd/
├── src/                          # Frontend (React)
│   ├── App.tsx                   # Main layout
│   ├── api.ts                    # Tauri IPC bindings
│   ├── types.ts                  # TypeScript interfaces
│   └── components/
│       ├── ConnectionCard.tsx     # Connection card with actions
│       ├── ConnectionForm.tsx     # Add/edit modal form
│       └── SettingsPanel.tsx     # Preferences panel
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs                # Tauri setup & commands
│   │   ├── models.rs             # Data structures
│   │   ├── db.rs                 # SQLite CRUD operations
│   │   ├── knock.rs              # UDP/TCP port knocking
│   │   ├── launcher.rs           # SSH client detection & launch
│   │   └── commands.rs           # Tauri IPC command handlers
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

## License

MIT

## Acknowledgments

- [knockd](https://github.com/jvinet/knock) — The original port knocking daemon
- [Tauri](https://tauri.app) — Cross-platform desktop framework
- [Knock on Ports](https://github.com/impalex/knockonports) — Inspiration for the multi-protocol knock format
