#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

nohup ./venv/bin/python3 -u monitor.py > scraper.log 2>&1 &
echo $! > scraper.pid
echo "Scraper started in background with PID $(cat scraper.pid)"
