import asyncio
import json
import os
import websockets
from dotenv import load_dotenv
from notifier import notify_status_change

load_dotenv()

DISCORD_TOKEN = os.getenv("DISCORD_TOKEN")
CHANNEL_ID = os.getenv("CHANNEL_ID")
GATEWAY_URL = "wss://gateway.discord.gg/?v=9&encoding=json"

last_channel_name = None


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

    async with websockets.connect(GATEWAY_URL) as ws:
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


def main():
    if not DISCORD_TOKEN:
        print("Error: DISCORD_TOKEN not set in .env")
        return
    if not CHANNEL_ID:
        print("Error: CHANNEL_ID not set in .env")
        return

    print(f"Monitoring channel: {CHANNEL_ID}")
    asyncio.run(listen())


if __name__ == "__main__":
    main()
