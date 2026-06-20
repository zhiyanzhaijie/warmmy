#!/bin/bash
set -euo pipefail

PROFILES_DIR="$HOME/Library/Developer/Xcode/UserData/Provisioning Profiles"
TEAM_ID="${TEAM_ID:-}"
APP_BUNDLE_ID="${APP_BUNDLE_ID:-com.zhiyanzhaijie.warmmy}"
EXTRA_BUNDLE_IDS="${EXTRA_BUNDLE_IDS:-}"
THRESHOLD_SEC="${RENEW_THRESHOLD_SEC:-86400}"

NOW_TS=$(date +%s)
CRITICAL=0

profile_expiry_for() {
    local target_bundle="$1"
    for f in "$PROFILES_DIR"/*.mobileprovision; do
        [ -f "$f" ] || continue
        local plist
        plist=$(security cms -D -i "$f" 2>/dev/null) || continue
        local app_id
        app_id=$(echo "$plist" | plutil -extract 'Entitlements.application-identifier' raw - 2>/dev/null || echo "")
        local app_bundle="${app_id#*.}"
        local app_team="${app_id%%.*}"
        if [ "$app_bundle" = "$target_bundle" ] && { [ -z "$TEAM_ID" ] || [ "$app_team" = "$TEAM_ID" ]; }; then
            local exp
            exp=$(echo "$plist" | plutil -extract ExpirationDate raw - 2>/dev/null)
            local uuid
            uuid=$(echo "$plist" | plutil -extract UUID raw - 2>/dev/null)
            echo "$uuid|$exp"
            return 0
        fi
    done
    echo "|"
}

report_one() {
    local label="$1"
    local bundle="$2"
    local info uuid exp
    info=$(profile_expiry_for "$bundle")
    uuid="${info%%|*}"
    exp="${info##*|}"
    if [ -z "$uuid" ]; then
        printf "  %-7s %-50s MISSING\n" "$label" "$bundle"
        CRITICAL=1
        return
    fi
    local exp_ts
    exp_ts=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$exp" +%s 2>/dev/null || echo 0)
    local remain=$((exp_ts - NOW_TS))
    local state
    if [ "$remain" -le 0 ]; then
        state="EXPIRED"
        CRITICAL=1
    elif [ "$remain" -lt "$THRESHOLD_SEC" ]; then
        state="EXPIRES_SOON"
        CRITICAL=1
    else
        local days=$((remain / 86400))
        state="OK (${days}d left)"
    fi
    printf "  %-7s %-50s %s | %s\n" "$label" "$bundle" "$exp" "$state"
}

echo "Provisioning profiles:"
if [ -z "$TEAM_ID" ]; then
    echo "  team    auto-detect from matching local profiles"
else
    echo "  team    $TEAM_ID"
fi
report_one "app" "$APP_BUNDLE_ID"

if [ -n "$EXTRA_BUNDLE_IDS" ]; then
    index=1
    for bundle in $EXTRA_BUNDLE_IDS; do
        report_one "extra$index" "$bundle"
        index=$((index + 1))
    done
fi

if [ "$CRITICAL" -eq 1 ]; then
    echo "[check-profiles] CRITICAL: at least one profile expired or expiring < $((THRESHOLD_SEC / 3600))h"
    exit 1
fi

echo "[check-profiles] OK"
exit 0
