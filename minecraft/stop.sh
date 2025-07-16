
#!/bin/bash
# =============================================================
# Rust Game Hosting Server - minecraft/stop.sh
# -------------------------------------------------------------
# STATUS: Project is in limbo and may not work on newer Rust versions.
# This script stops the Minecraft server running in a screen session.
# =============================================================

screen -S minecraft_server -p 0 -X stuff "stop^M"

rm screenlog.*
