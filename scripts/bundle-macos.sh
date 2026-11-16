#!/usr/bin/env bash
# Build a universal Oneko.app (and a .dmg if hdiutil is available) into dist/.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

APP=dist/Oneko.app
rm -rf dist && mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/oneko" \
  target/aarch64-apple-darwin/release/oneko \
  target/x86_64-apple-darwin/release/oneko

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Oneko</string>
  <key>CFBundleDisplayName</key><string>Oneko</string>
  <key>CFBundleIdentifier</key><string>xyz.smoothsend.oneko</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleExecutable</key><string>oneko</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Ad-hoc signature so macOS doesn't refuse to launch the universal binary outright.
codesign --force --deep --sign - "$APP"

if command -v hdiutil >/dev/null; then
  hdiutil create -quiet -volname Oneko -srcfolder "$APP" -ov -format UDZO "dist/Oneko-$VERSION.dmg"
fi
ls -la dist
