#!/usr/bin/env bash
set -euo pipefail

repo="${MAEH_REPO:-ahnsv/maeh}"
version="${MAEH_VERSION:-latest}"
install_dir="${MAEH_INSTALL_DIR:-${HOME}/.local/bin}"

usage() {
  cat <<'EOF'
Install maeh from GitHub Releases.

Usage:
  curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash
  curl -fsSL https://raw.githubusercontent.com/ahnsv/maeh/main/install.sh | bash -s -- --dir /usr/local/bin

Options:
  --repo OWNER/REPO       GitHub repo to install from (default: ahnsv/maeh)
  --version VERSION       Release tag to install (default: latest)
  --dir PATH              Install directory (default: ~/.local/bin)
  -h, --help              Show this help

Environment:
  MAEH_REPO               Same as --repo
  MAEH_VERSION            Same as --version
  MAEH_INSTALL_DIR        Same as --dir
  GH_TOKEN/GITHUB_TOKEN   Optional token for private forks or higher rate limits
EOF
}

log() {
  printf 'maeh installer: %s\n' "$*"
}

die() {
  printf 'maeh installer: %s\n' "$*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      [ "$#" -ge 2 ] || die "--repo needs OWNER/REPO"
      repo="$2"
      shift 2
      ;;
    --version)
      [ "$#" -ge 2 ] || die "--version needs a tag"
      version="$2"
      shift 2
      ;;
    --dir|--install-dir)
      [ "$#" -ge 2 ] || die "$1 needs a path"
      install_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

need_cmd curl
need_cmd uname
need_cmd awk
need_cmd mktemp
need_cmd mkdir
need_cmd cp
need_cmd chmod

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64|Linux-amd64)
    asset="maeh-linux-x86_64"
    ;;
  Darwin-arm64|Darwin-aarch64)
    asset="maeh-macos-arm64"
    ;;
  *)
    die "unsupported platform: $(uname -s) $(uname -m)"
    ;;
esac

checksum_asset="${asset}.sha256"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    die "missing required command: sha256sum or shasum"
  fi
}

download_with_gh() {
  command -v gh >/dev/null 2>&1 || return 1

  local tag="$version"
  if [ "$tag" = "latest" ]; then
    tag="$(gh release view --repo "$repo" --json tagName --jq .tagName 2>/dev/null)" || return 1
  fi

  gh release download "$tag" \
    --repo "$repo" \
    --pattern "$asset" \
    --pattern "$checksum_asset" \
    --dir "$tmp_dir" \
    --clobber >/dev/null 2>&1
}

download_url() {
  local name="$1"
  if [ "$version" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download/%s' "$repo" "$name"
  else
    printf 'https://github.com/%s/releases/download/%s/%s' "$repo" "$version" "$name"
  fi
}

curl_download() {
  local url="$1" out="$2" token="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
  if [ -n "$token" ]; then
    curl -fsSL \
      -H "Authorization: Bearer ${token}" \
      -H "Accept: application/octet-stream" \
      "$url" \
      -o "$out"
  else
    curl -fsSL "$url" -o "$out"
  fi
}

download_with_curl() {
  local asset_url checksum_url
  asset_url="$(download_url "$asset")"
  checksum_url="$(download_url "$checksum_asset")"
  curl_download "$asset_url" "$tmp_dir/$asset" || die "failed to download $asset_url; publish a v* release with asset $asset"
  curl_download "$checksum_url" "$tmp_dir/$checksum_asset" || die "failed to download $checksum_url; publish a v* release with asset $checksum_asset"
}

log "installing ${repo} ${version} for ${asset}"
if download_with_gh; then
  log "downloaded release assets with gh"
else
  log "downloading release assets with curl"
  download_with_curl
fi

[ -s "$tmp_dir/$asset" ] || die "downloaded binary is empty"
[ -s "$tmp_dir/$checksum_asset" ] || die "downloaded checksum is empty"

expected="$(awk '{print $1; exit}' "$tmp_dir/$checksum_asset")"
actual="$(sha256_file "$tmp_dir/$asset")"
[ "$expected" = "$actual" ] || die "checksum mismatch for $asset"

mkdir -p "$install_dir"
cp "$tmp_dir/$asset" "$install_dir/maeh"
chmod 755 "$install_dir/maeh"

"$install_dir/maeh" --help >/dev/null

log "installed $install_dir/maeh"
case ":$PATH:" in
  *":$install_dir:"*) ;;
  *) log "add $install_dir to PATH to run maeh from any shell" ;;
esac
