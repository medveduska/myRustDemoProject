mod app;
mod backup_io;
mod components;
mod csv_io;
mod model;
mod storage;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
