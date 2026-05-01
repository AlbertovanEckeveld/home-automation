#!/usr/bin/env bash
# Installation Script for Home Automation Project
# This script sets up the development/deployment environment with:
# 1. Docker
# 2. FFMPEG
# 3. Rust and Cargo

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "${GREEN}=== $1 ===${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check if running on Linux
if [[ ! "$OSTYPE" == "linux-gnu"* ]]; then
    print_error "Dit script is ontworpen voor Linux. Huidiig OS: $OSTYPE"
    exit 1
fi

# Check if running as root or with sudo
if [ "$EUID" -ne 0 ]; then
    print_error "Dit script moet met sudo worden uitgevoerd"
    exit 1
fi

print_header "Start Home Automation Setup"

# ============================================
# 1. Update system packages
# ============================================
print_header "Systeem packages updaten"
apt-get update || apt update
apt-get upgrade -y || apt upgrade -y
print_success "Systeem packages bijgewerkt"

# ============================================
# 2. Install Docker
# ============================================
print_header "Docker installeren"

# Check if Docker is already installed
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version)
    print_info "Docker is al geïnstalleerd: $DOCKER_VERSION"
else
    print_info "Docker wordt gedownload en geïnstalleerd..."

    # Install required packages
    apt-get install -y ca-certificates curl gnupg lsb-release build-essential

    # Add Docker GPG key
    mkdir -p /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg

    # Add Docker repository
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

    # Update packages and install Docker
    apt-get update
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin

    # Start Docker service
    systemctl start docker
    systemctl enable docker

    print_success "Docker is geïnstalleerd"
fi

# ============================================
# 3. Add user to docker group
# ============================================
print_header "User alberto-adm toevoegen aan docker group"

if id "alberto-adm" &>/dev/null; then
    print_info "User alberto-adm gevonden"

    # Add user to docker group
    usermod -aG docker alberto-adm
    print_success "User alberto-adm is toegevoegd aan docker group"
    print_info "OPMERKING: De user moet uitloggen en inloggen om de groep wijzigingen van kracht te laten worden"
else
    print_error "User alberto-adm bestaat niet!"
    print_info "Creeer eerst de user met: useradd -m -s /bin/bash alberto-adm"
    exit 1
fi

# ============================================
# 4. Install FFMPEG
# ============================================
print_header "FFMPEG installeren"

if command -v ffmpeg &> /dev/null; then
    FFMPEG_VERSION=$(ffmpeg -version | head -n 1)
    print_info "FFMPEG is al geïnstalleerd: $FFMPEG_VERSION"
else
    print_info "FFMPEG wordt gedownload en geïnstalleerd..."

    apt-get install -y ffmpeg

    print_success "FFMPEG is geïnstalleerd"
fi

# ============================================
# 5. Install Rust and Cargo
# ============================================
print_header "Rust en Cargo installeren"

if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    print_info "Rust is al geïnstalleerd: $RUST_VERSION"
else
    print_info "Rust en Cargo worden gedownload en geïnstalleerd..."

    # Download and install Rust using rustup
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile default

    # Source the cargo environment
    source "$HOME/.cargo/env"

    print_success "Rust en Cargo zijn geïnstalleerd"
fi

# ============================================
# 6. Mount zusätzliche Festplatte (optional)
# ============================================
print_header "Extra schijven mounten (optioneel)"

# Function to get boot disk
get_boot_disk() {
    df / | tail -1 | awk '{print $1}' | sed 's/[0-9]*$//'
}

# Function to list available disks
list_available_disks() {
    local boot_disk=$(get_boot_disk)
    print_info "Boot schijf: $boot_disk"
    print_info ""
    print_info "Beschikbare schijven:"

    # Use lsblk to get disk information
    lsblk -d -n -o NAME,SIZE,TYPE | grep disk | while read disk size type; do
        # Skip the boot disk
        if [[ ! "$disk" =~ ^"${boot_disk##*/}"[0-9]* ]] && [ "$disk" != "${boot_disk##*/}" ]; then
            echo "$disk $size"
        fi
    done
}

# Ask user if they want to mount a disk
print_info "Wil je een extra schijf mounten? (j/n)"
read -r mount_disk_choice

