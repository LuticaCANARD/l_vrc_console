

pub struct VrchatPageView {
    // VRChat 관련 데이터 및 상태를 여기에 추가
}

impl VrchatPageView {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::ViewComponent for VrchatPageView {
    fn draw_with_area(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let block = ratatui::widgets::Block::default()
            .title(" VRChat Page ")
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Magenta));

        frame.render_widget(block, area);

        // VRChat 페이지의 추가 UI 요소를 여기에 그리기
    }
}

impl super::TickingComponent for VrchatPageView {
    fn on_tick(&mut self) {
        
    }
}

