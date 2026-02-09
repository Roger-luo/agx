#!/usr/bin/env sh
set -eu

BIN_NAME="${AGX_BIN_NAME:-agx}"
REPO="${AGX_REPO:-Roger-luo/agx}"
INSTALL_DIR="${AGX_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${AGX_VERSION:-latest}"

tmp_dir=""

cleanup() {
  if [ -n "${tmp_dir}" ] && [ -d "${tmp_dir}" ]; then
    rm -rf "${tmp_dir}"
  fi
}

fail() {
  printf "error: %s\n" "$1" >&2
  exit 1
}

download_file() {
  url="$1"
  out="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$out"
    return
  fi
  if command -v wget >/dev/null 2>&1; then
    wget -qO "$out" "$url"
    return
  fi
  fail "curl or wget is required"
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64)
          printf "x86_64-unknown-linux-gnu"
          return
          ;;
        *)
          fail "unsupported Linux architecture: ${arch}"
          ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64|amd64)
          printf "x86_64-apple-darwin"
          return
          ;;
        arm64|aarch64)
          printf "aarch64-apple-darwin"
          return
          ;;
        *)
          fail "unsupported macOS architecture: ${arch}"
          ;;
      esac
      ;;
    *)
      fail "unsupported operating system: ${os}"
      ;;
  esac
}

verify_checksum() {
  archive_path="$1"
  checksum_path="$2"
  expected="$(awk '{print $1}' "$checksum_path")"
  [ -n "$expected" ] || fail "checksum file is empty"

  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$archive_path" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$archive_path" | awk '{print $1}')"
  elif command -v openssl >/dev/null 2>&1; then
    actual="$(openssl dgst -sha256 "$archive_path" | awk '{print $2}')"
  else
    fail "sha256sum, shasum, or openssl is required for checksum verification"
  fi

  if [ "$expected" != "$actual" ]; then
    fail "checksum verification failed"
  fi
}

trap cleanup EXIT INT TERM

target="$(detect_target)"
asset_base="${BIN_NAME}-${target}"
archive_name="${asset_base}.tar.gz"
checksum_name="${archive_name}.sha256"

if [ "$VERSION" = "latest" ]; then
  release_base_url="https://github.com/${REPO}/releases/latest/download"
else
  release_base_url="https://github.com/${REPO}/releases/download/${VERSION}"
fi

tmp_dir="$(mktemp -d)"
archive_path="${tmp_dir}/${archive_name}"
checksum_path="${tmp_dir}/${checksum_name}"

download_file "${release_base_url}/${archive_name}" "${archive_path}"
download_file "${release_base_url}/${checksum_name}" "${checksum_path}"
verify_checksum "${archive_path}" "${checksum_path}"

tar -xzf "${archive_path}" -C "${tmp_dir}"
[ -f "${tmp_dir}/${BIN_NAME}" ] || fail "release archive does not contain ${BIN_NAME}"

mkdir -p "${INSTALL_DIR}"
if command -v install >/dev/null 2>&1; then
  install -m 0755 "${tmp_dir}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
else
  cp "${tmp_dir}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
  chmod 0755 "${INSTALL_DIR}/${BIN_NAME}"
fi

printf "installed %s to %s/%s\n" "${BIN_NAME}" "${INSTALL_DIR}" "${BIN_NAME}"
case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    printf "add %s to your PATH to run %s globally\n" "${INSTALL_DIR}" "${BIN_NAME}" >&2
    ;;
esac
