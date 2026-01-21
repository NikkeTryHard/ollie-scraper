import asyncio
import json
import os
import urllib.request
import websockets
from dotenv import load_dotenv
from notifier import notify_status_change

load_dotenv()

DISCORD_TOKEN = os.getenv("DISCORD_TOKEN")
CHANNEL_ID = os.getenv("CHANNEL_ID")
GATEWAY_URL = "wss://gateway.discord.gg/?v=9&encoding=json"
API_BASE = "https://discord.com/api/v9"
POLL_INTERVAL = 1.5  # Poll every 1.5 seconds (safe under rate limit)

last_channel_name = None


def get_current_channel_name():
    """Fetch the current channel name via REST API."""
    url = f"{API_BASE}/channels/{CHANNEL_ID}"
    req = urllib.request.Request(url)
    req.add_header("Authorization", DISCORD_TOKEN)
    req.add_header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) Chrome/120.0.0.0")
    try:
        with urllib.request.urlopen(req, timeout=5) as response:
            data = json.loads(response.read().decode())
            return data.get("name")
    except Exception as e:
        print(f"Poll error: {e}")
        return None


async def poll_channel():
    """Backup polling loop - checks channel name every POLL_INTERVAL seconds."""
    global last_channel_name
    while True:
        await asyncio.sleep(POLL_INTERVAL)
        name = get_current_channel_name()
        if name and last_channel_name and name != last_channel_name:
            print(f"[POLL] Name changed from '{last_channel_name}' to '{name}'")
            notify_status_change(name)
            last_channel_name = name
        elif name and not last_channel_name:
            last_channel_name = name


async def send_identify(ws):
    identify_payload = {"op": 2, "d": {"token": DISCORD_TOKEN, "properties": {"$os": "linux", "$browser": "Chrome", "$device": "Chrome"}}}
    await ws.send(json.dumps(identify_payload))
    print("Sent Identify payload")


async def heartbeat(ws, interval):
    while True:
        await asyncio.sleep(interval / 1000)
        await ws.send(json.dumps({"op": 1, "d": None}))
        print("Sent heartbeat")


async def listen():
    global last_channel_name

    async with websockets.connect(GATEWAY_URL, max_size=10_000_000) as ws:
        # Receive Hello (op 10)
        hello = json.loads(await ws.recv())
        heartbeat_interval = hello["d"]["heartbeat_interval"]
        print(f"Connected. Heartbeat interval: {heartbeat_interval}ms")

        # Start heartbeat task
        asyncio.create_task(heartbeat(ws, heartbeat_interval))

        # Send Identify
        await send_identify(ws)

        # Listen for events
        async for message in ws:
            data = json.loads(message)
            op = data.get("op")
            t = data.get("t")

            # Handle heartbeat ACK
            if op == 11:
                print("Heartbeat ACK received")
                continue

            # Handle CHANNEL_UPDATE
            if op == 0 and t == "CHANNEL_UPDATE":
                channel_data = data.get("d", {})
                channel_id = channel_data.get("id")
                channel_name = channel_data.get("name")

                if channel_id == CHANNEL_ID:
                    print(f"Channel update detected: {channel_name}")
                    if last_channel_name is not None and channel_name != last_channel_name:
                        print(f"Name changed from '{last_channel_name}' to '{channel_name}'")
                        notify_status_change(channel_name)
                    last_channel_name = channel_name


async def run_both():
    """Run both WebSocket listener and polling concurrently."""
    await asyncio.gather(listen(), poll_channel())


def main():
    global last_channel_name

    if not DISCORD_TOKEN:
        print("Error: DISCORD_TOKEN not set in .env")
        return
    if not CHANNEL_ID:
        print("Error: CHANNEL_ID not set in .env")
        return

    # Fetch and display current channel name
    last_channel_name = get_current_channel_name()
    if last_channel_name:
        print(f"Current channel name: '{last_channel_name}'")
    else:
        print("Warning: Could not fetch initial channel name")

    print(f"Monitoring channel: {CHANNEL_ID}")
    print(f"Polling every {POLL_INTERVAL}s + WebSocket real-time events")
    asyncio.run(run_both())


if __name__ == "__main__":
    main()
