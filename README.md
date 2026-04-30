# 🌌 AlbertoVE

**AlbertoVE** is a high-performance, Rust-based application for **automated RTSP camera health checking** and **minute-based video recording**. It stores recordings in a structured directory format and uses **PostgreSQL** for camera stream management.

---

## 🚀 Features

- ✅ Loads configuration from `config.toml`
- 📡 Connects to a PostgreSQL database to fetch camera streams
- 🛡️ Performs RTSP health checks using FFmpeg
- 🎥 Records 1-minute video segments for healthy cameras
- 🗂️ Saves recordings in a clean, configurable folder structure
- 💻 Works on both **Windows** and **Linux**

---

## ⚙️ Requirements

- [Rust (stable)](https://www.rust-lang.org/)
- Cargo (included with Rust)
- [PostgreSQL](https://www.postgresql.org/)
- [FFmpeg](https://ffmpeg.org/) (must be installed and in your `PATH`)
- Docker (optional, for cross-compilation or containerization)

---

## 🛠️ Configuration

Create a `config.toml` file in the project root:

```toml
[database]
name = "your_database"
user = "your_user"
password = "your_password"
host = "localhost"
port = 5432

[app]
records_path = "./records"
```

---

## 🗃️ Database
The camera table must exist with at least the following columns:
- name (string)
- rtsp_url (string)

---

## ️🧪 Usage

### ▶️  Build
Native (Windows or Linux)
```bash
  cargo build --release
```

### 🐋 Run with Docker Compose (optional)
```bash
  docker-compose up --build
```

---

## 📁 Project Structure
```plaintext
albertove-cloud/
├── src/
│   ├── config.rs      # Loads config from config.toml
│   ├── recorder.rs    # RTSP health checks & recording
│   └── main.rs        # Entry point
├── config.toml        # Your configuration file
├── records/           # Output directory for recordings
└── README.md
```

---

