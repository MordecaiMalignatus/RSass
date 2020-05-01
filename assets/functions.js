
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
    open(document.__data.current_entry.html_url, "_blank");
};

function displayDone() {
    document.getElementById("headline").innerHTML = "And done!";
    document.getElementById("body").innerHTML = "";
};

requestNext();
