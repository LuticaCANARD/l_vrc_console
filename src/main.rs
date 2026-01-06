mod ui;
mod contracts;
mod integration;
mod config;
mod controllers;
mod queues;

fn main() {
    // 터미널 UI 실행
    let order_channel = queues::view_command::get_viewer_channels();
    if let Err(e) = ui::viewer::show_ui(
        order_channel.tx_command.clone(),
    ) {
        eprintln!("UI 오류: {}", e);
    }
}
