# Running Cortex IDE on Windows

A step-by-step guide to building and running Cortex IDE from source on Windows.

---

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| **Node.js** | >= 24.x | Frontend build tooling |
| **npm** | >= 10.x | Package manager (ships with Node.js) |
| **Rust** | >= 1.90 | Backend compilation |
| **Git** | >= 2.x | Source code management |
| **Visual Studio Build Tools 2022** | Latest | MSVC C/C++ compiler (required by native Rust crates) |
| **WebView2** | Latest | Desktop runtime (pre-installed on Windows 10 1803+ and Windows 11) |

---

## Step 1 — Install Visual Studio Build Tools 2022

The MSVC toolchain is required for compiling native Rust dependencies. Open **PowerShell** and run:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

This installs the "Desktop development with C++" workload. If you already have Visual Studio 2022 installed with the C++ workload, you can skip this.

---

## Step 2 — Install Rust

Download and run [rustup-init.exe](https://rustup.rs/).

Follow the on-screen prompts (the defaults are fine — it will select the `stable-x86_64-pc-windows-msvc` toolchain).

Open a **new** terminal after installation and verify:

```powershell
rustc --version   # Should be >= 1.90.0
cargo --version
```

---

## Step 3 — Install Node.js

**Option A** — Download directly from [nodejs.org](https://nodejs.org) (LTS or Current >= 24.x).

**Option B** — Using [nvm-windows](https://github.com/coreybutler/nvm-windows):

```powershell
nvm install 24
nvm use 24
```

Verify:

```powershell
node --version   # Should be >= 24.x
npm --version    # Should be >= 10.x
```

---

## Step 4 — Verify WebView2

Tauri uses WebView2 as the rendering engine on Windows. It ships pre-installed on Windows 10 (version 1803+) and Windows 11.

To check if it's installed, look for "Microsoft Edge WebView2 Runtime" in **Settings > Apps > Installed apps**.

If missing, download the [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

---

## Step 5 — Clone the Repository

```powershell
git clone https://github.com/CortexLM/cortex-ide.git
cd cortex-ide
```

---

## Step 6 — Install Dependencies

```powershell
npm install
```

---

## Step 7 — Configure Environment (Optional)

```powershell
copy .env.example .env
```

The `.env` file contains:

```env
VITE_API_URL=http://127.0.0.1:4096
```

Adjust the API URL if needed. The default value works for local development.

---

## Step 8 — Run in Development Mode

```powershell
npm run tauri:dev
```

This will:

1. Start the Vite dev server on `http://localhost:1420` (SolidJS frontend)
2. Compile the Tauri Rust backend
3. Launch the desktop application with hot-reload enabled

> **Note:** The first build takes **5–15 minutes** as Rust compiles ~400 crate dependencies. Subsequent builds are incremental and much faster.

---

## Step 9 — Production Build (Optional)

To create a Windows installer:

```powershell
npm run tauri:build
```

This produces an NSIS installer at:

```
src-tauri\target\release\bundle\nsis\Cortex IDE_0.1.0_x64-setup.exe
```

Run the installer to install Cortex IDE as a standard Windows application.

---

## Available Commands

| Command | Description |
|---------|-------------|
| `npm run tauri:dev` | Full development build with hot-reload |
| `npm run tauri:build` | Production build with NSIS installer |
| `npm run dev` | Start Vite dev server only (frontend) |
| `npm run build` | Build frontend for production |
| `npm run typecheck` | Run TypeScript type checking |
| `npm run test` | Run frontend unit tests (Vitest) |
| `npm run test:watch` | Run tests in watch mode |
| `npm run test:coverage` | Run tests with coverage report |

---

## Windows-Specific Features

Cortex IDE includes several Windows-specific features:

- **Shell support**: cmd.exe, PowerShell, PowerShell Core, Git Bash, Cygwin, MSYS2, Nushell
- **WSL integration**: Run commands and terminals inside Windows Subsystem for Linux distributions
- **PowerShell shell integration**: OSC 633 protocol for command detection and CWD tracking
- **Process sandboxing**: ACL-based security restrictions
- **Credential storage**: Integrated with Windows Credential Manager via OS keychain

---

## Troubleshooting

### Rust linker errors

Ensure Visual Studio Build Tools 2022 is installed with the "Desktop development with C++" workload:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

### WebView2 not found

Download the [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) from Microsoft.

### `cargo build` is very slow

First builds compile ~400 crates — this is expected. Speed up subsequent builds with sccache:

```powershell
cargo install sccache
$env:RUSTC_WRAPPER = "sccache"
npm run tauri:dev
```

To make this permanent, add `RUSTC_WRAPPER=sccache` to your system environment variables.

### Node.js version mismatch

```powershell
node --version
```

If the version is below 24, update via nvm-windows (`nvm install 24 && nvm use 24`) or download the latest from [nodejs.org](https://nodejs.org).

### Permission denied / access errors

Run your terminal as **Administrator**, or ensure your Windows user account has full read/write permissions on the project directory.

### Antivirus interference

Some antivirus software may flag or slow down the Rust compilation process. If builds are unusually slow or fail with file access errors, try adding the project directory and `%USERPROFILE%\.cargo` to your antivirus exclusion list.

---

## Further Reading

- [Full Build Guide (all platforms)](./BUILD.md)
- [MCP Server Setup](./MCP-SERVER.md)
- [Discord Community](https://discord.gg/cortexfoundation)
