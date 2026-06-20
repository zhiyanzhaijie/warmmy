#!/bin/bash
set -e

APP_PATH="${APP_PATH:-target/dx/mobile/debug/ios/Mobile.app}"

if [ ! -d "$APP_PATH" ]; then
  echo "ERROR: $APP_PATH not found. Run dx build first."
  exit 1
fi

echo ">> Extracting entitlements from provisioning profile..."
security cms -D -i "$APP_PATH/embedded.mobileprovision" 2>/dev/null \
  | xmllint --xpath '//key[text()="Entitlements"]/following-sibling::dict[1]' - 2>/dev/null \
  > /tmp/warmmy-ent-dict.plist
printf '<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n<plist version="1.0">\n' > /tmp/warmmy-ent.plist
cat /tmp/warmmy-ent-dict.plist >> /tmp/warmmy-ent.plist
printf '\n</plist>\n' >> /tmp/warmmy-ent.plist

echo ">> Re-signing app..."
IDENTITY="${IOS_SIGNING_IDENTITY:-$(security find-identity -v -p codesigning | grep "Apple Development" | head -1 | awk -F'"' '{print $2}')}"
if [ -z "$IDENTITY" ]; then
  echo "ERROR: No Apple Development signing identity found."
  exit 1
fi

if [ -d "$APP_PATH/Frameworks" ]; then
  find "$APP_PATH/Frameworks" -maxdepth 1 -type d -name "*.framework" -print0 \
    | while IFS= read -r -d '' framework; do
      codesign --force --sign "$IDENTITY" "$framework"
    done
fi
codesign --force --sign "$IDENTITY" --entitlements /tmp/warmmy-ent.plist "$APP_PATH"

echo ">> Installing on device..."
DEVICE_ID="$IOS_DEVICE_ID"
if [ -z "$DEVICE_ID" ]; then
  DEVICE_LIST="$(xcrun devicectl list devices 2>/dev/null)"
  if [ -n "$IOS_DEVICE_NAME" ]; then
    DEVICE_ID="$(printf '%s\n' "$DEVICE_LIST" | grep "^$IOS_DEVICE_NAME[[:space:]].*[[:space:]]connected[[:space:]]" | grep -oE '[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}' | head -1)"
  fi
  if [ -z "$DEVICE_ID" ]; then
    DEVICE_ID="$(printf '%s\n' "$DEVICE_LIST" | grep "[[:space:]]connected[[:space:]]" | grep -oE '[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}' | head -1)"
  fi
fi
if [ -z "$DEVICE_ID" ]; then
  echo "ERROR: No connected device found."
  exit 1
fi
xcrun devicectl device install app --device "$DEVICE_ID" "$APP_PATH" 2>&1

echo ">> Done."
