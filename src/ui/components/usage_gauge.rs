use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType},
    Frame,
};

const HISTORY_SIZE: usize = 60; // 60개 데이터 포인트 (약 3초 @ 50ms tick)

/// 사용량을 표시하는 게이지 컴포넌트
pub struct UsageGauge {
    title: String,
    usage_percent: f64,
    color: Color,
}

impl UsageGauge {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            usage_percent: 0.0,
            color: Color::Green,
        }
    }

    /// 사용량 업데이트 (0.0 ~ 100.0)
    pub fn set_usage(&mut self, percent: f64) {
        self.usage_percent = percent.clamp(0.0, 100.0);
        // 사용량에 따라 색상 변경
        self.color = match self.usage_percent as u32 {
            0..=50 => Color::Green,
            51..=75 => Color::Yellow,
            _ => Color::Red,
        };
    }

    pub fn get_usage(&self) -> f64 {
        self.usage_percent
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    /// 컴포넌트 렌더링
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(self.title.clone())
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(self.color))
            .percent(self.usage_percent as u16)
            .label(format!("{:.1}%", self.usage_percent));

        frame.render_widget(gauge, area);
    }
}

/// 시계열 그래프 컴포넌트
pub struct UsageGraph {
    title: String,
    history: Vec<f64>,
    color: Color,
    initialized: bool,
}

impl UsageGraph {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            history: vec![0.0; HISTORY_SIZE],
            color: Color::Green,
            initialized: false,
        }
    }

    /// 새 데이터 추가 (0.0 ~ 100.0)
    pub fn push(&mut self, percent: f64) {
        let clamped = percent.clamp(0.0, 100.0);
        
        // 첫 데이터가 들어오면 히스토리 전체를 현재 값으로 초기화
        if !self.initialized {
            self.history = vec![clamped; HISTORY_SIZE];
            self.initialized = true;
        } else {
            self.history.remove(0);
            self.history.push(clamped);
        }

        // 최신 값에 따라 색상 변경
        self.color = match clamped as u32 {
            0..=50 => Color::Green,
            51..=75 => Color::Yellow,
            _ => Color::Red,
        };
    }

    pub fn get_current(&self) -> f64 {
        *self.history.last().unwrap_or(&0.0)
    }

    /// 그래프 렌더링
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // 데이터를 (x, y) 형태로 변환
        let data: Vec<(f64, f64)> = self
            .history
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect();

        let datasets = vec![Dataset::default()
            .name(format!("{:.1}%", self.get_current()))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(self.color))
            .data(&data)];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        self.title.clone(),
                        Style::default().fg(Color::Cyan).bold(),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .bounds([0.0, HISTORY_SIZE as f64]),
            )
            .y_axis(
                Axis::default()
                    .bounds([0.0, 100.0])
                    .labels(vec![
                        Span::raw("0"),
                        Span::raw("50"),
                        Span::raw("100"),
                    ]),
            );

        frame.render_widget(chart, area);
    }
}

/// CPU 그래프 (게이지 + 그래프 통합)
pub struct CpuGraph {
    graph: UsageGraph,
}

impl CpuGraph {
    pub fn new() -> Self {
        Self {
            graph: UsageGraph::new("CPU"),
        }
    }

    pub fn push(&mut self, percent: f64) {
        self.graph.push(percent);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.graph.render(frame, area);
    }
}

impl Default for CpuGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU 그래프
pub struct GpuGraph {
    graph: UsageGraph,
    vram_graph: UsageGraph,
}

impl GpuGraph {
    pub fn new() -> Self {
        Self {
            graph: UsageGraph::new("GPU"),
            vram_graph: UsageGraph::new("VRAM"),
        }
    }

    pub fn push(&mut self, percent: f64) {
        self.graph.push(percent);
    }

    pub fn push_vram(&mut self, percent: f64) {
        self.vram_graph.push(percent);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.graph.render(frame, area);
    }

    pub fn render_vram(&self, frame: &mut Frame, area: Rect) {
        self.vram_graph.render(frame, area);
    }
}

impl Default for GpuGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// 메모리 그래프
pub struct MemoryGraph {
    graph: UsageGraph,
    used_gb: f64,
    total_gb: f64,
}

impl MemoryGraph {
    pub fn new() -> Self {
        Self {
            graph: UsageGraph::new("Memory"),
            used_gb: 0.0,
            total_gb: 0.0,
        }
    }

