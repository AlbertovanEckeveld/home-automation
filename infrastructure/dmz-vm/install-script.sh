
#!/usr/bin/env bash

set -euo pipefail

echo "======================================"
echo " Docker & NetBird installatie"
echo "======================================"

# Controleer rootrechten
if [ "$(id -u)" -ne 0 ]; then
    echo "Voer dit script uit met sudo:"
    echo "sudo bash $0"
    exit 1
fi

# 1. Verwijder conflicterende pakketten
echo "[1/5] Oude Docker-pakketten verwijderen..."

apt-get remove -y \
    docker.io \
    docker-compose \
    docker-compose-v2 \
    docker-doc \
    docker-buildx \
    podman-docker \
    containerd \
    runc || true

# 2. Benodigde afhankelijkheden installeren
echo "[2/5] Benodigde pakketten installeren..."

apt-get update
apt-get install -y ca-certificates curl

# 3. Docker GPG-key toevoegen
echo "[3/5] Docker repository configureren..."

install -m 0755 -d /etc/apt/keyrings

curl -fsSL https://download.docker.com/linux/ubuntu/gpg \
    -o /etc/apt/keyrings/docker.asc

chmod a+r /etc/apt/keyrings/docker.asc

# Docker APT repository toevoegen
cat > /etc/apt/sources.list.d/docker.sources <<EOF
Types: deb
URIs: https://download.docker.com/linux/ubuntu
Suites: $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}")
Components: stable
Architectures: $(dpkg --print-architecture)
Signed-By: /etc/apt/keyrings/docker.asc
EOF

apt-get update

# 4. Docker Engine en Docker Compose installeren
echo "[4/5] Docker Engine en Docker Compose installeren..."

apt-get install -y \
    docker-ce \
    docker-ce-cli \
    containerd.io \
    docker-buildx-plugin \
    docker-compose-plugin \
    jq \
    curl

# Docker inschakelen en starten
systemctl enable --now docker

# 5. NetBird installeren
echo "[5/5] NetBird installeren..."

curl -fsSL https://github.com/netbirdio/netbird/releases/latest/download/getting-started.sh | bash

# Installatie controleren
echo ""
echo "======================================"
echo " Installatie voltooid!"
echo "======================================"

docker --version
docker compose version
jq --version
curl --version | head -n 1

echo ""
echo "Docker status:"
systemctl status docker --no-pager
