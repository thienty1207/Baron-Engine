#!/bin/sh
set -eu

action="install"
version="latest"
install_dir="${BARON_INSTALL_DIR:-$HOME/.local/bin}"
base_url="${BARON_RELEASE_BASE_URL:-https://github.com/thienty1207/Baron-Engine/releases/download}"
source_dir=""
state_root="${BARON_STATE_DIR:-$HOME/.baron}"
backup_dir="$state_root/backups"
metadata_path="$state_root/install.json"
binary_path="$install_dir/baron"

while [ "$#" -gt 0 ]; do
    case "$1" in
        --action) action="$2"; shift 2 ;;
        --version) version="$2"; shift 2 ;;
        --install-dir) install_dir="$2"; binary_path="$2/baron"; shift 2 ;;
        --base-url) base_url="$2"; shift 2 ;;
        --source-dir) source_dir="$2"; shift 2 ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

case "$action" in
    install|update|rollback|uninstall) ;;
    *) echo "Action must be install, update, rollback, or uninstall." >&2; exit 2 ;;
esac

if [ "$action" = "uninstall" ]; then
    rm -f "$binary_path" "$metadata_path"
    echo "Baron executable removed. Project files and Vault memory were not touched."
    exit 0
fi

mkdir -p "$install_dir" "$backup_dir"

if [ "$action" = "rollback" ]; then
    backup="$(ls -1t "$backup_dir"/baron-* 2>/dev/null | head -n 1 || true)"
    if [ -z "$backup" ]; then
        echo "No Baron rollback binary is available." >&2
        exit 1
    fi
    if [ -f "$binary_path" ]; then
        mv "$binary_path" "$backup_dir/baron-rollback-current-$(date +%s)"
    fi
    cp "$backup" "$binary_path"
    chmod +x "$binary_path"
    "$binary_path" --version
    echo "Baron rollback completed."
    exit 0
fi

case "$(uname -s)" in
    Linux) os_target="unknown-linux-gnu" ;;
    Darwin) os_target="apple-darwin" ;;
    *) echo "Unsupported operating system: $(uname -s)" >&2; exit 1 ;;
esac

case "$(uname -m)" in
    x86_64|amd64) architecture="x86_64" ;;
    arm64|aarch64)
        if [ "$os_target" != "apple-darwin" ]; then
            echo "Baron does not currently publish Linux ARM64 releases." >&2
            exit 1
        fi
        architecture="aarch64"
        ;;
    *) echo "Unsupported CPU architecture: $(uname -m)" >&2; exit 1 ;;
esac

target="$architecture-$os_target"

download() {
    url="$1"
    destination="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$destination"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$destination"
    else
        echo "curl or wget is required to download Baron." >&2
        exit 1
    fi
}

if [ "$version" = "latest" ]; then
    if [ -n "$source_dir" ]; then
        echo "Offline installation requires an explicit --version." >&2
        exit 1
    fi
    release_json="$(mktemp)"
    download "https://api.github.com/repos/thienty1207/Baron-Engine/releases/latest" "$release_json"
    version="$(sed -n 's/.*"tag_name":[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/p' "$release_json" | head -n 1)"
    rm -f "$release_json"
    if [ -z "$version" ]; then
        echo "Could not resolve the latest Baron version." >&2
        exit 1
    fi
fi
if ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Baron version must use numeric major.minor.patch form." >&2
    exit 1
fi

archive_name="baron-v$version-$target.tar.gz"
temporary_root="$(mktemp -d)"
archive_path="$temporary_root/$archive_name"
checksums_path="$temporary_root/SHA256SUMS"
extract_path="$temporary_root/extract"
mkdir -p "$extract_path"
trap 'rm -rf "$temporary_root"' EXIT HUP INT TERM

if [ -n "$source_dir" ]; then
    cp "$source_dir/$archive_name" "$archive_path"
    cp "$source_dir/SHA256SUMS" "$checksums_path"
else
    release_base="${base_url%/}/v$version"
    download "$release_base/$archive_name" "$archive_path"
    download "$release_base/SHA256SUMS" "$checksums_path"
fi

expected_checksum="$(awk -v name="$archive_name" '$2 == name { print tolower($1); exit }' "$checksums_path")"
if [ -z "$expected_checksum" ]; then
    echo "SHA256SUMS does not contain $archive_name." >&2
    exit 1
fi
if command -v sha256sum >/dev/null 2>&1; then
    actual_checksum="$(sha256sum "$archive_path" | awk '{print tolower($1)}')"
else
    actual_checksum="$(shasum -a 256 "$archive_path" | awk '{print tolower($1)}')"
fi
if [ "$actual_checksum" != "$expected_checksum" ]; then
    echo "Baron checksum verification failed for $archive_name." >&2
    exit 1
fi

tar -xzf "$archive_path" -C "$extract_path"
staged_binary="$extract_path/baron"
if [ ! -f "$staged_binary" ]; then
    echo "The Baron archive does not contain baron." >&2
    exit 1
fi
chmod +x "$staged_binary"
reported_version="$("$staged_binary" --version)"
case "$reported_version" in
    *"$version"*) ;;
    *) echo "Downloaded Baron binary reported an unexpected version: $reported_version" >&2; exit 1 ;;
esac

backup_path=""
if [ -f "$binary_path" ]; then
    backup_path="$backup_dir/baron-$(date +%s)-$version"
    mv "$binary_path" "$backup_path"
fi
if ! mv "$staged_binary" "$binary_path"; then
    if [ -n "$backup_path" ] && [ -f "$backup_path" ]; then
        mv "$backup_path" "$binary_path"
    fi
    exit 1
fi
chmod +x "$binary_path"

cat > "$metadata_path" <<EOF
{"version":"$version","binary":"$binary_path","checksum":"$actual_checksum"}
EOF

echo "Baron $version $action completed at $binary_path."
case ":${PATH:-}:" in
    *":$install_dir:"*) ;;
    *) echo "Add $install_dir to PATH before running baron." ;;
esac
