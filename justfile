# iZone: common tasks. Run `just` to list them.
#
#   just up          set up anything missing (password, controller address), then start the web UI in Docker
#   just clean       remove everything: container, image, data volume (changed password), .env, local web data, build output

set shell := ["bash", "-euo", "pipefail", "-c"]

auth_file := "src/web_auth.rs"

# List available tasks
default:
    @just --list --unsorted

# Set up anything missing, then build and start the web UI in Docker
up: _ensure-password _ensure-env
    docker compose up -d --build
    @just _wait

# Set (or change) the built-in default web password. Prompts without echo; stores only a salted hash.
password:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Choose the built-in default web password (at least 10 characters)."
    echo "Only a salted hash is saved, in {{auth_file}}. This repo is public, so use a long password"
    echo "you don't use anywhere else; you'll be asked to replace it the first time you sign in."
    echo
    echo "Building izone first (the first build can take a few minutes)…"
    cargo build -q
    echo "Ready. Type the password when asked (nothing is shown while you type)."
    hash=$(./target/debug/izone web hash-password | grep '^\$argon2id\$' || true)
    if [[ -z "$hash" ]]; then echo "No hash was produced; nothing changed." >&2; exit 1; fi
    sed -i.bak "s|^pub const DEFAULT_PASSWORD_HASH: &str = \".*\";|pub const DEFAULT_PASSWORD_HASH: \&str = \"$hash\";|" {{auth_file}}
    rm -f {{auth_file}}.bak
    grep -q "$hash" {{auth_file}} || { echo "Could not update {{auth_file}}" >&2; exit 1; }
    echo "✓ Default password hash saved in {{auth_file}}. Rebuild to use it: just up"

# Set the controller address and web port (saved in .env, which docker compose reads)
configure:
    #!/usr/bin/env bash
    set -euo pipefail
    cur_ip=$(grep -s '^IZONE_IP=' .env | cut -d= -f2- || true)
    cur_port=$(grep -s '^IZONE_WEB_PORT=' .env | cut -d= -f2- || true)
    read -rp "iZone controller address [${cur_ip:-http://192.168.1.130}]: " ip
    ip=${ip:-${cur_ip:-http://192.168.1.130}}
    [[ "$ip" =~ ^https?:// ]] || ip="http://$ip"
    while true; do
      read -rp "Web UI port [${cur_port:-8080}]: " port
      port=${port:-${cur_port:-8080}}
      [[ "$port" =~ ^[0-9]+$ ]] && (( port >= 1 && port <= 65535 )) && break
      echo "Enter a number from 1 to 65535."
    done
    printf 'IZONE_IP=%s\nIZONE_WEB_PORT=%s\n' "$ip" "$port" > .env
    echo "✓ Saved to .env (controller $ip, port $port)"

# Stop the web UI (keeps your changed password)
down:
    docker compose down

# Restart the web UI
restart:
    docker compose restart
    @just _wait

# Follow the web UI logs
logs:
    docker compose logs -f

# Show whether the web UI is running
status:
    @docker compose ps
    @just _url

# Reset the web password to the built-in default (signs everyone out)
reset-password:
    docker compose exec izone-web izone web reset-password -y

# Build the CLI (release)
build:
    cargo build --release

# Run the tests
test:
    cargo test

# Remove EVERYTHING: container, image, data volume (changed password + key), .env, local web data, build output
clean:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "This removes:"
    echo "  - the izone-web container, its image and the data volume (your changed web password and its key)"
    echo "  - .env (controller address and port)"
    echo "  - anything 'izone web' stored on this computer outside Docker (service, password, key)"
    echo "  - the build output (target/)"
    echo "It keeps the source code."
    read -rp "Continue? [y/N] " ok
    [[ "$ok" =~ ^[Yy]([Ee][Ss])?$ ]] || { echo "Cancelled. Nothing was removed."; exit 0; }
    reset_default=no
    if ! grep -q '^pub const DEFAULT_PASSWORD_HASH: &str = "";' {{auth_file}}; then
      read -rp "Also remove the default web password from {{auth_file}}, so the next 'just up' asks for a new one? [y/N] " rd
      [[ "$rd" =~ ^[Yy]([Ee][Ss])?$ ]] && reset_default=yes
    fi
    docker compose down -v --rmi local --remove-orphans || true
    rm -f .env && echo "✓ removed .env"
    if [[ -x target/release/izone ]]; then bin=target/release/izone
    elif [[ -x target/debug/izone ]]; then bin=target/debug/izone
    else bin=""; fi
    if [[ -n "$bin" ]]; then "$bin" web uninstall -y || true; fi
    cargo clean && echo "✓ removed target/"
    if [[ "$reset_default" == yes ]]; then
      sed -i.bak 's|^pub const DEFAULT_PASSWORD_HASH: &str = ".*";|pub const DEFAULT_PASSWORD_HASH: \&str = "";|' {{auth_file}}
      rm -f {{auth_file}}.bak
      echo "✓ removed the default web password from {{auth_file}} (the next 'just up' will ask for one)"
    fi
    echo "✓ Clean."

# Same as clean
uninstall: clean

# --- helpers ---------------------------------------------------------------

_ensure-password:
    #!/usr/bin/env bash
    set -euo pipefail
    if grep -q '^pub const DEFAULT_PASSWORD_HASH: &str = "";' {{auth_file}}; then
      echo "No default web password is set yet."
      just password
    fi

_ensure-env:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! grep -qs '^IZONE_IP=' .env; then just configure; fi

_url:
    #!/usr/bin/env bash
    port=$(grep -s '^IZONE_WEB_PORT=' .env | cut -d= -f2- || true)
    echo "Web UI: http://localhost:${port:-8080}"

_wait:
    #!/usr/bin/env bash
    set -uo pipefail
    port=$(grep -s '^IZONE_WEB_PORT=' .env | cut -d= -f2- || true); port=${port:-8080}
    for _ in $(seq 1 30); do
      if curl -fs -o /dev/null "http://localhost:$port/api/auth/status"; then
        echo "✓ iZone web UI is running: http://localhost:$port"
        exit 0
      fi
      sleep 1
    done
    echo "The web UI didn't come up. Recent logs:" >&2
    docker compose logs --tail 20 >&2
    exit 1
