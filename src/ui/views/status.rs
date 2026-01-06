use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use sysinfo::System;

pub struct StatusView {
    system: System,
    os_name: String,
    os_version: String,
    kernel_version: String,
    host_name: String,
    cpu_name: String,
    cpu_cores: usize,
    total_memory_gb: f64,
}

impl StatusView {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let host_name = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        
        let cpu_name = system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let cpu_cores = system.cpus().len();
        let total_memory_gb = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

        StatusView {
            system,
            os_name,
            os_version,
            kernel_version,
            host_name,
            cpu_name,
            cpu_cores,
            total_memory_gb,
        }
    }
}

impl super::ViewComponent for StatusView {
    fn draw_with_area(&self, frame: &mut ratatui::Frame, area: Rect) {
        let block = Block::default()
            .title(" System Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::vertical([
            Constraint::Length(3),  // Title
            Constraint::Length(8),  // OS Info Table
            Constraint::Length(5),  // Hardware Info Table
            Constraint::Min(1),     // 나머지 공간
        ])
        .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled("🖥️  ", Style::default().fg(Color::Yellow)),
            Span::styled(&self.host_name, Style::default().fg(Color::White).bold()),
        ]))
        .centered();
        frame.render_widget(title, chunks[0]);

        // OS Info Table
        let os_rows = vec![
            Row::new(vec![
                Span::styled("Operating System", Style::default().fg(Color::Gray)),
                Span::styled(&self.os_name, Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Span::styled("OS Version", Style::default().fg(Color::Gray)),
                Span::styled(&self.os_version, Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Span::styled("Kernel Version", Style::default().fg(Color::Gray)),
                Span::styled(&self.kernel_version, Style::default().fg(Color::White)),
            ]),
        ];

        let os_table = Table::new(
            os_rows,
            [Constraint::Length(20), Constraint::Fill(1)],
        )
        .block(
            Block::default()
                .title(" 📋 OS Information ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );
        frame.render_widget(os_table, chunks[1]);

        // Hardware Info Table
        let hw_rows = vec![
            Row::new(vec![
                Span::styled("CPU", Style::default().fg(Color::Gray)),
                Span::styled(&self.cpu_name, Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Span::styled("CPU Cores", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} cores", self.cpu_cores),
                    Style::default().fg(Color::White),
                ),
            ]),
            Row::new(vec![
                Span::styled("Total Memory", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1} GB", self.total_memory_gb),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
        ];

        let hw_table = Table::new(
            hw_rows,
            [Constraint::Length(20), Constraint::Fill(1)],
        )
        .block(
            Block::default()
                .title(" 🔧 Hardware Information ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );
        frame.render_widget(hw_table, chunks[2]);
    }
}

impl super::TickingComponent for StatusView {
    fn on_tick(&mut self) {
        // 정적 정보만 표시하므로 tick 불필요
    }
}
