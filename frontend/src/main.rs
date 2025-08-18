use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ShortenRequest {
    url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ShortenResponse {
    short_code: String,
}

fn main() {
    dioxus::launch(app);
}

#[component]
fn app() -> Element {
    let mut url_input = use_signal(String::new);
    let mut shortened_url = use_signal(String::new);
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(String::new);

    let shorten_url = move |_| {
        let url = url_input.read().clone();
        if url.is_empty() {
            error_message.set("Please enter a URL".to_string());
            return;
        }

        is_loading.set(true);
        error_message.set(String::new());

        spawn(async move {
            match shorten_url_request(url).await {
                Ok(response) => {
                    shortened_url.set(response.short_code);
                }
                Err(e) => {
                    error_message.set(format!("Error: {}", e));
                }
            }
            is_loading.set(false);
        });
    };

    let copy_to_clipboard = move |_| {
        let url = shortened_url.read().clone();
        if !url.is_empty() {
            spawn(async move {
                if let Some(window) = web_sys::window() {
                    let clipboard = window.navigator().clipboard();
                    let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&url)).await;
                }
            });
        }
    };

    rsx! {
        div {
            style: "max-width: 600px; margin: 50px auto; padding: 20px; font-family: Arial, sans-serif;",
            h1 {
                style: "text-align: center; color: #333; margin-bottom: 30px;",
                "URL Shortener"
            }

            div {
                style: "margin-bottom: 20px;",
                input {
                    r#type: "url",
                    placeholder: "Enter URL to shorten...",
                    value: "{url_input.read()}",
                    style: "width: 100%; padding: 12px; border: 2px solid #ddd; border-radius: 4px; font-size: 16px; box-sizing: border-box;",
                    oninput: move |evt| url_input.set(evt.value()),
                }
            }

            div {
                style: "text-align: center; margin-bottom: 20px;",
                button {
                    onclick: shorten_url,
                    disabled: *is_loading.read(),
                    style: "background-color: #007bff; color: white; padding: 12px 30px; border: none; border-radius: 4px; font-size: 16px; cursor: pointer; disabled:opacity-0.6;",
                    if *is_loading.read() {
                        "Shortening..."
                    } else {
                        "Shorten URL"
                    }
                }
            }

            if !error_message.read().is_empty() {
                div {
                    style: "background-color: #f8d7da; color: #721c24; padding: 12px; border-radius: 4px; margin-bottom: 20px;",
                    "{error_message.read()}"
                }
            }

            if !shortened_url.read().is_empty() {
                div {
                    style: "background-color: #d4edda; color: #155724; padding: 20px; border-radius: 4px; margin-top: 20px;",
                    h3 {
                        style: "margin-top: 0; margin-bottom: 15px;",
                        "Shortened URL:"
                    }
                    div {
                        style: "display: flex; gap: 10px; align-items: center;",
                        input {
                            r#type: "text",
                            value: "{shortened_url.read()}",
                            readonly: true,
                            style: "flex: 1; padding: 10px; border: 1px solid #ccc; border-radius: 4px; background-color: #f8f9fa;",
                        }
                        button {
                            onclick: copy_to_clipboard,
                            style: "background-color: #28a745; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer;",
                            "Copy"
                        }
                    }
                }
            }
        }
    }
}

async fn shorten_url_request(url: String) -> Result<ShortenResponse, String> {
    let client = reqwest::Client::new();
    let mut payload = HashMap::new();
    payload.insert("url", url);

    let backend_url =
        std::env::var("BACKEND_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());

    let response = client
        .post(format!("{}/shorten", backend_url))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let shortened: ShortenResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(shortened)
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}
