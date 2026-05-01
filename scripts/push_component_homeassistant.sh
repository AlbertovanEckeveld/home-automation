#!/usr/bin/env bash

set -e

COMPONENT_NAME="albertove"
CONTAINER_NAME="homeassistant"
TARGET_DIR="custom_components/${COMPONENT_NAME}"

LOCAL_COMPONENT_DIR="custom_components/${COMPONENT_NAME}"

echo "🔍 Checking local component..."

# Check if local component exists
if [ ! -d "${LOCAL_COMPONENT_DIR}" ]; then
  echo "❌ Component directory '${LOCAL_COMPONENT_DIR}' does not exist. Please run the build script first."
  exit 1
fi

echo "🔍 Finding Home Assistant volume..."

# Get volume or mount path for /config
MOUNT_INFO=$(docker inspect "${CONTAINER_NAME}" \
  --format '{{range .Mounts}}{{if eq .Destination "/config"}}{{.Type}}|{{.Name}}|{{.Source}}{{end}}{{end}}')

if [ -z "$MOUNT_INFO" ]; then
  echo "❌ Could not find /config mount for container ${CONTAINER_NAME}"
  exit 1
fi

TYPE=$(echo "$MOUNT_INFO" | cut -d'|' -f1)
NAME=$(echo "$MOUNT_INFO" | cut -d'|' -f2)
SOURCE=$(echo "$MOUNT_INFO" | cut -d'|' -f3)

echo "📦 Mount type: $TYPE"

# Determine actual path
if [ "$TYPE" == "volume" ]; then
  VOLUME_PATH="/var/lib/docker/volumes/${NAME}/_data"
elif [ "$TYPE" == "bind" ]; then
  VOLUME_PATH="$SOURCE"
else
  echo "❌ Unsupported mount type: $TYPE"
  exit 1
fi

echo "📁 Home Assistant config path: $VOLUME_PATH"

TARGET_PATH="${VOLUME_PATH}/${TARGET_DIR}"

echo "🧹 Cleaning old component if exists..."

# Remove old component if exists
if [ -d "$TARGET_PATH" ]; then
  sudo rm -rf "$TARGET_PATH"
  echo "🗑️ Removed old component"
fi

echo "📦 Creating directory structure..."
mkdir -p "$TARGET_PATH"

echo "🚀 Copying new component..."

# Copy component
cp -r "${LOCAL_COMPONENT_DIR}/." "$TARGET_PATH/"

echo "✅ Done! Component deployed to Home Assistant."