use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame, Terminal,
};

struct App {
    /// 시간에 따라 변하는 데이터 (사인파)
    data1: Vec<(f64, f64)>,
    /// 두 번째 데이터 (코사인파)
    data2: Vec<(f64, f64)>,
    /// x축 윈도우 범위
    window: [f64; 2],
    /// 시작 시간
    start_time: Instant,
}

impl App {
    fn new() -> App {
        App {
            data1: Vec::new(),
            data2: Vec::new(),
            window: [0.0, 20.0],
            start_time: Instant::now(),
        }
    }

    /// 매 프레임마다 데이터 업데이트
    fn on_tick(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        
        // 새로운 데이터 포인트 추가 (사인파)
        let y1 = (elapsed * 2.0).sin();
        self.data1.push((elapsed, y1));
        
        // 두 번째 데이터 (코사인파)
        let y2 = (elapsed * 2.0).cos();
        self.data2.push((elapsed, y2));
        
        // 윈도우 슬라이딩 - 20초 범위 유지
        if elapsed > 20.0 {
            self.window[0] = elapsed - 20.0;
            self.window[1] = elapsed;
            
            // 오래된 데이터 제거 (메모리 절약)
            self.data1.retain(|(x, _)| *x >= self.window[0] - 1.0);
            self.data2.retain(|(x, _)| *x >= self.window[0] - 1.0);
        }
    }
}

fn main() -> Result<(), io::Error> {
    // 터미널 설정
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 앱 생성
    let mut app = App::new();
    let tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();

    loop {
        // 화면 렌더링
        terminal.draw(|f| ui(f, &app))?;

        // 이벤트 처리
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        // 주기적 업데이트
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }

    // 터미널 복원
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20),
        ])
        .split(f.area());

    // 메인 차트 렌더링
    render_chart(f, app, chunks[0]);
    
    // 도움말 영역
    render_help(f, chunks[1]);
}

fn render_chart(f: &mut Frame, app: &App, area: Rect) {
    // 데이터셋 생성
    let datasets = vec![
        Dataset::default()
            .name("Sin(x)")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&app.data1),
        Dataset::default()
            .name("Cos(x)")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&app.data2),
    ];

    // x축 레이블 생성
    let x_labels = vec![
        Span::styled(
            format!("{:.1}", app.window[0]),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{:.1}", (app.window[0] + app.window[1]) / 2.0)),
        Span::styled(
            format!("{:.1}", app.window[1]),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];

    // 차트 위젯 생성
    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" 📊 실시간 그래프 (Ratatui Chart Example) ")
                .borders(Borders::ALL)
                .style(Style::default()),
        )
        .x_axis(
            Axis::default()
                .title("시간 (초)")
                .style(Style::default().fg(Color::Gray))
                .bounds(app.window)
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("값")
                .style(Style::default().fg(Color::Gray))
                .bounds([-1.5, 1.5])
                .labels(vec![
                    Span::styled("-1.5", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("0"),
                    Span::styled("1.5", Style::default().add_modifier(Modifier::BOLD)),
                ]),
        );

    f.render_widget(chart, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_block = Block::default()
        .title(" 도움말 ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let help_text = ratatui::widgets::Paragraph::new(
        "q 또는 ESC: 종료 | 이 예제는 Sin과 Cos 함수를 실시간으로 그래프에 표시합니다."
    )
    .style(Style::default().fg(Color::Green))
    .block(help_block);

    f.render_widget(help_text, area);
}
