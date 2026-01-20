import subprocess
import os


def notify_status_change(new_name):
    # Visual notification
    subprocess.run(["notify-send", "Discord Status Change", f"Channel is now: {new_name}"])

    # Audio notification
    sound_path = "/home/nikketryhard/dev/ollie-scraper/boom.mp3"
    if os.path.exists(sound_path):
        subprocess.run(["paplay", sound_path])