if [[ "$mount_disk_choice" =~ ^[Jj]$ ]]; then
    available_disks=$(list_available_disks | awk '{print $1}' | grep -v "^$")

    if [ -z "$available_disks" ]; then
        print_error "Geen beschikbare schijven gevonden (behalve boot schijf)"
    else
        # Show available disks with numbers
        print_info "Beschikbare schijven:"
        declare -a disks_array
        counter=1
        while IFS= read -r disk; do
            size=$(lsblk -d -n -o SIZE "/dev/$disk" 2>/dev/null || echo "Onbekend")
            echo "  $counter) /dev/$disk ($size)"
            disks_array+=("$disk")
            ((counter++))
        done <<< "$available_disks"

        # Ask user to select a disk
        print_info "Selecteer welke schijf je wilt mounten (1-$((counter-1))):"
        read -r disk_choice

        if ! [[ "$disk_choice" =~ ^[0-9]+$ ]] || [ "$disk_choice" -lt 1 ] || [ "$disk_choice" -ge "$counter" ]; then
            print_error "Ongeldige selectie!"
        else
            selected_disk="/dev/${disks_array[$((disk_choice-1))]}"
            print_info "Geselecteerde schijf: $selected_disk"

            # Check if disk has partitions
            partitions=$(lsblk -n -o NAME "$selected_disk" | grep -v "^${selected_disk##*/}$" | head -1)

            if [ -z "$partitions" ]; then
                print_error "Geen partities gevonden op $selected_disk"
                print_info "Maak eerst een partitie aan met: sudo parted $selected_disk"
            else
                device_to_mount="/dev/$partitions"
                print_info "Partitie gevonden: $device_to_mount"

                # Ask for mount point name
                print_info "Geef de naam voor het mount point (zonder /mnt/ prefix):"
                read -r mount_name

                # Validate mount name
                if [ -z "$mount_name" ]; then
                    print_error "Mount naam kan niet leeg zijn!"
                else
                    mount_point="/mnt/$mount_name"

                    # Check if mount point already exists
                    if [ -d "$mount_point" ]; then
                        print_info "Mount point $mount_point bestaat al"
                    else
                        print_info "Mount point $mount_point aanmaken..."
                        mkdir -p "$mount_point"
                        print_success "Mount point aangemaakt"
                    fi

                    # Check filesystem type
                    print_info "Controleer bestandssysteem..."
                    fstype=$(lsblk -n -o FSTYPE "$device_to_mount" | head -1)

                    if [ -z "$fstype" ]; then
                        print_error "Geen bestandssysteem gevonden op $device_to_mount"
                        print_info "Je moet eerst het apparaat formatteren:"
                        print_info "  sudo mkfs.ext4 $device_to_mount"
                    else
                        print_info "Bestandssysteem: $fstype"

                        # Get UUID for fstab
                        disk_uuid=$(blkid -s UUID -o value "$device_to_mount")

                        if [ -z "$disk_uuid" ]; then
                            print_error "Kan UUID niet bepalen voor $device_to_mount"
                        else
                            print_info "UUID: $disk_uuid"

                            # Get options based on filesystem type
                            mount_options="defaults,nofail"

                            # Mount the disk
                            print_info "Schijf mounten naar $mount_point..."
                            mount "$device_to_mount" "$mount_point" || print_error "Mounten mislukt!"

                            # Change ownership to alberto-adm
                            chown -R alberto-adm:alberto-adm "$mount_point"
                            print_success "Eigenaarschap ingesteld op alberto-adm"

                            # Add to fstab if not already present
                            if grep -q "$disk_uuid" /etc/fstab; then
                                print_info "Mount al aanwezig in /etc/fstab"
                            else
                                print_info "Mount toevoegen aan /etc/fstab..."
                                echo "UUID=$disk_uuid $mount_point $fstype $mount_options 0 2" >> /etc/fstab
                                print_success "Mount toegevoegd aan /etc/fstab"
                            fi

                            # Verify mount
                            if mountpoint -q "$mount_point"; then
                                print_success "Schijf succesvol gemount op $mount_point"
                                df -h "$mount_point"
                            else
                                print_error "Schijf kon niet worden geverifieerd op $mount_point"
                            fi
                        fi
                    fi
                fi
            fi
        fi
    fi
else
    print_info "Disk mounting overgeslagen"
fi

# ============================================
# Final verification
# ============================================
print_header "Installatie verifiëren"

print_info "Docker versie:"
docker --version

print_info "FFMPEG versie:"
ffmpeg -version | head -n 1

print_info "Rust versie:"
rustc --version

print_info "Cargo versie:"
cargo --version

print_success "Installatie voltooid!"

# ============================================
# Summary and next steps
# ============================================
print_header "Volgende stappen"
cat << EOF

1. Docker setup:
   - User moet uitloggen en inloggen zodat docker group wijzigingen van kracht worden
   - Test met: docker ps

2. Rust development:
   - Cargo is klaar voor projecten
   - Bouw het project met: cargo build

3. Storage:
   - Gemounte schijven zijn nu beschikbaar voor opslag
   - Controleer beschikbare ruimte met: df -h

4. Project setup:
   - Voer uit: docker-compose up
   - Zorg dat de opslag paden in config.yaml correct zijn ingesteld

5. Na een herstart:
   - Alle gemounte schijven moeten automatisch gemount zijn
   - Controleer met: mount | grep /mnt/

EOF

print_success "Setup script voltooid!"

