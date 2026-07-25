# Pong

A Pong clone built with [Bevy](https://bevyengine.org/) 0.19. Play against an AI opponent — first to 5 wins.

**[Play in browser](https://kaonquantum.github.io/bevy-pong/)**

## Controls

| Key | Action |
|-----|--------|
| `W` / `↑` | Move paddle up |
| `S` / `↓` | Move paddle down |

## Download

Grab the latest binary from [Releases](https://github.com/KaonQuantum/bevy-pong/releases):

- **Mac (Apple Silicon)** — `pong-mac-arm`
- **Mac (Intel)** — `pong-mac-intel`
- **Windows** — `pong-windows.exe`
- **Linux** — `pong-linux`

On Mac/Linux you may need to mark the binary executable: `chmod +x pong-*`

## Building from source

```bash
git clone https://github.com/KaonQuantum/bevy-pong.git
cd bevy-pong
cargo run --release
```

**Linux** requires these system packages:
```bash
sudo apt-get install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

**Web** (requires [trunk](https://trunkrs.dev/)):
```bash
rustup target add wasm32-unknown-unknown
trunk serve
```
