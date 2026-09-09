#!/usr/bin/env python3
"""Generate Baron Engine's canonical unsigned release metadata.

The signer job runs this standard-library-only script before the protected
signing step. It never receives the release private key. The resulting bytes
match the field order and compact JSON representation emitted by
``baron-core::release``; the detached Ed25519 signature is added separately by
the OpenSSL signing step.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path


RELEASE_IDENTITY = "github:thienty1207/Baron-Engine"
TARGETS = (
    ("x86_64-pc-windows-msvc", "zip", "baron.exe"),
    ("x86_64-unknown-linux-gnu", "tar.gz", "baron"),
)


def artifact_name(version: str, target: str, extension: str) -> str:
    return f"baron-v{version}-{target}.{extension}"


def candidate_name(version: str, target: str, binary: str) -> str:
    suffix = ".exe" if binary.endswith(".exe") else ""
    return f"baron-v{version}-{target}{suffix}"


def digest(path: Path) -> tuple[str, int]:
    hasher = hashlib.sha256()
    size = 0
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
            size += len(chunk)
    return hasher.hexdigest(), size


def write_metadata(artifacts_dir: Path, version: str, source_revision: str) -> None:
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise SystemExit("release version must use numeric major.minor.patch form")
    if not re.fullmatch(r"[0-9a-fA-F]{40}", source_revision):
        raise SystemExit("source revision must be an exact 40-character SHA")

    artifacts: list[dict[str, object]] = []
    candidates: list[dict[str, object]] = []
    for target, extension, binary in TARGETS:
        archive = artifacts_dir / artifact_name(version, target, extension)
        candidate = artifacts_dir / candidate_name(version, target, binary)
        if not archive.is_file() or not candidate.is_file():
            raise SystemExit(f"missing release artifact for {target}")
        archive_sha, archive_size = digest(archive)
        candidate_sha, candidate_size = digest(candidate)
        artifacts.append(
            {
                "name": archive.name,
                "target": target,
                "binary": binary,
                "sha256": archive_sha,
                "size_bytes": archive_size,
            }
        )
        candidates.append(
            {
                "name": candidate.name,
                "target": target,
                "binary": binary,
                "sha256": candidate_sha,
                "size_bytes": candidate_size,
            }
        )

    manifest = {
        "schema_version": 2,
        "product": "Baron Engine",
        "version": version,
        "source_revision": source_revision.lower(),
        "release_identity": RELEASE_IDENTITY,
        "minimum_compatible_version": version,
        "artifacts": artifacts,
        "update_candidates": candidates,
    }
    # Compact insertion-ordered JSON is the canonical signed representation.
    manifest_bytes = json.dumps(
        manifest, ensure_ascii=False, separators=(",", ":")
    ).encode("utf-8")
    (artifacts_dir / "release-manifest.json").write_bytes(manifest_bytes)
    checksum_lines = [
        f"{item['sha256']}  {item['name']}"
        for item in [*artifacts, *candidates]
    ]
    (artifacts_dir / "SHA256SUMS").write_text(
        "\n".join(checksum_lines) + "\n", encoding="utf-8", newline=""
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("artifacts_dir", type=Path)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-revision", required=True)
    args = parser.parse_args()
    write_metadata(args.artifacts_dir, args.version, args.source_revision)


if __name__ == "__main__":
    main()
