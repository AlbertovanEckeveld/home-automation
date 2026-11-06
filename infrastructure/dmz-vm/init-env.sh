
#!/usr/bin/env bash

set -Eeuo pipefail

# ==========================================
# HOMELAB ENVIRONMENT INITIALIZER
# ==========================================

# Determine directory where this script is located
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR"

echo "=========================================="
echo " Environment initializer"
echo "=========================================="
echo "Working directory: $SCRIPT_DIR"
echo ""

# Check dependencies
for cmd in openssl find awk; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "ERROR: Missing dependency: $cmd"
        exit 1
    fi
done

# ==========================================
# SECRET GENERATION
# ==========================================

generate_secret() {
    openssl rand -hex 32
}

generate_authentik_secret() {
    openssl rand -hex 48
}

generate_laravel_key() {
    printf 'base64:%s' "$(openssl rand -base64 32 | tr -d '\n')"
}

# ==========================================
# READ VALUE FROM CENTRAL ENV
# ==========================================

get_env_value() {
    local key="$1"
    local file="$2"

    awk -F= -v key="$key" '
        $1 == key {
            sub(/^[^=]*=/, "")
            print
            exit
        }
    ' "$file"
}

# ==========================================
# BITWARDEN CREDENTIALS
# ==========================================

prompt_bitwarden_credential() {
    local label="$1"
    local value=""

    while [[ -z "$value" ]]; do
        read -r -s -p "$label: " value
        echo
    done

    printf '%s' "$value"
}

# ==========================================
# GENERATE VALUE
# ==========================================

generate_value() {
    local key="$1"
    local file="$2"
    local directory

    directory="$(dirname "$file")"

    case "$key" in
        AUTHENTIK_SECRET_KEY)
            generate_authentik_secret
            ;;

        APP_KEY)
            generate_laravel_key
            ;;

        BW_INSTALLATION_ID)
            prompt_bitwarden_credential \
                "Bitwarden Installation ID ($directory)"
            ;;

        BW_INSTALLATION_KEY)
            prompt_bitwarden_credential \
                "Bitwarden Installation Key ($directory)"
            ;;

        BW_DB_PASSWORD)
            local central_key=""

            case "$directory" in
                *turinguard*)
                    central_key="BW_TURINGUARD_DB_PASSWORD"
                    ;;
                *albertove*)
                    central_key="BW_ALBERTOVE_DB_PASSWORD"
                    ;;
                *)
                    echo "ERROR: Unknown Bitwarden instance: $directory" >&2
                    return 1
                    ;;
            esac

            get_env_value "$central_key" "$SCRIPT_DIR/.env"
            ;;

        *)
            generate_secret
            ;;
    esac
}

# ==========================================
# INITIALIZE ENVIRONMENT FILE
# ==========================================

initialize_env() {
    local example="$1"
    local target="${example%.example}"
    local temporary="${target}.tmp"

    if [[ -f "$target" ]]; then
        echo "[SKIP] $target already exists"
        return
    fi

    echo "[CREATE] $target"

    : > "$temporary"

    while IFS= read -r line || [[ -n "$line" ]]; do

        if [[ "$line" =~ ^([A-Za-z_][A-Za-z0-9_]*)=(.*)$ ]]; then
            local key="${BASH_REMATCH[1]}"
            local value="${BASH_REMATCH[2]}"

            if [[ "$value" == "CHANGE_ME" ]]; then
                value="$(generate_value "$key" "$example")"

                if [[ -z "$value" ]]; then
                    rm -f "$temporary"
                    return 1
                fi
            fi

            printf '%s=%s\n' "$key" "$value" >> "$temporary"
        else
            printf '%s\n' "$line" >> "$temporary"
        fi

    done < "$example"

    chmod 600 "$temporary"
    mv "$temporary" "$target"

    echo "[OK] $target created"
}

# ==========================================
# INITIALIZE CENTRAL ENV FIRST
# ==========================================

if [[ -f "$SCRIPT_DIR/.env.example" ]]; then
    initialize_env "$SCRIPT_DIR/.env.example"
else
    echo "No central .env.example found. Continuing..."
fi

# ==========================================
# FIND ALL OTHER ENV EXAMPLES
# ==========================================

while IFS= read -r -d '' example; do

    [[ "$example" == "$SCRIPT_DIR/.env.example" ]] && continue

    initialize_env "$example"

done < <(
    find "$SCRIPT_DIR" \
        -type d \( -name .git -o -name .venv -o -name node_modules \) -prune -o \
        -type f \( -name ".env.example" -o -name "*.env.example" \) \
        -print0
)

# ==========================================
# COMPLETED
# ==========================================

echo ""
echo "=========================================="
echo " Initialization completed!"
echo "=========================================="

echo ""
echo "Generated environment files:"

find "$SCRIPT_DIR" \
    -type d \( -name .git -o -name .venv -o -name node_modules \) -prune -o \
    -type f \( -name ".env" -o -name "*.env" \) \
    -print

echo ""
echo "IMPORTANT:"
echo "- Review generated environment files."
echo "- Verify Bitwarden installation credentials."
echo "- Never commit generated .env files."
echo "- Keep a secure backup of your secrets."
