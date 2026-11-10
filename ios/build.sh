#!/bin/bash
# Builds LocationSpoofer.ipa (unsigned: Sideloadly signs it at install time).
# Runs on macOS with Xcode and Rust installed — normally on GitHub Actions.
set -euo pipefail
cd "$(dirname "$0")/.."

APP_NAME="LocationSpoofer"
MIN_IOS="16.0"
OUT="build"
APP="$OUT/Payload/$APP_NAME.app"

export IPHONEOS_DEPLOYMENT_TARGET="$MIN_IOS"

echo "== 1/3 Rust core (aarch64-apple-ios) =="
rustup target add aarch64-apple-ios
cargo build --release -p spoofer_core --target aarch64-apple-ios

echo "== 2/3 Swift app =="
rm -rf "$OUT"
mkdir -p "$APP"
SDK="$(xcrun --sdk iphoneos --show-sdk-path)"
xcrun --sdk iphoneos swiftc \
  -target "arm64-apple-ios$MIN_IOS" \
  -sdk "$SDK" \
  -swift-version 5 \
  -O \
  -parse-as-library \
  -module-name "$APP_NAME" \
  -import-objc-header ios/Bridge.h \
  ios/Sources/*.swift \
  -L target/aarch64-apple-ios/release \
  -lspoofer_core \
  -framework Security \
  -framework SystemConfiguration \
  -lresolv \
  -liconv \
  -Xlinker -dead_strip \
  -o "$APP/$APP_NAME"

echo "== 3/3 Bundle =="
cp ios/Info.plist "$APP/Info.plist"
cp ios/AppIcon60x60@2x.png ios/AppIcon60x60@3x.png "$APP/"
printf 'APPL????' > "$APP/PkgInfo"
( cd "$OUT" && zip -qr "$APP_NAME.ipa" Payload )
ls -lh "$OUT/$APP_NAME.ipa"
