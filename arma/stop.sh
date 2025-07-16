
#!/bin/bash
# =============================================================
# Rust Game Hosting Server - arma/stop.sh
# -------------------------------------------------------------
# STATUS: Project is in limbo and may not work on newer Rust versions.
# This script stops the Arma server running in a screen session.
# =============================================================

screen -S arma_server -p 0 -X stuff "^C"

rm screenlog.*
