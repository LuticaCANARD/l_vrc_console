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
    layout::Rect,
    Frame, Terminal,
};

use super::views::{
    cpu_cores::CpuCoresView,
    status::StatusView,
    system_monitor::SystemMonitorView,
    TickingComponent, ViewComponent,
};

/// 앱 상태를 관리하는 구조체
pub struct App {
    /// 현재 활성화된 뷰 인덱스
    current_view: usize,
    /// tick이 필요한 뷰들
    ticking_views: Vec<Box<dyn TickingViewTrait>>,
    /// 종료 플래그
    should_quit: bool,
    /// 화면 클리어 필요 플래그
    needs_clear: bool,
}

/// ViewComponent + TickingComponent를 함께 처리하기 위한 trait
trait TickingViewTrait {
    fn draw_with_area(&self, frame: &mut Frame, area: Rect);
    fn on_tick(&mut self);
    fn handle_key(&mut self, key: KeyCode) -> bool;
    fn needs_tick(&self) -> bool;
}

/// TickingViewTrait 구현체 (tick 있는 뷰)
struct TickingViewHolder<T: ViewComponent + TickingComponent> {
    inner: T,
}

impl<T: ViewComponent + TickingComponent> TickingViewTrait for TickingViewHolder<T> {
    fn draw_with_area(&self, frame: &mut Frame, area: Rect) {
        self.inner.draw_with_area(frame, area);
    }
    fn on_tick(&mut self) {
        self.inner.on_tick();
    }
    fn handle_key(&mut self, key: KeyCode) -> bool {
        self.inner.handle_key(key)
    }
    fn needs_tick(&self) -> bool {
        true
    }
}

/// ViewHolder (tick 없는 뷰)
struct ViewHolder<T: ViewComponent> {
    inner: T,
}

impl<T: ViewComponent> TickingViewTrait for ViewHolder<T> {
    fn draw_with_area(&self, frame: &mut Frame, area: Rect) {
        self.inner.draw_with_area(frame, area);
    }
    fn on_tick(&mut self) {
        // tick 불필요
    }
    fn handle_key(&mut self, key: KeyCode) -> bool {
        self.inner.handle_key(key)
    }
    fn needs_tick(&self) -> bool {
        false
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = App {
            current_view: 0,
            ticking_views: Vec::new(),
            should_quit: false,
            needs_clear: true,
        };

        // 기본 뷰 등록
        app.register_ticking_view(StatusView::new());
        app.register_ticking_view(SystemMonitorView::new());
        app.register_ticking_view(CpuCoresView::new());

        app
    }

    /// Tick 기능이 있는 뷰 등록
    pub fn register_ticking_view<T: ViewComponent + TickingComponent + 'static>(&mut self, view: T) {
        self.ticking_views.push(Box::new(TickingViewHolder { inner: view }));
    }

    /// Tick 기능이 없는 뷰 등록
    pub fn register_view<T: ViewComponent + 'static>(&mut self, view: T) {
        self.ticking_views.push(Box::new(ViewHolder { inner: view }));
    }

    /// 다음 뷰로 전환
    pub fn next_view(&mut self) {
        if !self.ticking_views.is_empty() {
            self.current_view = (self.current_view + 1) % self.ticking_views.len();
            self.needs_clear = true;
        }
    }

    /// 이전 뷰로 전환
    pub fn prev_view(&mut self) {
        if !self.ticking_views.is_empty() {
            self.current_view = if self.current_view == 0 {
                self.ticking_views.len() - 1
            } else {
                self.current_view - 1
            };
            self.needs_clear = true;
        }
    }

    /// 화면 클리어가 필요한지 확인하고 플래그 리셋
    pub fn take_needs_clear(&mut self) -> bool {
        let result = self.needs_clear;
        self.needs_clear = false;
        result
    }

    /// 현재 뷰 그리기
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        if let Some(view) = self.ticking_views.get(self.current_view) {
            view.draw_with_area(frame, area);
        }
    }

    /// tick 처리 (현재 보이는 뷰만 업데이트)
    pub fn on_tick(&mut self) {
        // 현재 뷰만 tick 처리 (성능 최적화)
        if let Some(view) = self.ticking_views.get_mut(self.current_view) {
            view.on_tick();
        }
    }

    /// 키 입력 처리
    pub fn handle_key(&mut self, key: KeyCode) {
        // 먼저 현재 뷰에 키 이벤트 전달
        if let Some(view) = self.ticking_views.get_mut(self.current_view) {
            if view.handle_key(key) {
                return; // 뷰에서 이벤트를 소비함
            }
        }

        // 전역 키 처리
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab | KeyCode::Right => self.next_view(),
            KeyCode::BackTab | KeyCode::Left => self.prev_view(),
            _ => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

/// 터미널 UI 실행
pub fn show_ui() -> Result<(), io::Error> {
    // 터미널 초기화
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 동적 tick rate 설정
    const MIN_TICK_MS: u64 = 16;   // ~60fps 최대
    const MAX_TICK_MS: u64 = 200;  // 최소 5fps
    const TARGET_FRAME_MS: u64 = 33; // 목표 ~30fps

    let mut tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();
    let mut last_frame_time = Duration::from_millis(0);
    let mut app = App::new();

    // 메인 루프
    loop {
        let frame_start = Instant::now();

        // 뷰 전환 시 화면 클리어
        if app.take_needs_clear() {
            terminal.clear()?;
        }

        // 화면 그리기
        terminal.draw(|frame| {
            app.draw(frame);
        })?;

        // 이벤트 처리
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // 키가 눌렸을 때만 처리 (Release, Repeat 무시)
                if key.kind == event::KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        // tick 처리
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        // 종료 조건
        if app.should_quit() {
            break;
        }

        // 동적 tick rate 조절
        last_frame_time = frame_start.elapsed();
        let frame_ms = last_frame_time.as_millis() as u64;

        if frame_ms > TARGET_FRAME_MS + 10 {
            // 프레임이 느리면 tick rate 증가 (부하 감소)
            let new_tick = tick_rate.as_millis() as u64 + 10;
            tick_rate = Duration::from_millis(new_tick.min(MAX_TICK_MS));
        } else if frame_ms < TARGET_FRAME_MS - 5 {
            // 프레임이 빠르면 tick rate 감소 (반응성 향상)
            let new_tick = tick_rate.as_millis() as u64 - 5;
            tick_rate = Duration::from_millis(new_tick.max(MIN_TICK_MS));
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