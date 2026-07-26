#!/usr/bin/env bash
set -euo pipefail

log() {
	printf "[cleanup] %s\n" "$*"
}

if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
	log "Re-running with sudo..."
	exec sudo -E bash "$0" "$@"
fi

for dir in /mnt/recordings/cam1 /mnt/recordings/cam2; do
	if [[ -d "$dir" ]]; then
		log "Cleaning $dir..."
		find "$dir" -mindepth 1 -delete
	else
		log "Directory not found, creating: $dir"
		mkdir -p "$dir"
	fi
done

log "Done."
