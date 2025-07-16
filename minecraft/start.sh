
#!/bin/bash
# =============================================================
# Rust Game Hosting Server - minecraft/start.sh
# -------------------------------------------------------------
# STATUS: Project is in limbo and may not work on newer Rust versions.
# This script starts the Minecraft server in a screen session.
# =============================================================

cd /home/nacor/minecraft

rm screenlog.*

if ! screen -list | grep -q "minecraft_server"; then
    screen -S minecraft_server -L -d -m java -Xms1024M -Xmx4G -jar server.jar nogui
fi