# prox

A fast terminal UI to see what's listening on your Linux ports — and kill it.

---

## Why

You've seen it: `Error: listen EADDRINUSE: address already in use :::3000`. Some forgotten dev server,
a zombie process, something you can't remember starting — it owns a port and won't let you work.
**prox** gives you a live, filterable table of every listening port on the machine, with the owning
process right next to it. Find the culprit, press `K`, confirm, and get back to work.

---

## Demo

![demo](docs/demo.gif)

<!-- TODO: record demo GIF -->

---

## Features

- Live table of all listening TCP and UDP ports with the owning process (PID, name, user, command)
- Real-time filter/search by port number, process name, PID, or username
- Sort by port, process name, or PID (toggle with `s`)
- Kill the selected process with a confirmation step — SIGTERM first, SIGKILL fallback
- Auto-refresh every ~1 second; manual refresh with `r`
- Single static binary, no runtime dependencies

---

## Install

Requires a [Rust toolchain](https://rustup.rs/) (stable, 1.70+).

```bash
git clone https://github.com/elias-santos/prox
cd prox
cargo install --path .
```

Pre-built binaries for common Linux targets will be available via GitHub Releases in a future version.

---

## Usage

```bash
prox
```

### Keybindings

| Key | Action |
|---|---|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `/` | Enter filter mode — type to filter live |
| `Esc` | Clear filter / cancel |
| `s` | Toggle sort (port → name → PID) |
| `r` | Refresh now |
| `K` / `Delete` | Kill selected process (prompts for confirmation) |
| `s` / `y` | Confirm kill |
| `n` / `Esc` | Cancel kill |
| `q` | Quit |

---

## Platform

**Linux only** for now. `prox` reads `/proc/net/{tcp,tcp6,udp,udp6}` and `/proc/<pid>/fd` to
discover sockets and their owning processes — this is Linux-specific.

macOS and Windows support is planned for a future release.

---

## How it works

`prox` parses `/proc/net/tcp`, `/proc/net/tcp6`, `/proc/net/udp`, and `/proc/net/udp6` to collect
all sockets in the listening state, extracting the local port and socket inode for each. It then
walks `/proc/<pid>/fd` for every running process to find which process owns each socket inode,
resolving the UID to a username via `/etc/passwd`. The result is a sorted, deduplicated table
updated every second.

---

## License

MIT — see [LICENSE](LICENSE).