    pub fn push(&mut self, used_bytes: u64, total_bytes: u64) {
        self.used_gb = used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        self.total_gb = total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let percent = if total_bytes > 0 {
            (used_bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };
        self.graph.push(percent);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // 데이터를 (x, y) 형태로 변환
        let data: Vec<(f64, f64)> = self
            .graph
            .history
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect();

        let current = self.graph.get_current();
        let color = match current as u32 {
            0..=50 => Color::Green,
            51..=75 => Color::Yellow,
            _ => Color::Red,
        };

        let datasets = vec![Dataset::default()
            .name(format!("{:.1}GB / {:.1}GB ({:.1}%)", self.used_gb, self.total_gb, current))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(&data)];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Memory",
                        Style::default().fg(Color::Cyan).bold(),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .bounds([0.0, HISTORY_SIZE as f64]),
            )
            .y_axis(
                Axis::default()
                    .bounds([0.0, 100.0])
                    .labels(vec![
                        Span::raw("0"),
                        Span::raw("50"),
                        Span::raw("100"),
                    ]),
            );

        frame.render_widget(chart, area);
    }
}

impl Default for MemoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// 개별 코어/항목용 그래프 컴포넌트 (재사용 가능)
pub struct CoreGraph {
    title: String,
    history: Vec<f64>,
}

impl CoreGraph {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            history: vec![0.0; HISTORY_SIZE],
        }
    }

    /// 새 데이터 추가 (0.0 ~ 100.0)
    pub fn push(&mut self, percent: f64) {
        self.history.remove(0);
        self.history.push(percent.clamp(0.0, 100.0));
    }

    pub fn current(&self) -> f64 {
        *self.history.last().unwrap_or(&0.0)
    }

    pub fn color(&self) -> Color {
        match self.current() as u32 {
            0..=50 => Color::Green,
            51..=75 => Color::Yellow,
            _ => Color::Red,
        }
    }

    /// 게이지 모드로 렌더링
    pub fn render_gauge(&self, frame: &mut Frame, area: Rect) {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(self.title.clone())
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(self.color()))
            .percent(self.current() as u16)
            .label(format!("{:.1}%", self.current()));

        frame.render_widget(gauge, area);
    }

    /// 그래프 모드로 렌더링
    pub fn render_graph(&self, frame: &mut Frame, area: Rect) {
        let data: Vec<(f64, f64)> = self
            .history
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v))
            .collect();

        let datasets = vec![Dataset::default()
            .name(format!("{:.1}%", self.current()))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(self.color()))
            .data(&data)];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        self.title.clone(),
                        Style::default().fg(Color::Cyan),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(Axis::default().bounds([0.0, HISTORY_SIZE as f64]))
            .y_axis(
                Axis::default()
                    .bounds([0.0, 100.0])
                    .labels(vec![Span::raw("0"), Span::raw("50"), Span::raw("100")]),
            );

        frame.render_widget(chart, area);
    }
}

impl Default for CoreGraph {
    fn default() -> Self {
        Self::new("Core")
    }
}

/// CPU 사용량 전용 게이지
pub struct CpuGauge {
    gauge: UsageGauge,
}

impl CpuGauge {
    pub fn new() -> Self {
        Self {
            gauge: UsageGauge::new("CPU"),
        }
    }

    pub fn set_usage(&mut self, percent: f64) {
        self.gauge.set_usage(percent);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.gauge.render(frame, area);
    }
}

impl Default for CpuGauge {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU 사용량 전용 게이지
pub struct GpuGauge {
    gauge: UsageGauge,
    vram_gauge: UsageGauge,
}

impl GpuGauge {
    pub fn new() -> Self {
        Self {
            gauge: UsageGauge::new("GPU"),
            vram_gauge: UsageGauge::new("VRAM"),
        }
    }

    pub fn set_usage(&mut self, percent: f64) {
        self.gauge.set_usage(percent);
    }

    pub fn set_vram_usage(&mut self, percent: f64) {
        self.vram_gauge.set_usage(percent);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.gauge.render(frame, area);
    }

    pub fn render_vram(&self, frame: &mut Frame, area: Rect) {
        self.vram_gauge.render(frame, area);
    }
}

impl Default for GpuGauge {
    fn default() -> Self {
        Self::new()
    }
}

/// 메모리 사용량 게이지
pub struct MemoryGauge {
    gauge: UsageGauge,
    used_gb: f64,
    total_gb: f64,
}

impl MemoryGauge {
    pub fn new() -> Self {
        Self {
            gauge: UsageGauge::new("Memory"),
            used_gb: 0.0,
            total_gb: 0.0,
        }
    }

    pub fn set_usage(&mut self, used_bytes: u64, total_bytes: u64) {
        self.used_gb = used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        self.total_gb = total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let percent = if total_bytes > 0 {
            (used_bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };
        self.gauge.set_usage(percent);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(format!(
                        "Memory ({:.1} / {:.1} GB)",
                        self.used_gb, self.total_gb
                    ))
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(match self.gauge.get_usage() as u32 {
                0..=50 => Color::Green,
                51..=75 => Color::Yellow,
                _ => Color::Red,
            }))
            .percent(self.gauge.get_usage() as u16)
            .label(format!("{:.1}%", self.gauge.get_usage()));

        frame.render_widget(gauge, area);
    }
}

impl Default for MemoryGauge {
    fn default() -> Self {
        Self::new()
    }
}
