use crate::rss;
use web_view::*;

pub fn make_window() {
    web_view::builder()
        .title("RSass")
        .size(1024, 768)
        .content(Content::Html(make_html()))
        .invoke_handler(|view, arg| handle_invoke(view, arg))
        .user_data(Vec::new()) //rss::get_unread_entries())
        .run()
        .unwrap();
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
  <body>
    <h1 id="headline">

    </h1>
    <div id = "body"> </div>
    <button id="next-btn" onclick="requestNext()">Next</button>
    <button id="open-url-btn" onclick="openUrl()">Open in Browser</button>
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
        "next" => {
            let data = webview.user_data_mut();
            rss::mark_as_read(&data.pop().expect("Current entry is empty"));
            match data.last() {
                Some(x) => {
                    let serialized = serde_json::to_string(x).unwrap();
                    webview.eval(&format!(r#"render({});"#, serialized))
                }
                None => webview.eval("displayDone();"),
            }
        }
        _ => panic!("Unknown argument: {}", arg),
    }
}
