#!/bin/sh
set -e

AUTOSTART="/etc/xdg/autostart/spotlight.desktop"
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
  if [ -f "$AUTOSTART" ]; then
    rm -f "$AUTOSTART"
  fi
fi

exit 0
