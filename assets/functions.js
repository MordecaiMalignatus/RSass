
document.__data = {};

function requestNext() {
    console.log("next!");
    external.invoke("next");
};

function render(entry) {
    console.log(entry);
    document.__data.current_entry = entry;

    document.getElementById("body").innerHTML = entry.rss_entry.content;
    document.getElementById("headline").innerHTML = entry.title;
};

function openUrl() {
    external.invoke("openCurrentUrl");
};

function displayDone() {
    document.getElementById("headline").innerHTML = "And done!";
    document.getElementById("body").innerHTML = "";
};

function openSuccessful() {
    console.log("Opened url" + document.__data.current_entry.title);
}

function openFailed() {
    console.log("Failed to open URL at: " + document.__data.current_entry.title);
}

external.invoke("init");
