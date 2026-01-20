# Discord Channel Status Scraper Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan batch by batch.

**Goal:** Build a real-time Discord channel name scraper that triggers Ubuntu notifications and plays a sound when the status (channel name) changes.

**Architecture:** A lightweight Python script connecting to the Discord Gateway via WebSockets. It listens for `CHANNEL_UPDATE` events, compares name changes, and triggers local system commands (`notify-send`, `paplay`).

**Tech Stack:** Python 3, `websockets`, `python-dotenv`, `notify-send`, `paplay`.

---

### Batch 1: Setup and Environment

**Goal:** Initialize the project environment and configuration.

#### Task 1.1: Environment Configuration

**Files:**

- Create: `.env`
- Create: `.gitignore`

**Step 1: Create .env template**

```text
DISCORD_TOKEN=your_token_here
CHANNEL_ID=1371980076949835886
```

**Step 2: Create .gitignore**

```text
.env
__pycache__/
*.py[cod]
```

**Step 3: Commit**

```bash
git add .env .gitignore
git commit -m "chore: initial environment setup"
```

#### Task 1.2: Dependency Management

**Files:**

- Create: `requirements.txt`

**Step 1: Add dependencies**

```text
websockets
python-dotenv
```

**Step 2: Install dependencies**
Run: `pip install -r requirements.txt`

**Step 3: Commit**

```bash
git add requirements.txt
git commit -m "chore: add python dependencies"
```

---

### Batch 2: Notification System (TDD)

**Goal:** Create a reliable notification wrapper for Ubuntu.

#### Task 2.1: Notification Wrapper

**Files:**

- Create: `notifier.py`
- Create: `tests/test_notifier.py`

**Step 1: Write the failing test**

```python
import pytest
from unittest.mock import patch
from notifier import notify_status_change

def test_notify_calls_system_commands():
    with patch('subprocess.run') as mock_run:
        notify_status_change("New Status")
        # Check if notify-send and paplay were called
        assert mock_run.call_count >= 1
```

**Step 2: Run test to verify it fails**
Run: `pytest tests/test_notifier.py`
Expected: FAIL (Module not found)

**Step 3: Implement minimal code**

```python
import subprocess
import os

def notify_status_change(new_name):
    # Visual notification
    subprocess.run(['notify-send', 'Discord Status Change', f'Channel is now: {new_name}'])

    # Audio notification
    sound_path = os.path.join(os.getcwd(), 'boom.mp3')
    if os.path.exists(sound_path):
        subprocess.run(['paplay', sound_path])
```

**Step 4: Run test to verify it passes**
Run: `pytest tests/test_notifier.py`
Expected: PASS

**Step 5: Commit**

```bash
git add notifier.py tests/test_notifier.py
git commit -m "feat: add notification system"
```

---

### Batch 3: Discord Gateway Listener

**Goal:** Connect to Discord and listen for channel updates.

#### Task 3.1: WebSocket Client

**Files:**

- Create: `monitor.py`

**Step 1: Implement WebSocket logic**

- Load `.env`
- Connect to `wss://gateway.discord.gg/?v=9&encoding=json`
- Send Identify payload
- Listen for `CHANNEL_UPDATE`
- Compare `name` and trigger `notifier.py`

**Step 2: Manual Verification**

- Run: `python monitor.py`
- Verify heartbeat starts and connection stays open.

**Step 3: Commit**

```bash
git add monitor.py
git commit -m "feat: implement discord gateway listener"
```

---

### Batch 4: Background Execution

**Goal:** Ensure the script runs persistently.

#### Task 4.1: Background Script

**Files:**

- Create: `run.sh`

**Step 1: Create wrapper script**

```bash
#!/bin/bash
nohup python3 monitor.py > scraper.log 2>&1 &
echo $! > scraper.pid
echo "Scraper started in background with PID $(cat scraper.pid)"
```

**Step 2: Commit**

```bash
git add run.sh
chmod +x run.sh
git commit -m "feat: add background execution script"
```
