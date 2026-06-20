#!/usr/bin/env bash
set -euo pipefail

ANDROID_MAIN_DIR="${ANDROID_MAIN_DIR:?ANDROID_MAIN_DIR is required}"
ICON_SOURCE="${ICON_SOURCE:?ICON_SOURCE is required}"

RES_DIR="${ANDROID_MAIN_DIR}/res"

if [[ ! -f "${ICON_SOURCE}" ]]; then
  echo "Android icon source not found: ${ICON_SOURCE}" >&2
  exit 1
fi

if [[ ! -d "${RES_DIR}" ]]; then
  echo "Android res directory not found: ${RES_DIR}" >&2
  exit 1
fi

command -v sips >/dev/null 2>&1 || {
  echo "sips is required to generate Android launcher icons" >&2
  exit 1
}

rm -f "${RES_DIR}/mipmap-anydpi-v26/ic_launcher.xml"
rm -f "${RES_DIR}/drawable/ic_launcher_background.xml"
rm -f "${RES_DIR}/drawable-v24/ic_launcher_foreground.xml"

for entry in \
  "mipmap-mdpi 48" \
  "mipmap-hdpi 72" \
  "mipmap-xhdpi 96" \
  "mipmap-xxhdpi 144" \
  "mipmap-xxxhdpi 192"
do
  dir="${entry% *}"
  size="${entry#* }"
  mkdir -p "${RES_DIR}/${dir}"
  rm -f "${RES_DIR}/${dir}/ic_launcher.webp"
  sips -s format png -z "${size}" "${size}" "${ICON_SOURCE}" --out "${RES_DIR}/${dir}/ic_launcher.png" >/dev/null
done

