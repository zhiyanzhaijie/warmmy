#!/bin/bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TEMPLATE_DIR="$REPO_ROOT/tools/provision-renew/template"
PROFILES_DIR="$HOME/Library/Developer/Xcode/UserData/Provisioning Profiles"
TEAM_ID="${TEAM_ID:-}"
APP_BUNDLE_ID="${APP_BUNDLE_ID:-com.zhiyanzhaijie.warmmy}"
EXTRA_BUNDLE_IDS="${EXTRA_BUNDLE_IDS:-}"

if [ ! -d "$TEMPLATE_DIR" ]; then
    echo "[renew] ERROR: template missing at $TEMPLATE_DIR"
    exit 1
fi

renew_one() {
    local bundle="$1"
    local label="$2"
    local work_dir="/tmp/warmmy-renew-$(echo "$bundle" | tr '.' '-')"

    echo ""
    echo "[renew] === $label : $bundle ==="

    rm -rf "$work_dir"
    cp -R "$TEMPLATE_DIR" "$work_dir"

    local pbxproj="$work_dir/RenewApp.xcodeproj/project.pbxproj"
    sed -i '' "s|__BUNDLE_ID__|$bundle|g" "$pbxproj"
    if [ -n "$TEAM_ID" ]; then
        sed -i '' "s|__TEAM_ID__|$TEAM_ID|g" "$pbxproj"
    else
        sed -i '' '/DEVELOPMENT_TEAM = __TEAM_ID__;/d' "$pbxproj"
    fi

    local team_args=()
    if [ -n "$TEAM_ID" ]; then
        team_args=(DEVELOPMENT_TEAM="$TEAM_ID")
    fi

    cd "$work_dir"
    xcodebuild build \
        -project RenewApp.xcodeproj \
        -target RenewApp \
        -configuration Debug \
        -destination 'generic/platform=iOS' \
        -allowProvisioningUpdates \
        -allowProvisioningDeviceRegistration \
        CODE_SIGN_STYLE=Automatic \
        "${team_args[@]}" \
        PRODUCT_BUNDLE_IDENTIFIER="$bundle" \
        2>&1 | tail -20
    cd - > /dev/null

    rm -rf "$work_dir"
    echo "[renew] $label done"
}

mkdir -p "$PROFILES_DIR"

renew_one "$APP_BUNDLE_ID" "app"

if [ -n "$EXTRA_BUNDLE_IDS" ]; then
    index=1
    for bundle in $EXTRA_BUNDLE_IDS; do
        renew_one "$bundle" "extra$index"
        index=$((index + 1))
    done
fi

echo ""
echo "[renew] Verifying:"
TEAM_ID="$TEAM_ID" APP_BUNDLE_ID="$APP_BUNDLE_ID" EXTRA_BUNDLE_IDS="$EXTRA_BUNDLE_IDS" bash "$(dirname "$0")/check-profiles.sh"
