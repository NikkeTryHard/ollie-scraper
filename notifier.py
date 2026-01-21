import subprocess
import os
import time
import signal

# Global flag to stop the alarm loop
alarm_running = False
alarm_pid_file = "/home/nikketryhard/dev/ollie-scraper/alarm.pid"


def notify_status_change(new_name):
    global alarm_running
    alarm_running = True

    # Save PID so stop.sh can kill this process
    with open(alarm_pid_file, "w") as f:
        f.write(str(os.getpid()))

    sound_path = "/home/nikketryhard/dev/ollie-scraper/boom.mp3"

    print(f"ALARM STARTED: Channel is now '{new_name}' - Run ./stop.sh to silence")

    while alarm_running:
        # Visual notification
        subprocess.run(["notify-send", "-u", "critical", "CHANNEL OPEN", f"Channel is now: {new_name}"])

        # Audio notification
        if os.path.exists(sound_path):
            subprocess.run(["mpv", "--no-video", "--really-quiet", sound_path])

        time.sleep(3)  # Wait 3 seconds before repeating
