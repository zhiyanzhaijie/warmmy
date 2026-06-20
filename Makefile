UI_DIR := crates/ui

# Keep third-party tracing quiet by default. Raise our crates to info.
WARMMY_LOG ?= warn,ui=info,api=info,app=info,infra=info,adapters=info
IOS_DEVICE ?= echo iPhone
IOS_TEAM_ID ?=
IOS_BUNDLE_ID ?= com.zhiyanzhaijie.warmmy
IOS_TEAM_ARG := $(if $(strip $(IOS_TEAM_ID)),--apple-team-id "$(IOS_TEAM_ID)",)
WEB_BASE_PATH ?= /warmmy/
WEB_DIST ?= dist-pages
VERSION := $(shell grep -m 1 '^version' Cargo.toml | cut -d '"' -f 2)

.PHONY: ios ios-device ios-debug android android-debug desktop desktop-debug web web-debug web-release ios-release android-release desktop-mac-release desktop-window-release desktop-win-release check-profiles renew-profiles ensure-profiles

ios:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG="$(WARMMY_LOG)" dx serve -p warmmy-mobile --ios

ios-device: ensure-profiles
	cd $(UI_DIR) && APP_ENV=development IPHONEOS_DEPLOYMENT_TARGET=16.0 RUST_LOG="$(WARMMY_LOG)" dx build -p warmmy-mobile --ios --device "$(IOS_DEVICE)" $(IOS_TEAM_ARG)
	IOS_DEVICE_NAME="$(IOS_DEVICE)" APP_PATH="$$(find $(UI_DIR)/target/dx/warmmy-mobile/debug/ios -maxdepth 1 -name '*.app' -print -quit)" bash scripts/install-ios-app.sh

ios-debug:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG=debug dx serve -p warmmy-mobile --ios

android:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG="$(WARMMY_LOG)" dx serve -p warmmy-mobile --android

android-debug:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG=debug dx serve -p warmmy-mobile --android

desktop:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG="$(WARMMY_LOG)" dx serve -p warmmy --desktop

desktop-debug:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG=debug dx serve -p warmmy --desktop

web:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG="$(WARMMY_LOG)" dx serve -p web --web

web-debug:
	cd $(UI_DIR) && APP_ENV=development RUST_LOG=debug dx serve -p web --web

web-release:
	cd $(UI_DIR)/packages/web && APP_ENV=production dx bundle --web --release --base-path "$(WEB_BASE_PATH)" --out-dir "$(WEB_DIST)"

ios-release: ensure-profiles
	cd $(UI_DIR) && APP_ENV=production IPHONEOS_DEPLOYMENT_TARGET=16.0 dx bundle -p warmmy-mobile --ios --release --package-types ipa --codesign true $(IOS_TEAM_ARG)
	@mkdir -p dist
	@find $(UI_DIR)/target/dx/warmmy-mobile -name "*.ipa" -exec cp {} dist/warmmy-v$(VERSION)-ios.ipa \;
	@echo "iOS release build saved to: dist/warmmy-v$(VERSION)-ios.ipa"

check-profiles:
	@TEAM_ID="$(IOS_TEAM_ID)" APP_BUNDLE_ID="$(IOS_BUNDLE_ID)" bash scripts/check-profiles.sh

renew-profiles:
	@TEAM_ID="$(IOS_TEAM_ID)" APP_BUNDLE_ID="$(IOS_BUNDLE_ID)" bash scripts/renew-profiles.sh

ensure-profiles:
	@TEAM_ID="$(IOS_TEAM_ID)" APP_BUNDLE_ID="$(IOS_BUNDLE_ID)" bash scripts/check-profiles.sh > /dev/null 2>&1 || TEAM_ID="$(IOS_TEAM_ID)" APP_BUNDLE_ID="$(IOS_BUNDLE_ID)" bash scripts/renew-profiles.sh

android-release:
	cd $(UI_DIR) && APP_ENV=production dx bundle -p warmmy-mobile --android --release --package-types apk
	@ANDROID_MAIN_DIR="$(UI_DIR)/target/dx/warmmy-mobile/release/android/app/app/src/main" ICON_SOURCE="$(UI_DIR)/packages/ui/assets/app-icon.png" bash scripts/patch-android-icons.sh
	@cd $(UI_DIR)/target/dx/warmmy-mobile/release/android/app && ./gradlew :app:assembleDebug >/dev/null
	@mkdir -p dist
	@cp $(UI_DIR)/target/dx/warmmy-mobile/release/android/app/app/build/outputs/apk/debug/app-debug.apk dist/warmmy-v$(VERSION)-android.apk
	@echo "Android release build saved to: dist/warmmy-v$(VERSION)-android.apk"

desktop-mac-release:
	cd $(UI_DIR) && APP_ENV=production dx bundle -p warmmy --macos --release --package-types dmg
	@mkdir -p dist
	@find $(UI_DIR)/target/dx/warmmy -name "*.dmg" -exec cp {} dist/warmmy-v$(VERSION)-macos.dmg \;
	@echo "macOS release build saved to: dist/warmmy-v$(VERSION)-macos.dmg"

desktop-window-release:
	cd $(UI_DIR) && APP_ENV=production dx bundle -p warmmy --windows --release --package-types msi
	@mkdir -p dist
	@find $(UI_DIR)/target/dx/warmmy -name "*.msi" -exec cp {} dist/warmmy-v$(VERSION)-windows.msi \;
	@echo "Windows release build saved to: dist/warmmy-v$(VERSION)-windows.msi"

desktop-win-release: desktop-window-release
