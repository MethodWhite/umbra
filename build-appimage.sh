#!/bin/bash
# UMBRA AppImage Builder v2 (Tauri-based)
# Produces a single AppImage with everything bundled.
set -e

APP="Umbra"
VERSION="0.1.0"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Building $APP AppImage v$VERSION ==="
echo ""

# 1. Build Angular frontend
echo "[1/4] Building Angular frontend..."
cd "$SCRIPT_DIR/frontend"
pnpm install --frozen-lockfile 2>/dev/null || pnpm install
pnpm exec ng build --configuration production
echo "  ✅ Frontend built"

# 2. Build Tauri desktop app (produces AppImage natively)
echo "[2/4] Building Tauri desktop app..."
export PATH="$HOME/.local/share/pnpm/bin:$HOME/.local/share/fnm/aliases/default/bin:$PATH"
cd "$SCRIPT_DIR/src-tauri"
# Build without re-running frontend build (already done in step 1)
cargo build --release -p umbra-desktop 2>&1 | tail -3
echo "  ✅ Tauri binary built"

# 3. Build AppImage manually
echo "[3/4] Building AppImage package..."
# Find the binary
BINARY="$SCRIPT_DIR/src-tauri/target/release/umbra-desktop"
if [ ! -f "$BINARY" ]; then
    echo "  ❌ Binary not found: $BINARY"
    exit 1
fi

BUILD_DIR="/tmp/umbra-appimage"
APP_DIR="$BUILD_DIR/Umbra.AppDir"
rm -rf "$BUILD_DIR"
mkdir -p "$APP_DIR/usr/bin"
mkdir -p "$APP_DIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/512x512/apps"

# Copy binary
cp "$BINARY" "$APP_DIR/usr/bin/umbra"

# Copy frontend dist
cp -r "$SCRIPT_DIR/frontend/dist/browser" "$APP_DIR/usr/share/umbra/frontend" 2>/dev/null

# Copy icon
cp "$SCRIPT_DIR/logo.svg" "$APP_DIR/umbra.svg"
cp "$SCRIPT_DIR/src-tauri/icons/icon.png" "$APP_DIR/usr/share/icons/hicolor/512x512/apps/umbra.png" 2>/dev/null

# Create AppRun
cat > "$APP_DIR/AppRun" << 'APPRUN'
#!/bin/bash
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="$HERE/usr/bin:$PATH"
export UMBRA_FRONTEND_DIR="$HERE/usr/share/umbra/frontend"
export UMBRA_HOME="$HOME/.umbra"
mkdir -p "$UMBRA_HOME"

# Run in background mode if --background is passed
if [ "$1" = "--background" ]; then
    exec "$HERE/usr/bin/umbra" 2>/dev/null &
    disown
    exit 0
fi

exec "$HERE/usr/bin/umbra"
APPRUN
chmod +x "$APP_DIR/AppRun"

# Create .desktop file
cat > "$APP_DIR/umbra.desktop" << DESKTOP
[Desktop Entry]
Name=UMBRA
Comment=AI Agent System
Exec=AppRun
Icon=umbra
Terminal=false
Type=Application
Categories=Utility;AI;
StartupNotify=true
DESKTOP

# Download appimagetool
if [ ! -f "/tmp/appimagetool" ]; then
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" -O /tmp/appimagetool 2>/dev/null || {
        echo "  ⚠️ Could not download appimagetool"
        exit 1
    }
    chmod +x /tmp/appimagetool
fi

# Build AppImage
echo "[4/4] Packaging AppImage..."
ARCH=x86_64 /tmp/appimagetool "$APP_DIR" "$SCRIPT_DIR/UMBRA-$VERSION-x86_64.AppImage" 2>&1 | tail -3
echo "  ✅ AppImage packaged"

# 3. Find the AppImage
APPIMAGE_PATH=$(find "$SCRIPT_DIR/src-tauri/target/release/bundle" -name "*.AppImage" 2>/dev/null | head -1)
if [ -z "$APPIMAGE_PATH" ]; then
    # Try alternative location
    APPIMAGE_PATH=$(find "$SCRIPT_DIR/src-tauri/target" -name "*.AppImage" 2>/dev/null | head -1)
fi

echo ""
echo "=== Build complete ==="
echo ""
if [ -n "$APPIMAGE_PATH" ]; then
    echo "  AppImage: $APPIMAGE_PATH"
    ls -lh "$APPIMAGE_PATH"
    cp "$APPIMAGE_PATH" "$SCRIPT_DIR/$APP-$VERSION-x86_64.AppImage"
    echo "  Copiado a: $SCRIPT_DIR/$APP-$VERSION-x86_64.AppImage"
    echo ""
    echo "  Para instalar:"
    echo "    chmod +x '$APP-$VERSION-x86_64.AppImage'"
    echo "    ./$APP-$VERSION-x86_64.AppImage"
    echo ""
    echo "  Para auto-inicio en segundo plano:"
    echo "    mkdir -p ~/.config/autostart"
    echo "    cat > ~/.config/autostart/umbra.desktop << EOF"
    echo "    [Desktop Entry]"
    echo "    Type=Application"
    echo "    Name=UMBRA"
    echo "    Exec=$HOME/$APP-$VERSION-x86_64.AppImage --background"
    echo "    Terminal=false"
    echo "    X-GNOME-Autostart-enabled=true"
    echo "    EOF"
else
    echo "  ⚠️  No se encontró AppImage en las rutas esperadas."
    echo "  Buscando en:"
    find "$SCRIPT_DIR/src-tauri/target" -name "*.AppImage" 2>/dev/null || echo "  (ninguno)"
fi
