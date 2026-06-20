# provision-renew

Minimal Xcode project template used by `scripts/renew-profiles.sh` to refresh iOS provisioning profiles with `xcodebuild -allowProvisioningUpdates -allowProvisioningDeviceRegistration`.

The generated app is temporary and discarded. Its only purpose is to let Xcode renew the profile for the Warmmy bundle id before `dx serve --ios --device` or `dx bundle --ios --release` runs.
