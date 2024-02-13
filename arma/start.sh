#!/bin/bash

cd /home/nacor/Steam/arma3

rm screenlog.*

if ! screen -list | grep -q "arma_server"; then
    screen -S arma_server -L -d -m ./arma3server_x64 -name=server -config=server.cfg
fi