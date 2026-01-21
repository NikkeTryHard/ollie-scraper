#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "========================================"
echo "   OLLIE SCRAPER STATUS"
echo "========================================"
echo ""

# Check if running
if [ -f scraper.pid ]; then
    PID=$(cat scraper.pid)
    if ps -p $PID > /dev/null 2>&1; then
        echo "STATUS:    RUNNING"
        echo "PID:       $PID"

        # Uptime
        UPTIME=$(ps -o etime= -p $PID | tr -d ' ')
        echo "UPTIME:    $UPTIME"

        # Memory usage
        MEM=$(ps -o rss= -p $PID | awk '{printf "%.1f MB", $1/1024}')
        echo "MEMORY:    $MEM"

        # CPU usage
        CPU=$(ps -o %cpu= -p $PID | tr -d ' ')
        echo "CPU:       ${CPU}%"
    else
        echo "STATUS:    STOPPED (stale PID file)"
    fi
else
    echo "STATUS:    STOPPED"
fi

echo ""
echo "----------------------------------------"
echo "   CHANNEL INFO"
echo "----------------------------------------"

# Get current channel name from log
if [ -f scraper.log ]; then
    CHANNEL_NAME=$(grep "Current channel name:" scraper.log | tail -1 | sed "s/.*Current channel name: '\(.*\)'/\1/")
    if [ -n "$CHANNEL_NAME" ]; then
        echo "CHANNEL:   $CHANNEL_NAME"
    fi

    # Count events
    WS_EVENTS=$(grep -c "Channel update detected" scraper.log 2>/dev/null || echo "0")
    POLL_EVENTS=$(grep -c "\[POLL\]" scraper.log 2>/dev/null || echo "0")
    HEARTBEATS=$(grep -c "Heartbeat ACK" scraper.log 2>/dev/null || echo "0")
    ALARMS=$(grep -c "ALARM STARTED" scraper.log 2>/dev/null || echo "0")

    echo ""
    echo "----------------------------------------"
    echo "   STATISTICS"
    echo "----------------------------------------"
    echo "WebSocket Events:  $WS_EVENTS"
    echo "Poll Detections:   $POLL_EVENTS"
    echo "Heartbeats:        $HEARTBEATS"
    echo "Alarms Triggered:  $ALARMS"

    echo ""
    echo "----------------------------------------"
    echo "   LAST 5 LOG ENTRIES"
    echo "----------------------------------------"
    tail -5 scraper.log
else
    echo "No log file found"
fi

echo ""
echo "========================================"
