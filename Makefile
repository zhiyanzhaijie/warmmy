UI_DIR := crates/ui

# Keep third-party tracing quiet by default. Raise our crates to info.
WARMMY_LOG ?= warn,ui=info,api=info,app=info,infra=info,adapters=info

.PHONY: ios ios-debug android android-debug web web-debug

ios:
	cd $(UI_DIR) && RUST_LOG="$(WARMMY_LOG)" dx serve -p mobile --ios

ios-debug:
	cd $(UI_DIR) && RUST_LOG=debug dx serve -p mobile --ios

android:
	cd $(UI_DIR) && RUST_LOG="$(WARMMY_LOG)" dx serve -p mobile --android

android-debug:
	cd $(UI_DIR) && RUST_LOG=debug dx serve -p mobile --android

web:
	cd $(UI_DIR) && RUST_LOG="$(WARMMY_LOG)" dx serve -p web --web

web-debug:
	cd $(UI_DIR) && RUST_LOG=debug dx serve -p web --web
