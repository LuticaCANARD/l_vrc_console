mod ui;
mod contracts;
mod integration;
mod config;
mod controllers;

fn main() {
    // 터미널 UI 실행
    if let Err(e) = ui::viewer::show_ui() {
        eprintln!("UI 오류: {}", e);
    }
}
