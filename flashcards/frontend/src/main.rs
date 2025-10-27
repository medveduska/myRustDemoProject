use yew::prelude::*;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use reqwasm::http::Request;

#[derive(Deserialize, Clone, PartialEq)]
struct Flashcard {
    word: String,
    pinyin: Option<String>,
    translation: String,
}

#[function_component(App)]
fn app() -> Html {
    let flashcards = use_state(|| vec![]);
    let load_cards = {
        let flashcards = flashcards.clone();
        Callback::from(move |_| {
            let flashcards = flashcards.clone();
            spawn_local(async move {
            let resp = Request::get("http://127.0.0.1:8080/api/import")
                .send()
                .await
                .unwrap()
                .json::<Vec<Flashcard>>()
                .await
                .unwrap();
                flashcards.set(resp);
            });
        })
    };

    html! {
        <div style="font-family: sans-serif; text-align: center;">
            <h1>{"Language Flashcards 🈶"}</h1>
            <button onclick={load_cards}>{"Load Flashcards"}</button>
            <ul>
                { for flashcards.iter().map(|c| html! {
                    <li>
                        <b>{ &c.word }</b>{" "}
                        { c.pinyin.as_ref().map(|p| html!{<i>{format!("({}) ", p)}</i>}).unwrap_or_default() }
                        {"– "}{ &c.translation }
                    </li>
                })}
            </ul>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
