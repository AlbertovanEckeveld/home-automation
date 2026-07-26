#!/usr/bin/env bash
set -euo pipefail

# Raspberry Pi OS Lite 64-bit (Pi 4GB) setup
# Prepares system to build/run Rust applications and applies sane defaults.

log() {
	printf "[setup] %s\n" "$*"
}

if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
	log "Re-running with sudo..."
	exec sudo -E bash "$0" "$@"
fi

export DEBIAN_FRONTEND=noninteractive

log "Updating system packages..."
apt-get update -y
apt-get full-upgrade -y

log "Installing base dependencies for Rust builds..."
apt-get install -y \
	build-essential \
	pkg-config \
	libssl-dev \
	libclang-dev \
	clang \
	lld \
	cmake \
	git \
	curl \
	ca-certificates \
	unzip \
	zip \
	tar \
	jq \
	htop \
	ffmpeg \
	ufw \
	fail2ban \
	openssh-server

log "Enabling time sync..."
systemctl enable --now systemd-timesyncd >/dev/null 2>&1 || true

log "Configuring firewall (UFW)..."
ufw allow OpenSSH >/dev/null 2>&1 || true
ufw --force enable >/dev/null 2>&1 || true

log "Enabling fail2ban..."
systemctl enable --now fail2ban >/dev/null 2>&1 || true

log "Installing Rust toolchain (rustup)..."
RUST_USER="${SUDO_USER:-}" 
if [[ -z "$RUST_USER" || "$RUST_USER" == "root" ]]; then
	log "No non-root user detected; skipping rustup install. Run as a normal user with sudo."
else
	USER_HOME="$(getent passwd "$RUST_USER" | cut -d: -f6)"
	if [[ -z "$USER_HOME" || ! -d "$USER_HOME" ]]; then
		log "Could not find home directory for $RUST_USER; skipping rustup install."
	else
		if [[ ! -x "$USER_HOME/.cargo/bin/rustup" ]]; then
			su - "$RUST_USER" -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable'
		else
			su - "$RUST_USER" -c "$USER_HOME/.cargo/bin/rustup update"
		fi

		# Ensure Rust is on PATH for all users
		if [[ ! -f /etc/profile.d/rust.sh ]]; then
			cat >/etc/profile.d/rust.sh <<'EOF'
export PATH="$HOME/.cargo/bin:$PATH"
EOF
		fi
	fi
fi

log "Cleaning up..."
apt-get autoremove -y
apt-get autoclean -y

log "Done. Reboot recommended."
