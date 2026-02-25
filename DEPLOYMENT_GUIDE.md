# Jacox Deployment Guide 🚀

This guide explains how to deploy Jacox in production environments, both with and without Docker.

---

## 🏗 Option 1: Standalone Binary (Non-Docker)
Rust allows you to compile Jacox into a high-performance binary that you can run directly on a server.

### 1. Build the Release Binary
On your build machine (or server):
```bash
cargo build --release
```
The resulting binary will be at `target/release/jacox`.

### 2. Create a Deployment Bundle 📦
To run Jacox on a remote server, you need three things:
1.  **The Binary**: `target/release/jacox`
2.  **Configuration**: `config.yaml`
3.  **Static Assets**: The `static/` directory (for the Playground and Landing Page).

**Structure:**
```text
deploy/
├── jacox        (the binary)
├── config.yaml  (your production config)
└── static/      (the folder)
```

### 3. Running as a System Service (Linux)
To keep Jacox running in the background, use a `systemd` unit:

**`/etc/systemd/system/jacox.service`**
```ini
[Unit]
Description=Jacox LLM Server
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/home/youruser/jacox
ExecStart=/home/youruser/jacox/jacox serve
Restart=always
Environment=JACOX_SERVER_HOST=0.0.0.0
Environment=OPENAI_API_KEY=sk-...

[Install]
WantedBy=multi-user.target
```

---

## 🐧 Option 2: Static Linking (Ultra-Portable)
If you want a binary that runs on any Linux distribution without needing shared libraries (like GLIBC), you can use the `musl` target.

### 1. Install Musl Target
```bash
rustup target add x86_64-unknown-linux-musl
```

### 2. Build Static Binary
```bash
cargo build --release --target x86_64-unknown-linux-musl
```
*Note: You may need to install `musl-tools` on your system.*

---

## 🐳 Option 3: Docker Compose (Recommended)
The easiest way to deploy with all dependencies (DuckDB, OpenSSL) pre-configured.

### 🚀 Launch
```bash
docker compose up -d
```
Your data is persisted in a Docker volume, and the server is automatically restarted if it crashes.

---

## ☁️ Where to host?
1.  **VPS (DigitalOcean, Hetzner, AWS EC2)**: Best for Standalone Binary or Docker.
2.  **PaaS (Fly.io, Railway, Render)**: Best for Docker.
3.  **Local Home Server**: Great for Ollama-based setups.
