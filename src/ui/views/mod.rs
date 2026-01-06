use ratatui::{crossterm::event::KeyCode, Frame};

/// 뷰 컴포넌트를 위한 trait - 구현체에서 draw를 반드시 구현해야 함
pub trait ViewComponent {
    fn draw_with_area(&self, frame: &mut Frame, area: ratatui::layout::Rect);
    
    /// 키 입력 처리 (Optional) - true 반환 시 이벤트 소비됨
    fn handle_key(&mut self, _key: KeyCode) -> bool {
        false
    }
}

/// Tick 기반 업데이트가 필요한 컴포넌트용 trait
pub trait TickingComponent {
    fn on_tick(&mut self) {}
}

/// ViewComponent + TickingComponent를 둘 다 구현하는 뷰용 trait
pub trait TickingView: ViewComponent + TickingComponent {}
impl<T: ViewComponent + TickingComponent> TickingView for T {}

pub mod status;
pub mod system_monitor;
pub mod cpu_cores;