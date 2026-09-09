#!/bin/sh
set -eu

action="install"
version="latest"
install_dir="${BARON_INSTALL_DIR:-$HOME/.local/bin}"
base_url="${BARON_RELEASE_BASE_URL:-https://github.com/thienty1207/Baron-Engine/releases/download}"
latest_manifest_url="${BARON_RELEASE_LATEST_MANIFEST_URL:-https://github.com/thienty1207/Baron-Engine/releases/latest/download/release-manifest.json}"
source_dir=""
state_root="${BARON_STATE_DIR:-$HOME/.baron}"
backup_dir="$state_root/backups"
metadata_path="$state_root/install.json"
binary_path="$install_dir/baron"

# This key is pinned in the trusted installer source. Downloaded keys are never
# accepted. The PEM is the Ed25519 SubjectPublicKeyInfo for the same raw key.
trusted_release_key_id="baron-release-2026"
trusted_release_public_key_base64="SBgrmtK5emhgD5e6UW664fXR3qogCjVPHtrxeVk2TcA="

while [ "$#" -gt 0 ]; do
    case "$1" in
        --action) action="$2"; shift 2 ;;
        --version) version="$2"; shift 2 ;;
        --install-dir) install_dir="$2"; binary_path="$2/baron"; shift 2 ;;
        --base-url) base_url="$2"; shift 2 ;;
        --latest-manifest-url) latest_manifest_url="$2"; shift 2 ;;
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

if [ "$action" = "rollback" ]; then
    mkdir -p "$install_dir" "$backup_dir"
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
if [ "$version" != "latest" ] && ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Baron version must use numeric major.minor.patch form." >&2
    exit 1
fi

download() {
    url="$1"
    destination="$2"
    case "$url" in
        https://*) ;;
        *) echo "Refusing insecure non-HTTPS Baron release URL: $url" >&2; exit 1 ;;
    esac
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --proto '=https' --tlsv1.2 "$url" -o "$destination"
    elif command -v wget >/dev/null 2>&1; then
        wget -q --https-only "$url" -O "$destination"
    else
        echo "curl or wget is required to download Baron." >&2
        exit 1
    fi
}

find_python() {
    if command -v python3 >/dev/null 2>&1; then
        command -v python3
    elif command -v python >/dev/null 2>&1; then
        if python -c 'import sys; raise SystemExit(0 if sys.version_info[0] == 3 else 1)' >/dev/null 2>&1; then
            command -v python
        else
            echo "Python 3 is required to parse authenticated Baron metadata; refusing installation." >&2
            exit 1
        fi
    else
        echo "Python 3 is required to parse authenticated Baron metadata; refusing installation." >&2
        exit 1
    fi
}

find_openssl() {
    if command -v openssl >/dev/null 2>&1; then
        command -v openssl
    else
        echo "OpenSSL with Ed25519 support is required to verify Baron metadata; refusing installation." >&2
        exit 1
    fi
}

temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/baron-install.XXXXXX")"
manifest_path="$temporary_root/release-manifest.json"
signature_path="$temporary_root/release-manifest.sig"
signature_bin="$temporary_root/release-manifest.signature"
message_path="$temporary_root/release-manifest.message"
checksums_path="$temporary_root/SHA256SUMS"
extract_path="$temporary_root/extract"
mkdir -p "$extract_path"
trap 'rm -rf "$temporary_root"' EXIT HUP INT TERM

if [ "$version" = "latest" ]; then
    if [ -n "$source_dir" ]; then
        echo "Offline installation requires an explicit --version." >&2
        exit 1
    fi
    download "$latest_manifest_url" "$manifest_path"
    latest_signature_url="${latest_manifest_url%.json}.sig"
    download "$latest_signature_url" "$signature_path"
elif [ -n "$source_dir" ]; then
    cp "$source_dir/release-manifest.json" "$manifest_path"
    cp "$source_dir/release-manifest.sig" "$signature_path"
