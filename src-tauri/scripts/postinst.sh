#!/bin/sh
set -e

AUTOSTART="/etc/xdg/autostart/spotlight.desktop"
mkdir -p /etc/xdg/autostart

cat > "$AUTOSTART" << 'EOF'
[Desktop Entry]
Type=Application
Version=1.0
Name=Spotlight
Comment=Universal launcher for Linux
Exec=/usr/bin/spotlight
Icon=spotlight
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
EOF

chmod 644 "$AUTOSTART"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /etc/xdg/autostart 2>/dev/null || true
fi

exit 0
