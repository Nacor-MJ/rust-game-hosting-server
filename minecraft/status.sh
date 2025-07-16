
#!/bin/bash
# =============================================================
# Rust Game Hosting Server - minecraft/status.sh
# -------------------------------------------------------------
# STATUS: Project is in limbo and may not work on newer Rust versions.
# This script checks the status of the Minecraft server in a screen session.
# =============================================================

screen -S minecraft_server -p 0 -X stuff "list^M"
