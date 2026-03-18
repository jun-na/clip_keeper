#!/bin/zsh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="ClipKeeper"
APP_DIR="$ROOT_DIR/target/release/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
BIN_PATH="$ROOT_DIR/target/release/$APP_NAME"
PLIST_PATH="$CONTENTS_DIR/Info.plist"

cd "$ROOT_DIR"
cargo build --release

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"
cp "$BIN_PATH" "$MACOS_DIR/$APP_NAME"
chmod +x "$MACOS_DIR/$APP_NAME"

cat > "$PLIST_PATH" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>ja</string>
    <key>CFBundleExecutable</key>
    <string>ClipKeeper</string>
    <key>CFBundleIdentifier</key>
    <string>com.jun-na.clipkeeper</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>ClipKeeper</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.9.3</string>
    <key>CFBundleVersion</key>
    <string>0.9.3</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF

echo "Created $APP_DIR"