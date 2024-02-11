#!/bin/bash

cd /home/nacor/web_server/Minecraft

rm screenlog.*

if ! screen -list | grep -q "minecraft_server"; then
    screen -S minecraft_server -L -d -m java -Xms1024M -Xmx4G -jar server.jar nogui
fi