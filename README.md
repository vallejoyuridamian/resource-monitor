# Resource Monitor

A lightweight desktop process monitor built with [Tauri](https://tauri.app/) + Vue 3. Pick the processes you care about, watch their CPU and RAM in real time, and inspect per-subprocess breakdowns.

## Features

- **Process picker** — search by name and pin any process to the dashboard
- **Live stats** — CPU (% of total machine capacity) and RAM (MB) updated every second
- **Sparkline history** — 60-second rolling chart per process
- **Subprocess breakdown** — expand a card to see individual threads/children sorted by RAM
- **Snapshot table** — capture a point-in-time view for comparison
- Linux-native: reads `/proc` directly for accuracy; cross-platform stub included for future Windows/macOS support

## Tech stack

| Layer | Technology |
|---|---|
| UI | Vue 3 + TypeScript |
| Shell | Tauri 2 (Rust) |
| System info | `sysinfo` crate |
| Build | Vite + `pnpm` |

## Dev

```bash
pnpm install
pnpm dev        # starts Vite + Tauri in watch mode
```

Requires Rust, `pnpm`, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

## Build

```bash
pnpm build      # produces a native binary in src-tauri/target/release/
```
