use web_sys::MouseEvent;
use yew::prelude::*;

use crate::model::{StudyDirection, StudyMode};

#[derive(Properties, PartialEq)]
pub struct StudyToolbarProps {
    pub direction: StudyDirection,
    pub mode: StudyMode,
    pub on_toggle_mode: Callback<MouseEvent>,
    pub on_toggle_direction: Callback<MouseEvent>,
    pub on_randomize: Callback<MouseEvent>,
    pub on_open_add: Callback<MouseEvent>,
}

#[function_component(StudyToolbar)]
pub fn study_toolbar(props: &StudyToolbarProps) -> Html {
    html! {
        <section class="toolbar panel">
            <h3 class="panel-title">{"Study Controls"}</h3>
            <div class="toolbar-actions">
            <button class="btn btn-secondary" onclick={props.on_toggle_mode.clone()}>
                {
                    match props.mode {
                        StudyMode::Reveal => "Switch to Exercise Mode",
                        StudyMode::Exercise => "Switch to Reveal Mode",
                    }
                }
            </button>
            <button class="btn btn-secondary" onclick={props.on_toggle_direction.clone()}>
                {
                    match props.direction {
                        StudyDirection::Normal => "Switch to Translation -> Pinyin -> Character",
                        StudyDirection::Reverse => "Switch to Character -> Pinyin -> Translation",
                    }
                }
            </button>
            <button class="btn btn-primary" onclick={props.on_randomize.clone()}>
                {"Randomize"}
            </button>
            <button class="btn btn-primary" onclick={props.on_open_add.clone()}>{"Add New Flashcard"}</button>
            </div>
        </section>
    }
}
