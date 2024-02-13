async function start_mc() {
    sendPost('/minecraft/start');
    setTimeout(update_minecraft, 1000);
}
async function stop_mc() {
    sendPost('/minecraft/stop');
    setTimeout(update_minecraft, 1000);
}
var mc_global_state = false;
async function update_minecraft(force_update = false) {
    if (!force_update && !mc_global_state) {
        return;
    }

    const mc_status_response = await getUpdate('/minecraft/get_status');

    const mc_status = JSON.parse(mc_status_response);

    console.log(mc_status);

    var mc_new_status = "Status: " + mc_status.state;
    if (mc_status.state == "On") {
        mc_global_state = true;
        mc_new_status +=  ", Player count: " + mc_status.players.count;
        if (mc_status.players.name_tags.length > 0) {
            mc_new_status +=  ", Player nametags: " + mc_status.players.name_tags;
        }
    } else {
        mc_global_state = false;
    }

    const minecraft_status_div = document
        .getElementById("minecraft-status");
    minecraft_status_div.innerHTML = mc_new_status;
}

document.body.innerHTML +='<section id="minecraft-section"><h2>Minecraft Control</h2><button id="start-mc" onclick="start_mc()">Start</button><button id="stop-mc" onclick="stop_mc()">Stop</button><button id="update-mc" onclick="update_minecraft(true)">Update</button><div id="minecraft-status">Status: Off</div></section>'
