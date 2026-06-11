#!/bin/bash
echo "=== Uninstalling Umbra ==="
rm -f "$HOME/.local/bin/umbra" "$HOME/.local/bin/umbra-gui"
rm -f "$HOME/.local/share/applications/umbra.desktop"
echo "Umbra uninstalled. Config files remain in ~/.umbra/"
echo "Remove them manually: rm -rf ~/.umbra"
