async function start_arma() {
    sendPost('/arma/start');
    setTimeout(update_arma, 1000);
}
async function stop_arma() {
    sendPost('/arma/stop');
    setTimeout(update_arma, 1000);
}
var arma_global_state = false;
async function update_arma(force_update = false) {
    if (!force_update && !arma_global_state) {
        return;
    }

    const arma_status_response = await getUpdate('/arma/get_status');

    const arma_status = JSON.parse(arma_status_response);

    console.log(arma_status);

    var arma_new_status = "Status: " + arma_status.state;
    if (arma_status.state == "On") {
        arma_global_state = true;
        arma_new_status +=  ", Player count: " + arma_status.players.count;
        if (arma_status.players.name_tags.length > 0) {
            arma_new_status +=  ", Player nametags: " + arma_status.players.name_tags;
        }
    } else {
        arma_global_state = false;
    }

    const arma_status_div = document
        .getElementById("arma-status");
    arma_status_div.innerHTML = arma_new_status;
}

document.body.innerHTML +='<section id="arma-section"><h2>Arma Control</h2><button id="start-arma" onclick="start_arma()">Start</button><button id="stop-arma" onclick="stop_arma()">Stop</button><button id="update-arma" onclick="update_arma(true)">Update</button><div id="arma-status">Status: Off</div></section>'
