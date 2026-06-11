#!/bin/bash
HERE="/home/methodwhite/.local/lib/umbra"
JARVIS="./../jarvis"
export UMBRA_HOME="$HOME/.umbra"
export PATH="$HERE:$PATH"

case "${1:-}" in
    start)
        nohup "$HERE/umbra-daemon" --api-port 8484 --no-frontend > "$UMBRA_HOME/backend.log" 2>&1 &
        echo "Umbra backend started (PID $!)"
        ;;
    stop)
        pkill -f "umbra-daemon" 2>/dev/null || true
        echo "Umbra stopped"
        ;;
    gui)
        "$HERE/umbra-gui"
        ;;
    frontend)
        if [ -d "$JARVIS" ]; then
            cd "$JARVIS"
            "$JARVIS/venv/bin/python" server.py --port 8340
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
