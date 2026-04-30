# 🏠 AlbertoVE - Home Automation NVR

**AlbertoVE** is a high-performance, Rust-based **Network Video Recorder (NVR)** and API gateway for **Home Assistant**. It provides camera management, live streaming, video playback, and seamless integration with Home Assistant for complete home automation control.

**Designed for**: Security monitoring, camera management, and future home automation devices like automatic pet feeders.

---

## 🚀 Features

### 🎥 Current Features
- ✅ RTSP camera health monitoring
- 🔴 Live view streaming via WebRTC
- 📹 Configurable video segment recording
- 🗂️ Organized storage directory structure with automatic management
- 🏠 Home Assistant integration via custom component
- 📱 Web UI for camera management and playback
- ⚡ High-performance Rust backend with FFmpeg integration

### 🔮 Future Features
- 🍖 Automatic pet feeder control
  - Time-based scheduling
  - Manual up/down control via button
  - Real-time monitoring in Home Assistant
- 🔔 Motion detection and alerts
- 📊 Analytics and statistics
- 🔐 Advanced security features
- 🌐 Multi-user support

---

## ⚙️ Requirements

### System Requirements
- [Rust (stable)](https://www.rust-lang.org/) with Cargo
- [FFmpeg](https://ffmpeg.org/) (must be installed and in your `PATH`)
- Docker (for containerized deployment)
- Sufficient storage for video recordings

### Optional
- [PostgreSQL](https://www.postgresql.org/) (for camera database)
- Home Assistant (for integration and automation)

---

## 🛠️ Configuration

### Application Config
Create a `src/config.yaml` file in the project root:

```yaml
default_storage_folder: /mnt/records

cameras:
  - id: cam1
    rtsp_url: rtsp://admin:password@192.168.1.100:554/stream
    segment_time: 10
  - id: cam2
    rtsp_url: rtsp://admin:password@192.168.1.101:554/stream
    segment_time: 10
```

### Home Assistant Integration
Copy the custom component from `custom_components/albertove/` to your Home Assistant configuration directory for seamless integration.

---

## 📦 Storage

By default, recordings are stored in: `/mnt/records/`

The storage structure is organized as follows:
```
/mnt/records/
├── cam1/
│   ├── 2024-01-15/
│   │   ├── segment_0001.mp4
│   │   ├── segment_0002.mp4
│   │   └── ...
│   └── 2024-01-16/
└── cam2/
    └── ...
```

Configure the storage path in `src/config.yaml` with the `default_storage_folder` parameter.

---

## 🏠 Home Assistant Integration

AlbertoVE includes a custom Home Assistant component that provides:
- 🎥 **Camera entities** for live streaming
- 📺 **Binary sensors** for camera status
- 📊 **Sensor entities** for recording status and storage information
- 🎛️ **Future controls** for automation devices (pet feeder, etc.)

See `/custom_components/albertove/` for installation instructions.

---

## 📖 Quick Start

### 1️⃣ System Setup
Run the installation script to set up all dependencies:

```bash
sudo ./scripts/install.sh
```

This will install:
- Docker & Docker Compose
- FFmpeg
- Rust & Cargo
- Optional: Mount additional storage

### 2️⃣ Configure Cameras
Edit `src/config.yaml` with your camera details.

### 3️⃣ Build and Run

#### Native Build
```bash
cargo build --release
./target/release/albertove
```

#### Docker Compose
```bash
docker-compose up --build
```

### 4️⃣ Web Interface
Access the web UI at: `http://localhost:8080`

### 5️⃣ Home Assistant Integration
Copy the custom component and configure in Home Assistant.

---

## 🧪 Development

### Build
```bash
cargo build
```

### Build Release
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Format Code
```bash
cargo fmt
```

### Lint
```bash
cargo clippy
```

---

## 📁 Project Structure

```plaintext
home-automation/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── api.rs                  # REST API endpoints
│   ├── config.yaml             # Camera and storage configuration
│   ├── config/
│   │   ├── mod.rs              # Configuration management
│   │   └── camera.rs           # Camera configuration
│   ├── recorder/
│   │   ├── mod.rs              # Recording orchestration
│   │   └── ffmpeg.rs           # FFmpeg integration
│   ├── storage/
│   │   ├── mod.rs              # Storage management
│   │   └── local.rs            # Local file storage
│   └── supervisor/
│       ├── mod.rs              # Process supervision
│       └── watchdog.rs         # Health monitoring
├── custom_components/
│   └── albertove/              # Home Assistant integration
│       ├── __init__.py
│       ├── config_flow.py
│       ├── manifest.json
│       └── ...
├── scripts/
│   ├── install.sh              # System setup and dependency installation
│   ├── setup.sh                # Legacy setup (deprecated)
│   ├── clean-up.sh             # Cleanup script
│   └── gen_go2rtc_config.py    # WebRTC config generator
├── web/
│   ├── index.html              # Web UI
│   ├── app.js                  # Frontend application
│   └── styles.css              # Styling
├── docker-compose.yml          # Docker deployment configuration
├── Cargo.toml                  # Rust dependencies
├── config.toml.example         # Configuration template
└── README.md                   # This file
```

---

## 📝 License

This project is designed for personal home automation use.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and enhancement requests.

---

## 📚 Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Home Assistant Documentation](https://www.home-assistant.io/docs/)
- [FFmpeg Documentation](https://ffmpeg.org/documentation.html)
- [WebRTC Setup Guide](./WEBRTC_SETUP.md)
- [Quick Start Guide](./QUICKSTART_WEBRTC.md)
