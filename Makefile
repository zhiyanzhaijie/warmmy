UI_DIR := crates/ui

# Keep third-party tracing quiet by default. Raise our crates to info.
WARMMY_LOG ?= warn,ui=info,api=info,app=info,infra=info,adapters=info
IOS_DEVICE ?= echo iPhone
IOS_TEAM_ID ?= 2PU94V3G4N
WEB_BASE_PATH ?= /warmmy/
WEB_DIST ?= dist-pages

.PHONY: ios ios-device ios-debug android android-debug desktop desktop-debug web web-debug web-release

ios:
	cd $(UI_DIR) && RUST_LOG="$(WARMMY_LOG)" dx serve -p mobile --ios

ios-device:
	cd $(UI_DIR) && IPHONEOS_DEPLOYMENT_TARGET=16.0 RUST_LOG="$(WARMMY_LOG)" dx serve -p mobile --ios --device "$(IOS_DEVICE)" --apple-team-id "$(IOS_TEAM_ID)"

ios-debug:
	cd $(UI_DIR) && RUST_LOG=debug dx serve -p mobile --ios

android:
	cd $(UI_DIR) && RUST_LOG="$(WARMMY_LOG)" dx serve -p mobile --android

android-debug:
	cd $(UI_DIR) && RUST_LOG=debug dx serve -p mobile --android

desktop:
	cd $(UI_DIR) && RUST_LOG="$(WARMMY_LOG)" dx serve -p desktop --desktop

desktop-debug:
	cd $(UI_DIR) && RUST_LOG=debug dx serve -p desktop --desktop

web:
	cd $(UI_DIR) && RUST_LOG="$(WARMMY_LOG)" dx serve -p web --web

web-debug:
	cd $(UI_DIR) && RUST_LOG=debug dx serve -p web --web

web-release:
	cd $(UI_DIR)/packages/web && dx bundle --web --release --base-path "$(WEB_BASE_PATH)" --out-dir "$(WEB_DIST)"
