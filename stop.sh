#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Stopping Discord scraper..."

# Kill the main scraper process
if [ -f scraper.pid ]; then
    kill $(cat scraper.pid) 2>/dev/null && echo "Scraper stopped"
    rm -f scraper.pid
fi

# Kill any running mpv sounds
pkill -f "mpv.*boom.mp3" 2>/dev/null

echo "All stopped."
