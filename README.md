# clc-jpd

CLC's Just Pull Down - An anti-recoil solution for configurable weapon management with dynamic loadout support.

## Overview

clc-jpd is a desktop application built with Tauri that provides automated recoil compensation for supported games. The application features a Svelte-based frontend with a custom UI for managing weapons, loadouts, and keybinds, backed by a Rust core that handles low-level input simulation and recoil pattern execution.

### Key Features

- Game-specific weapon profiles with customizable recoil patterns
- Loadout system with primary and secondary weapon slots
- Category-based weapon organization
- Configurable keybinds for weapon switching and activation
- ACOG scope detection with separate sensitivity multipliers
- Hardware-based license key validation
- Auto-update support via GitHub releases
- Custom theme support with accent color customization

## Architecture

### Frontend (Svelte + TypeScript)
- Component-based UI using Svelte 5
- Tailwind CSS for styling with Flowbite components
- SvelteKit for routing and SSG
- Tauri IPC for backend communication

### Backend (Rust)
- Tauri 2.x framework for desktop app structure
- Windows API integration for input simulation
- Tokio async runtime for concurrent operations
- Parking lot for high-performance synchronization primitives
- HTTP client for license validation and updates

### Core Modules
- `recoil.rs` - Recoil pattern execution and mouse movement control
- `winapi.rs` - Windows API bindings for input simulation and hardware identification
- `types.rs` - Shared data structures and state management
- `lib.rs` - Tauri command handlers and application lifecycle

## Distribution

Pre-built releases are available at the [Release Repository](https://github.com/hiibolt/clc-jpd-releases).

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js v22.16.0 (LTS)](https://nodejs.org/en/download) or [Bun](https://bun.sh/)
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) - Select "Desktop Development with C++" workload
- Windows 10 1803 or later (WebView2 pre-installed) or install [Microsoft WebView2](https://tauri.app/start/prerequisites/#webview2) manually

### Installation

```bash
# Clone the repository
git clone https://github.com/hiibolt/clc-jpd.git
cd clc-jpd

# Install frontend dependencies
npm install
# or
bun install

# Install Rust dependencies (automatic on first build)
```

### Development Commands

```bash
# Run development server with hot reload
npm run dev
# or
bun run dev

# Build production application
npm run build
# or
bun run build

# Run type checking
npm run check
# or
bun run check

# Run type checking in watch mode
npm run check:watch
```

### Building the Application

```bash
# Build the Tauri application
npm run tauri build
# or
bun run tauri build
```

The compiled executable will be in `src-tauri/target/release/`.

## Configuration

The application stores configuration in a JSON format including:
- Keybind assignments
- Mouse sensitivity multipliers (standard and ACOG)
- Theme settings
- Grid layout preferences

Configuration is managed through the settings interface and persisted via Tauri's filesystem API.

## Project Structure

```
clc-jpd/
├── src/                    # Frontend source
│   ├── components/         # Svelte components
│   ├── layouts/           # Page layouts
│   ├── lib/               # Utilities and helpers
│   ├── routes/            # SvelteKit routes
│   └── stores/            # State management
├── src-tauri/             # Backend source
│   ├── src/
│   │   ├── lib.rs        # Main Tauri handlers
│   │   ├── recoil.rs     # Recoil system
│   │   ├── winapi.rs     # Windows API integration
│   │   └── types.rs      # Data structures
│   ├── capabilities/      # Tauri permissions
│   └── icons/            # Application icons
└── static/               # Static assets
```

## License

MIT

## IDE Setup

Recommended extensions for VS Code:
- [Svelte for VS Code](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)