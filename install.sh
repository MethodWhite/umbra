#!/bin/bash
set -e

APP="Umbra"
VERSION="0.1.0"

echo "=== Installing $APP v$VERSION ==="

# Check requirements
echo "[1/5] Checking requirements..."
command -v python3 >/dev/null 2>&1 || { echo "ERROR: python3 required"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "ERROR: Rust/Cargo required"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "WARNING: Node.js not found (frontend build will be skipped)"; }

# Create directories
echo "[2/5] Creating directories..."
mkdir -p "$HOME/.umbra"
mkdir -p "$HOME/.local/bin"
mkdir -p "$HOME/.local/share/applications"

# Build Rust backend
echo "[3/5] Building Rust backend..."
cd "$(dirname "$0")"
cargo build --release
mkdir -p "$HOME/.local/lib/umbra"
cp target/release/umbra "$HOME/.local/lib/umbra/umbra-daemon"
cp target/release/umbra-gui "$HOME/.local/lib/umbra/umbra-gui"

# Setup Python venv
echo "[4/5] Setting up Python environment..."
JARVIS_DIR="$(dirname "$0")/../jarvis"
if [ -d "$JARVIS_DIR" ]; then
    cd "$JARVIS_DIR"
    if [ ! -d "venv" ]; then
        python3 -m venv venv
    fi
    source venv/bin/activate
    pip install --quiet fastapi uvicorn httpx pydantic cryptography tomli-w 2>/dev/null || true
    deactivate
    cd - >/dev/null
fi

# Create launcher script
echo "[5/5] Creating launcher..."
UMBRA_BIN="$HOME/.local/lib/umbra"
JARVIS_DIR_FINAL="$JARVIS_DIR"
cat > "$HOME/.local/bin/umbra" << LAUNCHER
#!/bin/bash
HERE="$UMBRA_BIN"
JARVIS="$JARVIS_DIR_FINAL"
export UMBRA_HOME="\$HOME/.umbra"
export PATH="\$HERE:\$PATH"

case "\${1:-}" in
    start)
        nohup "\$HERE/umbra-daemon" --api-port 8484 --no-frontend > "\$UMBRA_HOME/backend.log" 2>&1 &
        echo "Umbra backend started (PID \$!)"
        ;;
    stop)
        pkill -f "umbra-daemon" 2>/dev/null || true
        echo "Umbra stopped"
        ;;
    gui)
        "\$HERE/umbra-gui"
        ;;
    frontend)
        SSL_FLAG=""
        if [ "\$2" = "--ssl" ]; then SSL_FLAG="--ssl"; fi
        if [ -d "\$JARVIS" ]; then
            cd "\$JARVIS"
            echo "Iniciando frontend en http://127.0.0.1:8340"
            [ -n "\$SSL_FLAG" ] && echo "  con HTTPS habilitado"
            exec "\$JARVIS/venv/bin/python" server.py --port 8340 \$SSL_FLAG
        else
            echo "Frontend directory not found"
            exit 1
        fi
        ;;
    status)
        if pgrep -f "umbra-daemon" >/dev/null 2>&1; then echo "Umbra: RUNNING"; else echo "Umbra: STOPPED"; fi
        ;;
    help)
        echo "Usage: umbra {start|stop|gui|frontend|status}"
        echo ""
        echo "  start     Start backend daemon (port 8484)"
        echo "  stop      Stop backend daemon"
        echo "  gui       Open desktop GUI (egui)"
        echo "  frontend  Start web frontend (port 8340)"
        echo "  status    Check if running"
        echo ""
        echo "First run:  umbra start && umbra frontend"
        echo "Desktop:    umbra gui"
        ;;
    *)
        echo "Usage: umbra {start|stop|gui|frontend|status|help}"
        ;;
esac
LAUNCHER
chmod +x "$HOME/.local/bin/umbra"

# Create .desktop file
cat > "$HOME/.local/share/applications/umbra.desktop" << DESKTOP
[Desktop Entry]
Name=Umbra
Comment=AI Agent System — MT4 Automated Trading
Exec=$HOME/.local/bin/umbra gui
Icon=$HOME/.umbra/logo.svg
Terminal=false
Type=Application
Categories=Utility;Finance;AI;
StartupNotify=true
DESKTOP

# Copy assets
cp "$(dirname "$0")/logo.svg" "$HOME/.umbra/" 2>/dev/null || true
cp "$(dirname "$0")/target/release/build/umbra/out/logo.png" "$HOME/.umbra/" 2>/dev/null || true

# Auto-start backend
echo ""
echo "=== Installation complete ==="
echo ""
echo "  Umbra v$VERSION instalado"
echo ""
echo "  First run (quick start):"
echo "    umbra start          Iniciar backend"
echo "    umbra frontend       Abrir interfaz web"
echo ""
echo "  Desktop:"
echo "    umbra gui            Interfaz gráfica"
echo ""
echo "  Other:"
echo "    umbra stop           Detener backend"
echo "    umbra status         Estado del sistema"
echo "    umbra help           Ayuda"
echo ""
echo "  O desde el menú de aplicaciones como 'Umbra'"
echo ""

# Auto-start on first install
"$HOME/.local/bin/umbra" start 2>/dev/null || true
echo "Backend auto-iniciado. Abriendo interfaz web..."
sleep 1
echo "→ Abre http://localhost:8340 en tu navegador"
echo "→ O ejecuta 'umbra gui' para la versión de escritorio"
