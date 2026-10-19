#!/usr/bin/env bash
# Simple macOS build without code signing
# Just for testing the app locally

set -euo pipefail

TARGET=aarch64-apple-darwin
DIST=dist
VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"

echo "==> Building for $TARGET with version $VERSION"

# Build the app
echo "==> cargo build (release)"
cargo build --release --target "$TARGET"

# Create icon from SVG
echo "==> Creating icon"
rm -rf "$DIST" && mkdir -p "$DIST"
ICONSET="$DIST/OpenCADStudio.iconset"
mkdir -p "$ICONSET"

for SIZE in 16 32 64 128 256 512 1024; do
    rsvg-convert -w $SIZE -h $SIZE assets/logo.svg -o "$ICONSET/icon_${SIZE}x${SIZE}.png"
done

for BASE in 16 32 128 256 512; do
    cp "$ICONSET/icon_$((BASE * 2))x$((BASE * 2)).png" "$ICONSET/icon_${BASE}x${BASE}@2x.png"
done

iconutil -c icns "$ICONSET" -o "$DIST/AppIcon.icns"

echo "==> Assembling .app"
APP="$DIST/QVCAD.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp "target/$TARGET/release/OpenCADStudio" "$APP/Contents/MacOS/QVCAD"
chmod +x "$APP/Contents/MacOS/QVCAD"

# Copy icon
cp "$DIST/AppIcon.icns" "$APP/Contents/Resources/"

# Update and copy Info.plist
sed "s/__VERSION__/$VERSION/g" packaging/Info.plist > "$APP/Contents/Info.plist"

echo "==> Ad-hoc signing"
codesign --force --sign - "$APP"

echo "==> Done!"
echo "App: $APP"
echo ""
echo "To test:"
echo "  open $APP"
echo ""
echo "To create DMG:"
echo "  mkdir dmg-staging && cp -R $APP dmg-staging/"
echo "  ln -s /Applications dmg-staging/"
echo "  hdiutil create -volname 'QVCAD' -srcfolder dmg-staging -format UDZO -o releases/QVCAD-macOS-test.dmg"
