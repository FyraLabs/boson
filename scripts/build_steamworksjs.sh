#!/bin/bash -x

DEST_DIR="${DEST_DIR:-$PWD/build}"
echo "DEST_DIR: ${DEST_DIR}"

# This script builds steamworks.js for the current platform.

# It assumes that you have NPM and the Rust toolchain installed.

GIT_REPO="https://github.com/ceifa/steamworks.js.git"

TMP_DIR="$(mktemp -d)/steamworks.js"
DIST_DIR="${TMP_DIR}/dist"

function clone_repo {
  git clone "${GIT_REPO}" "${TMP_DIR}"
}

function build {
  pushd "${TMP_DIR}"
  npm install
  npm run build
}

function copy_dist {
  mkdir -p "${DIST_DIR}"
  cp -r "${TMP_DIR}/dist/linux64/"*.node "${DEST_DIR}/lib/"
}

clone_repo

build

copy_dist