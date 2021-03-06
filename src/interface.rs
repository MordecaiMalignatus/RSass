use crate::{db, rss};
use std::error::Error;
use std::process::Command;
use web_view::*;

pub fn make_window() -> Result<(), Box<dyn Error>>{
    let unread = db::get_unread_entries(&db::get_db_connection())?;

    web_view::builder()
        .title("RSass")
        .size(550, 700)
        .content(Content::Html(make_html()))
        .invoke_handler(|view, arg| handle_invoke(view, arg))
        .user_data(unread)
        .run()
        .unwrap();

    Ok(())
}

fn make_html() -> String {
    format!(
        r#"
<!doctype html>
<html>
  <head>
    <title>Tiny RSS </title>
    {scripts}
    {styles}
  </head>
  <body class="near-black bg-washed-yellow">
  <div class="pl4">
    <h1 id="headline" class="h1 tracked">

    </h1>
    <div id = "body" class="measure"> </div>
    <div id="buttons" class="pb4">
      <button id="next-btn" onclick="requestNext()">Next</button>
      <button id="open-url-btn" onclick="openUrl()">Open in Browser</button>
    </div>
  </div>
  </body>
</html>
"#,
        scripts = format!(
            "<script>{}</script>",
            include_str!("../assets/functions.js")
        ),
        styles = format!(
            r#"<style type="text/css">{}</style>"#,
            include_str!("../assets/style.css")
        )
    )
}

fn handle_invoke(webview: &mut WebView<Vec<rss::Entry>>, arg: &str) -> WVResult {
    match arg {
        "init" => {
            let data = webview.user_data_mut();
            match data.last() {
                Some(x) => {
                    let serialized = serde_json::to_string(x).unwrap();
                    webview.eval(&format!(r#"render({});"#, serialized))
                }
                None => webview.eval("nothingNew();"),
            }
        }
        "next" => {
            let data = webview.user_data_mut();
            rss::mark_as_read(&data.pop().expect("Current entry is empty"))
                .expect("can't mark data as read");
            match data.last() {
                Some(x) => {
                    let serialized = serde_json::to_string(x).unwrap();
                    webview.eval(&format!(r#"render({});"#, serialized))
                }
                None => webview.eval("displayDone();"),
            }
        }
        "openCurrentUrl" => {
            let data = webview.user_data();
            match open_in_shell(&data.last().unwrap().html_url) {
                Ok(_) => webview.eval("openSuccessful()"),
                Err(_) => webview.eval("openFailed()"),
            }
        }
        _ => panic!("Unknown argument: {}", arg),
    }
}

#[cfg(target_os = "macos")]
fn open_in_shell(url: &str) -> Result<(), Box<dyn Error>> {
    match Command::new("open").arg(url).output() {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

#[cfg(target_family = "windows")]
fn open_in_shell(url: &str) -> Result<(), Box<dyn Error>> {
    match Command::new("start").arg(url).output() {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

#[cfg(target_os = "linux")]
fn open_in_shell(url: &str) -> Result<(), Box<dyn Error>> {
    match Command::new("xdg-open").arg(url).output() {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
