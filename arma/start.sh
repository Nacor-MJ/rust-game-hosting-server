
#!/bin/bash
# =============================================================
# Rust Game Hosting Server - arma/start.sh
# -------------------------------------------------------------
# STATUS: Project is in limbo and may not work on newer Rust versions.
# This script starts the Arma server in a screen session.
# =============================================================

cd /home/nacor/Steam/arma3

rm screenlog.*

if ! screen -list | grep -q "arma_server"; then
    screen -S arma_server -L -d -m ./arma3server_x64 -name=server -config=server.cfg
fi