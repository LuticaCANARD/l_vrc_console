use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sysinfo::System;

use crate::ui::components::usage_gauge::CoreGraph;

/// CPU 멀티코어 모니터 뷰
pub struct CpuCoresView {
    system: System,
    cores: Vec<CoreGraph>,
    show_graph: bool, // true: 그래프, false: 게이지
}

impl CpuCoresView {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let core_count = system.cpus().len();
        let cores = (0..core_count)
            .map(|i| CoreGraph::new(format!("Core {}", i)))
            .collect();

        Self {
            system,
            cores,
            show_graph: false,
        }
    }

    fn refresh(&mut self) {
        self.system.refresh_cpu_all();

        for (i, cpu) in self.system.cpus().iter().enumerate() {
            if let Some(core) = self.cores.get_mut(i) {
                core.push(cpu.cpu_usage() as f64);
            }
        }
    }

    /// 게이지 모드로 렌더링
    fn render_gauges(&self, frame: &mut Frame, area: Rect) {
        let core_count = self.cores.len();
        if core_count == 0 {
            return;
        }

        // 코어 수에 따라 레이아웃 결정
        let rows = (core_count as f32 / 4.0).ceil() as usize;
        let cols = 4.min(core_count);

        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Length(3))
            .chain(std::iter::once(Constraint::Min(0)))
            .collect();

        let row_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        for row in 0..rows {
            let col_constraints: Vec<Constraint> =
                (0..cols).map(|_| Constraint::Ratio(1, cols as u32)).collect();

            let col_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(row_chunks[row]);

            for col in 0..cols {
                let idx = row * cols + col;
                if let Some(core) = self.cores.get(idx) {
                    core.render_gauge(frame, col_chunks[col]);
                }
            }
        }
    }

    /// 그래프 모드로 렌더링
    fn render_graphs(&self, frame: &mut Frame, area: Rect) {
        let core_count = self.cores.len();
        if core_count == 0 {
            return;
        }

        // 2열 레이아웃
        let cols = 2;
        let rows = (core_count as f32 / cols as f32).ceil() as usize;

        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Ratio(1, rows as u32))
            .collect();

        let row_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        for row in 0..rows {
            let col_constraints: Vec<Constraint> =
                (0..cols).map(|_| Constraint::Ratio(1, cols as u32)).collect();

            let col_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(row_chunks[row]);

            for col in 0..cols {
                let idx = row * cols + col;
                if let Some(core) = self.cores.get(idx) {
                    core.render_graph(frame, col_chunks[col]);
                }
            }
        }
    }

    pub fn toggle_mode(&mut self) {
        self.show_graph = !self.show_graph;
    }
}

impl Default for CpuCoresView {
    fn default() -> Self {
        Self::new()
    }
}

impl super::ViewComponent for CpuCoresView {
    fn draw_with_area(&self, frame: &mut Frame, area: Rect) {
        // 전체 레이아웃: 타이틀 + 코어들
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // 타이틀
        let mode = if self.show_graph { "Graph" } else { "Gauge" };
        let title = Paragraph::new(format!(
            "CPU Cores Monitor ({} cores) [G: toggle mode - {}] [Tab: switch view]",
            self.cores.len(),
            mode
        ))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, main_chunks[0]);

        // 모드에 따라 렌더링
        if self.show_graph {
            self.render_graphs(frame, main_chunks[1]);
        } else {
            self.render_gauges(frame, main_chunks[1]);
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('g') | KeyCode::Char('G') => {
                self.toggle_mode();
                true // 이벤트 소비됨
            }
            _ => false,
        }
    }
}

impl super::TickingComponent for CpuCoresView {
    fn on_tick(&mut self) {
        self.refresh();
    }
}