else
    release_base="${base_url%/}/v$version"
    download "$release_base/release-manifest.json" "$manifest_path"
    download "$release_base/release-manifest.sig" "$signature_path"
fi

python_bin="$(find_python)"
openssl_bin="$(find_openssl)"
public_key_path="$temporary_root/release-public-key.pem"
"$python_bin" - "$trusted_release_public_key_base64" "$public_key_path" <<'PY'
import base64
import sys

encoded, path = sys.argv[1:]
raw = base64.b64decode(encoded, validate=True)
if len(raw) != 32:
    raise ValueError("pinned release public key must decode to 32 bytes")
der = bytes.fromhex("302a300506032b6570032100") + raw
pem = base64.b64encode(der).decode("ascii")
open(path, "w", encoding="ascii").write("-----BEGIN PUBLIC KEY-----\n" + pem + "\n-----END PUBLIC KEY-----\n")
PY

signature_key_id="$("$python_bin" - "$signature_path" "$signature_bin" <<'PY'
import json
import re
import sys

path, signature_path = sys.argv[1:]
raw = open(path, "rb").read()

def pairs(items):
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError("duplicate signature field")
        result[key] = value
    return result

value = json.loads(raw.decode("utf-8"), object_pairs_hook=pairs)
if set(value) != {"schema_version", "key_id", "signature"}:
    raise ValueError("signature metadata fields are invalid")
if value["schema_version"] != 1:
    raise ValueError("unsupported release signature schema")
if not isinstance(value["key_id"], str) or not re.fullmatch(r"[A-Za-z0-9._-]{1,120}", value["key_id"]):
    raise ValueError("release signing key ID is invalid")
if not isinstance(value["signature"], str) or not re.fullmatch(r"[0-9a-fA-F]{128}", value["signature"]):
    raise ValueError("release signature must be 64-byte hexadecimal Ed25519 data")
open(signature_path, "wb").write(bytes.fromhex(value["signature"]))
print(value["key_id"])
PY
)" || {
    echo "Release signature metadata is malformed; refusing installation." >&2
    exit 1
}
if [ "$signature_key_id" != "$trusted_release_key_id" ]; then
    echo "Release signature key ID is not Baron's trusted production key; refusing installation." >&2
    exit 1
fi

printf 'baron-release-manifest-v1\000' > "$message_path"
cat "$manifest_path" >> "$message_path"
if ! "$openssl_bin" pkeyutl -verify -pubin -inkey "$public_key_path" \
    -rawin -in "$message_path" -sigfile "$signature_bin" >/dev/null 2>&1; then
    echo "Baron release metadata signature verification failed; no files were changed." >&2
    exit 1
fi

manifest_fields="$("$python_bin" - "$manifest_path" "$target" "$version" <<'PY'
import json
import re
import sys

path, target, requested_version = sys.argv[1:]

def pairs(items):
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError("duplicate manifest field")
        result[key] = value
    return result

data = json.loads(open(path, "rb").read().decode("utf-8"), object_pairs_hook=pairs)
required = {"schema_version", "product", "version", "source_revision", "release_identity", "minimum_compatible_version", "artifacts", "update_candidates"}
if set(data) != required:
    raise ValueError("release manifest fields are invalid")
if data["schema_version"] not in (1, 2) or data["product"] != "Baron Engine":
    raise ValueError("release manifest identity is invalid")
if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", data["version"]):
    raise ValueError("release manifest version is invalid")
if requested_version != "latest" and data["version"] != requested_version:
    raise ValueError("release manifest version does not match the requested version")
if data["release_identity"] != "github:thienty1207/Baron-Engine":
    raise ValueError("release manifest source identity is invalid")
if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", data["minimum_compatible_version"]):
    raise ValueError("release manifest compatibility version is invalid")
