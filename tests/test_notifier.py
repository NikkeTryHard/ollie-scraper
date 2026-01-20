import pytest
from unittest.mock import patch
from notifier import notify_status_change


def test_notify_calls_system_commands():
    with patch("subprocess.run") as mock_run:
        notify_status_change("New Status")
        # Check if notify-send and paplay were called
        assert mock_run.call_count >= 1
