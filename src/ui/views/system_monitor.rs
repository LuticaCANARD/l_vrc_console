use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sysinfo::System;

use crate::ui::components::usage_gauge::{CpuGraph, GpuGraph, MemoryGraph};

/// 시스템 모니터 뷰 - CPU, GPU, Memory 사용량 그래프 표시
pub struct SystemMonitorView {
    system: System,
    cpu_graph: CpuGraph,
    gpu_graph: GpuGraph,
    memory_graph: MemoryGraph,
    nvml: Option<nvml_wrapper::Nvml>,
}

impl SystemMonitorView {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        // NVML 초기화 (NVIDIA GPU용)
        let nvml = nvml_wrapper::Nvml::init().ok();

        Self {
            system,
            cpu_graph: CpuGraph::new(),
            gpu_graph: GpuGraph::new(),
            memory_graph: MemoryGraph::new(),
            nvml,
        }
    }

    /// 시스템 정보 갱신
    fn refresh(&mut self) {
        self.system.refresh_all();

        // CPU 사용량 업데이트
        let cpu_usage = self.system.global_cpu_usage() as f64;
        self.cpu_graph.push(cpu_usage);

        // 메모리 사용량 업데이트
        let used_memory = self.system.used_memory();
        let total_memory = self.system.total_memory();
        self.memory_graph.push(used_memory, total_memory);

        // GPU 사용량 업데이트 (NVIDIA)
        if let Some(ref nvml) = self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(utilization) = device.utilization_rates() {
                    self.gpu_graph.push(utilization.gpu as f64);
                }
                if let Ok(memory_info) = device.memory_info() {
                    let vram_percent =
                        (memory_info.used as f64 / memory_info.total as f64) * 100.0;
                    self.gpu_graph.push_vram(vram_percent);
                }
            }
        }
    }
}

impl Default for SystemMonitorView {
    fn default() -> Self {
        Self::new()
    }
}

impl super::ViewComponent for SystemMonitorView {
    fn draw_with_area(&self, frame: &mut Frame, area: Rect) {
        // 전체 레이아웃: 타이틀 + 그래프들
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // 타이틀
        let title = Paragraph::new("System Monitor (Tab to switch view)")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, main_chunks[0]);

        // 그래프들 레이아웃 (2x2 그리드)
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[1]);

        let top_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let bottom_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        // 각 그래프 렌더링
        self.cpu_graph.render(frame, top_row[0]);
        self.memory_graph.render(frame, top_row[1]);
        self.gpu_graph.render(frame, bottom_row[0]);
        self.gpu_graph.render_vram(frame, bottom_row[1]);
    }
}

impl super::TickingComponent for SystemMonitorView {
    fn on_tick(&mut self) {
        self.refresh();
    }
}