requested_archive = f"baron-v{data['version']}-{target}.tar.gz"
artifact = None
for candidate in data["artifacts"]:
    if not isinstance(candidate, dict):
        raise ValueError("release artifact entry is invalid")
    if set(candidate) != {"name", "target", "binary", "sha256", "size_bytes"}:
        raise ValueError("release artifact fields are invalid")
    if candidate["target"] == target:
        if artifact is not None:
            raise ValueError("release manifest contains duplicate target artifacts")
        artifact = candidate
if artifact is None or artifact["name"] != requested_archive or artifact["binary"] != "baron":
    raise ValueError("release manifest does not contain the expected platform artifact")
if not isinstance(artifact["sha256"], str) or not re.fullmatch(r"[0-9a-fA-F]{64}", artifact["sha256"]):
    raise ValueError("release artifact digest is invalid")
if isinstance(artifact["size_bytes"], bool) or not isinstance(artifact["size_bytes"], int) or artifact["size_bytes"] < 0:
    raise ValueError("release artifact size is invalid")
print(f"{data['version']}|{artifact['sha256'].lower()}|{artifact['size_bytes']}")
PY
)" || {
    echo "Authenticated Baron release metadata does not match this platform; refusing installation." >&2
    exit 1
}
IFS='|' read -r manifest_version expected_checksum expected_size <<EOF
$manifest_fields
EOF
version="$manifest_version"
archive_name="baron-v$version-$target.tar.gz"

temporary_archive="$temporary_root/$archive_name"
if [ -n "$source_dir" ]; then
    cp "$source_dir/$archive_name" "$temporary_archive"
    cp "$source_dir/SHA256SUMS" "$checksums_path"
else
    release_base="${base_url%/}/v$version"
    download "$release_base/$archive_name" "$temporary_archive"
    download "$release_base/SHA256SUMS" "$checksums_path"
fi

checksum_from_file="$(awk -v name="$archive_name" '$2 == name { print tolower($1); exit }' "$checksums_path")"
if [ "$checksum_from_file" != "$expected_checksum" ]; then
    echo "Unsigned SHA256SUMS does not match authenticated Baron metadata." >&2
    exit 1
fi
if command -v sha256sum >/dev/null 2>&1; then
    actual_checksum="$(sha256sum "$temporary_archive" | awk '{print tolower($1)}')"
elif command -v shasum >/dev/null 2>&1; then
    actual_checksum="$(shasum -a 256 "$temporary_archive" | awk '{print tolower($1)}')"
else
    echo "sha256sum or shasum is required to verify the Baron artifact; refusing installation." >&2
    exit 1
fi
actual_size="$(wc -c < "$temporary_archive" | tr -d '[:space:]')"
if [ "$actual_size" != "$expected_size" ]; then
    echo "Baron artifact size verification failed for $archive_name." >&2
    exit 1
fi
if [ "$actual_checksum" != "$expected_checksum" ]; then
    echo "Baron artifact checksum verification failed for $archive_name." >&2
    exit 1
fi

tar -xzf "$temporary_archive" -C "$extract_path"
staged_binary="$extract_path/baron"
if [ ! -f "$staged_binary" ]; then
    echo "The Baron archive does not contain baron." >&2
    exit 1
fi
chmod +x "$staged_binary"
reported_version="$("$staged_binary" --version)"
if [ "$reported_version" != "baron $version" ]; then
    echo "Downloaded Baron binary reported an unexpected version: $reported_version" >&2
    exit 1
fi

# Authentication, platform selection, size, and digest verification have all
# completed before this first mutation of the existing installation.
mkdir -p "$install_dir" "$backup_dir"
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

mkdir -p "$state_root"
cat > "$metadata_path" <<EOF
{\"version\":\"$version\",\"binary\":\"$binary_path\",\"checksum\":\"$actual_checksum\"}
EOF

echo \"Baron $version $action completed at $binary_path.\"
case \":${PATH:-}:\" in
    *\":$install_dir:\"*) ;;
    *) echo \"Add $install_dir to PATH before running baron.\" ;;
esac
