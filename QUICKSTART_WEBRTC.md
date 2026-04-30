# WebRTC + Playback Setup - Snelle Checklist

## 🚀 Stap 1: Docker + go2rtc starten

```bash
# Generate go2rtc config auto
python3 scripts/gen_go2rtc_config.py 100.80.163.32 8080 > go2rtc.yaml

# Start alles
docker-compose up -d
```

Controleer:
```bash
docker-compose logs go2rtc
curl http://localhost:8555  # WebRTC port check
```

## 🏠 Stap 2: Home Assistant integratie

1. **Herstart Home Assistant**
2. Ga naar: **Instellingen → Apparaten & services → Integraties**
3. Zoek op **AlbertoVE** → voeg toe
4. Vul in: `Host: 100.80.163.32` en `Port: 8080`

## 📹 Stap 3: Nieuwe entiteiten checken

Na integratie zie je:

```
Per camera:
├── 🎥 Live view        (camera)     → WebRTC via go2rtc
├── 🎬 Playback         (camera)     → HLS playlists
├── 🔴 Recording        (binary_sensor) → Is recording?
└── 📼 Recent recordings (sensor)    → MP4 files + URLs

Systeem:
└── 💾 NVR storage      (sensor)     → 0.54% used
```

## ✅ Testen

### Live view testen
1. Open entiteit: `camera.albertove_cam2_live_view`
2. Click de card → stream afspeelt (WebRTC)

### Playback testen
1. Open entiteit: `camera.albertove_cam2_playback`
2. Toont HLS playlist van alle beschikbare mp4's

### Recent recordings testen
1. Open sensor: `sensor.albertove_cam2_recent_recordings`
2. State = aantal bestanden (bijv. "5")
3. Attributes → zie mp4 files met:
   - Filename
   - Size (MB)
   - Modified time
   - Direct URL (`http://...`)

## 🔧 Configuratie

### Opslag pad aanpassen (niet standaard `/recordings`)

In Home Assistant, in de integratie-opties:
```
Opslag pad: /mnt/ssd/nvr-backup
```

### WebRTC port veranderen

In `go2rtc.yaml`:
```yaml
webrtc:
  listen: :9999  # in plaats van 8555
```

## 🐛 Debug

### Live view niet beschikbaar?

```bash
# Check NVR streams
curl http://100.80.163.32:8080/api/cameras | jq .

# Check go2rtc
curl http://localhost:1984/api/streams

# Check WebRTC
curl -I http://localhost:8555
```

### Geen recordings zichtbaar?

```bash
# Check opslag
ls -la /recordings/cam2/
```

## 📚 Verder lezen

Zie `WEBRTC_SETUP.md` voor volledige documentatie.

