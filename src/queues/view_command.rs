use std::sync::{mpsc, Arc, Mutex, OnceLock};

use crate::ui::viewer::{ViewerCommand, ViewerMessage};

/// 채널 쌍을 담을 구조체
pub struct ViewerChannels {
    pub tx_command: mpsc::Sender<ViewerCommand>,
    pub rx_command: Arc<Mutex<mpsc::Receiver<ViewerCommand>>>,
    pub tx_message: mpsc::Sender<ViewerMessage>,
    pub rx_message: Arc<Mutex<mpsc::Receiver<ViewerMessage>>>,
}

static VIEWER_CHANNELS: OnceLock<ViewerChannels> = OnceLock::new();

/// 싱글톤 채널 접근 함수
pub fn get_viewer_channels() -> &'static ViewerChannels {
    VIEWER_CHANNELS.get_or_init(|| {
        let (tx_command, rx_command) = mpsc::channel();
        let (tx_message, rx_message) = mpsc::channel();
        ViewerChannels {
            tx_command,
            rx_command: Arc::new(Mutex::new(rx_command)),
            tx_message,
            rx_message: Arc::new(Mutex::new(rx_message)),
        }
    })
}