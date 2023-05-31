use client::sauron;
use client::sauron::prelude::*;
use client::{App, Msg};
use crate::http::page::jss::jss;

pub fn index(app: &App) -> Node<Msg> {
    println!("app: {:#?}", app);
    let serialized_state = serde_json::to_string(&app).unwrap();
    node! {
        <!doctype html>
        <html lang="en">
            <head>
                <title>"Example app"</title>
                <script type="module">
                    {text!("
import init, {{ main }} from '/pkg/client.js';
async function start() {{
  await init();
  let app_state = String.raw`{}`;
  await main(app_state);
}}
start();\
                        ", serialized_state)}
                </script>
                <link rel="stylesheet" type="text/css" href="/style/main.css"></link>
            </head>
            { app.view() }
        </html>
    }
}

pub fn style() -> String {
    jss! {
        "body": {
            font_family: "'Arial', sans-serif",
            margin: "0 auto",
            max_width: "90rem",
        },
        "header": {
            display: "flex",
            background_color: "#333",
            color: "#fff",
        },
        "header a": {
            color: "#fff",
            padding: "0.5em 2em",
            text_decoration: "none",
        },
        "header a.active": {
            background_color: "#c00",
        },
        "table.file-listing": {
            width: "100%",
        },
        "table.file-listing td.gitobj-tree span::before": {
            content: "'\\1f4c1 '",
        },
        "table.file-listing td.gitobj-blob span::before": {
            content: "'\\1f4c4 '",
        },
        "#indicator, #indicator div": {
            height: "10px",
        },
        "#indicator div.loading": {
            background: "repeating-linear-gradient(-45deg, #c00 0, #c00 33%, #333 0, #333 66%)",
            background_size: "20px",
            animation: "loading_indicator 1s infinite linear",
        },
        "@keyframes loading_indicator": {
            from: {
                background_position: "0px 0px",
            },
            to: {
                background_position: "-40px 0px",
            }
        },
    }
}
