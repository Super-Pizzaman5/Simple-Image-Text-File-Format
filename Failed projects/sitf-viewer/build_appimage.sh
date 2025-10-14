#!/bin/bash
set -e

APPDIR="SITFViewer.AppDir"

echo "Building static Rust binary..."
cargo build --release --target x86_64-unknown-linux-musl

echo "Creating AppDir..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"

# Copy static binary
cp target/x86_64-unknown-linux-musl/release/sitf-viewer "$APPDIR/usr/bin/"

# Desktop file
cat > "$APPDIR/sitf-viewer.desktop" <<EOL
[Desktop Entry]
Type=Application
Name=SITF Viewer
Comment=Display SITF files in the terminal
Exec=sitf-viewer %f
Icon=icon
Terminal=true
Categories=Utility;
EOL

# Create placeholder icon
convert -size 256x256 xc:none "$APPDIR/icon.png"

# Download appimagetool if not present
if [ ! -f appimagetool-x86_64.AppImage ]; then
    wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x appimagetool-x86_64.AppImage
fi

# Build AppImage
echo "Building AppImage..."
./appimagetool-x86_64.AppImage "$APPDIR"

echo "✅ AppImage built: SITFViewer-x86_64.AppImage"


