use chrono::{DateTime, Local};
use dioxus::{
    document::eval,
    logger::tracing::{error, info},
    prelude::*,
};
use serde::{Deserialize, Serialize};

const FAVICON: Asset = asset!("/assets/favicon.svg");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            document::Meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1.0",
            }
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS }
            h1 { class: "title", "✨ Abi Quotes ✨" }
            div { class: "container",
                SuspenseBoundary {
                    fallback: move |_| rsx! {
                        div { class: "loading", "Loading..." }
                    },
                    AddQuote {}
                }
                SuspenseBoundary {
                    fallback: move |_| rsx! {
                        div { class: "loading", "Loading..." }
                    },
                    Quotes {}
                }
            }
        }
    }
}

#[component]
fn AddQuote() -> Element {
    rsx! {
        form {
            class: "createNew",
            method: "POST",
            onsubmit: move |e| async move {
                let vs = e.values();
                info!("{:?}", vs);
                let name = vs.get("quote").map(|v| v.0[0].clone());
                let author = vs.get("author").map(|v| v.0[0].clone());
                if let (Some(n), Some(a)) = (name, author) {
                    info!("Calling");
                    if let Ok(true) = add_quote(n, a).await {
                        eval("window.location = '/'");
                    }
                }
            },
            textarea {
                name: "quote",
                placeholder: "Your quote",
                cols: 40,
                rows: 3,
            }
            input { r#type: "text", name: "author", placeholder: "by who" }
            input { r#type: "submit", value: "Add" }
        }
    }
}

#[component]
fn Quotes() -> Element {
    let quotes: MappedSignal<Result<Vec<Quote>, ServerFnError>> =
        use_resource(get_quotes).suspend()?;

    info!("Rendering Quotes component");

    rsx! {
        div { class: "quotes-container",
            match &*quotes.read() {
                Ok(qs) => rsx! {
                    for q in qs {
                        QuoteDisplay { quote: q.clone() }
                    }
                },
                Err(_) => rsx! { "No quotes found" },
            }
        }
    }
}

#[component]
fn QuoteDisplay(quote: Quote) -> Element {
    rsx! {
        div { class: "quote",
            p { class: "content", {quote.text} }
            p { class: "author", {quote.author} }
            code { class: "time", {quote.created.format("%d.%m.%Y").to_string()} }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Quote {
    text: String,
    author: String,
    created: DateTime<Local>,
}

fn file_path() -> String {
    std::env::var("QUOTE_FILE").unwrap()
}

#[server]
async fn get_quotes() -> Result<Vec<Quote>, ServerFnError> {
    let s = std::fs::read_to_string(file_path())
        .map_err(|e| eprintln!("{e:?}"))
        .unwrap_or("".to_string());
    let quotes: Vec<Quote> = serde_json::from_str(&s)
        .map_err(|e| eprintln!("{e:?}"))
        .unwrap_or(vec![]);

    // Add this debug log
    info!("Loaded quotes: {:?}", quotes);

    Ok(quotes)
}

#[server(AddQuoteFn)]
async fn add_quote(quote: String, author: String) -> Result<bool, ServerFnError> {
    let q = Quote {
        text: quote,
        author,
        created: Local::now(),
    };

    info!("Adding");

    let s = std::fs::read_to_string(file_path())
        .map_err(|e| error!("{e:?}"))
        .ok();
    if let Some(s) = s {
        let quotes: Option<Vec<Quote>> = serde_json::from_str(&s).map_err(|e| error!("{e:?}")).ok();
        if let Some(mut quotes) = quotes {
            quotes.push(q);
            let s = serde_json::to_string(&quotes)
                .map_err(|e| error!("{e:?}"))
                .ok();

            if let Some(s) = s {
                let success = std::fs::write(file_path(), s)
                    .map_err(|e| error!("{e:?}"))
                    .is_ok();

                if success {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}
