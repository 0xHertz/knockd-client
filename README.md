# Knockd Client

Cross-platform desktop GUI for managing SSH and web connections with [port knocking](https://en.wikipedia.org/wiki/Port_knocking) and [Single Packet Authorization](https://en.wikipedia.org/wiki/Single-packet_authorization).

Built with **Tauri v2** (Rust + React + TypeScript + Tailwind CSS).

## Features

- **Port Knocking** — UDP/TCP knock sequences (standard knockd protocol)
- **KnockPass SPA** — Ed25519 signature + AES-256-GCM encrypted Single Packet Authorization
- **Dynamic Port** — Time-based rotating UDP port (changes every 60s)
- **Admin/User Enrollment** — Admin generates keys, users import encrypted keys via X25519 ECDH
- **SSH + Web** — Knock then auto-launch SSH client or browser
- **10+ SSH Clients** — Auto-detection via filesystem + Windows Registry + manual file picker
- **Browser Extension** — Chrome/Edge popup with auto-knock on navigation, site list from desktop app
- **Encrypted Key Storage** — AES-256-GCM with device-fingerprint-derived key, stored in SQLite
- **Cross-Platform** — Linux (.deb/.rpm/.AppImage), Windows (.exe/.msi), macOS (.dmg)
- **Dark Theme** — Modern UI with search, filter, quick actions
- **CI/CD** — GitHub Actions native builds

## Installation

### Pre-built Packages

| Platform | Format | Source |
|----------|--------|--------|
| **Ubuntu/Debian** | `.deb` | [Releases](https://github.com/0xHertz/knockd-client/releases) |
| **Fedora/RHEL** | `.rpm` | [Releases](https://github.com/0xHertz/knockd-client/releases) |
| **Linux** | `.AppImage` | [Releases](https://github.com/0xHertz/knockd-client/releases) |
| **Windows** | `.exe` NSIS installer | [Releases](https://github.com/0xHertz/knockd-client/releases) |
| **macOS** | `.dmg` | [Releases](https://github.com/0xHertz/knockd-client/releases) |

> **Windows**: Requires [WebView2 Runtime](https://go.microsoft.com/fwlink/p/?LinkId=2124703) (pre-installed on Win11). NSIS installer embeds bootstrapper.

### Browser Extension Setup

```bash
# Install native messaging host manifest
sudo ./install/install-linux.sh <chrome-extension-id> [chrome|edge|chromium|brave]
```

Then load the extension: `chrome://extensions` → Developer mode → Load unpacked → select `extension/`

### Build from Source

```bash
# Prerequisites (Ubuntu/Debian)
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev \
  libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev \
  libjavascriptcoregtk-4.1-dev libsoup-3.0-dev

git clone https://github.com/0xHertz/knockd-client.git
cd knockd-client
pnpm install
pnpm tauri build
```

## Usage

### Port Knocking (knockd)

Create an SSH or Web connection → configure knock ports as JSON:

```json
[{"protocol":"udp","port":7000},{"protocol":"tcp","port":8000},{"protocol":"udp","port":9000}]
```

Click **🚀 Knock & Connect** → knock packets sent → SSH client or browser launches.

### KnockPass SPA

#### Admin (Generate Site Keys)

1. Create SSH or Web connection → select **🔐 KnockPass SPA**
2. Choose **Admin** tab → **Generate Keys**
3. Copy **Public Key** → configure on server
4. Fill Site ID, host, port → **Save** (auto-encrypts private key to SQLite)

#### User (Import Encrypted Key)

1. Select **User** tab → **Generate X25519 Keys** (first time only)
2. Copy X25519 public key → send to admin
3. Admin encrypts site key (Settings → Admin Tool)
4. Paste encrypted blob → **Decrypt & Import** → **Save**

#### Admin Tool

Settings → **Admin: Encrypt Site Key**: select site → paste user's X25519 public key → **Encrypt** (or **Batch CSV** for multiple users). Output copied or saved to file.

### Knock Port JSON

| Field | Type | Description |
|-------|------|-------------|
| `protocol` | `"udp"` or `"tcp"` | Per-step protocol |
| `port` | `number` | Target port (1-65535) |

### SSH Clients

Auto-detection: known paths → Windows Registry → PATH.

| Platform | Clients |
|----------|---------|
| **Windows** | Win Terminal, OpenSSH, PuTTY, KiTTY, MobaXterm, Xshell, NxShell, Bitvise, SecureCRT, Termius |
| **Linux** | OpenSSH, PuTTY, Termius |
| **macOS** | Terminal + ssh, iTerm2, Termius |

Custom clients: Settings → **Custom SSH Clients** → 📂 select .exe.

### Browser Extension

- **Popup**: Shows SPA web sites from desktop app, manual knock button, auto-knock toggle
- **Auto-intercept**: Navigating to protected URL → blocked page → auto-sends SPA → redirects
- **Refresh**: Popup opens → fetches latest site list from native host via SQLite

## Configuration

| Item | Path |
|------|------|
| **Database** | `~/.local/share/knockd-client/knockd.db` (Linux) |
| **Extension manifest** | `~/.config/google-chrome/NativeMessagingHosts/com.knockd.client.json` |

## Security

| Layer | Mechanism |
|-------|-----------|
| SPA Encryption | AES-256-GCM |
| SPA Signature | Ed25519 (HMAC-SHA256 fallback) |
| Key Storage | AES-256-GCM, key = SHA256(device fingerprint + pepper) |
| Key Distribution | X25519 ECDH + AES-256-GCM (admin → user) |
| Dynamic Port | HMAC-SHA256 time-slot (changes every 60s) |
| Database File | SQLCipher AES-256, key derived from device fingerprint |
| Port Sequences | AES-256-GCM encrypted in SQLite |

### Debugging Encrypted Database

```bash
FP=$(knockd-client --activate)
KEY=$(echo -n "${FP}|knockd-sqlcipher-v1" | sha256sum | cut -d' ' -f1)
sqlcipher ~/.local/share/knockd-client/knockd.db \
  "PRAGMA key=\"x'${KEY}'\"; SELECT name,conn_type,auth_method,host FROM connections;"
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri v2](https://v2.tauri.app) |
| Backend | Rust (rusqlite, ed25519-dalek, curve25519-dalek, aes-gcm, sha2, hmac) |
| Frontend | React 18 + TypeScript + Vite + Tailwind CSS 3 |
| Database | SQLite (bundled) |
| Extension | Chrome Manifest V3, Native Messaging |
| CI/CD | GitHub Actions |

## Project Structure

```
knockd/
├── src/                              # Frontend (React)
│   ├── App.tsx / api.ts / types.ts
│   └── components/
│       ├── ConnectionCard.tsx         # Connection card with SPA badge
│       ├── ConnectionForm.tsx         # Add/edit with KnockPass enrollment
│       └── SettingsPanel.tsx          # Settings + admin tool + custom SSH
├── src-tauri/                        # Backend (Rust)
│   └── src/
│       ├── main.rs                   # Entry + native host mode
│       ├── lib.rs                    # Plugin setup + commands
│       ├── models.rs                 # Connection, SshClient
│       ├── db.rs                     # SQLite CRUD + migration
│       ├── knock.rs                  # Port knocking
│       ├── knockpass.rs              # SPA: Ed25519 + X25519 + AES + dyn port
│       ├── crypto_store.rs           # AES-256-GCM encrypted key storage
│       ├── spa_cmds.rs               # SPA encrypt/decrypt commands
│       ├── launcher.rs               # SSH detection + launch
│       └── commands.rs               # Tauri IPC handlers
├── extension/                        # Browser Extension (Manifest V3)
│   ├── manifest.json / background.js
│   ├── popup.html/js/css
│   ├── blocked.html/js/css
│   ├── storage.js
│   ├── icons/
│   └── native-host/
├── install/                          # Install scripts
│   └── install-linux.sh
├── .github/workflows/build.yml       # CI/CD
├── DEVELOPMENT.md
└── README.md
```

## License

MIT
