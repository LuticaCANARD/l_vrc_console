# Viewer 채널 아키텍처

## 개요

`l_vrc_console`은 싱글톤 채널 패턴을 사용하여 외부 모듈과 Viewer(UI) 간의 비동기 통신을 구현합니다.

## 채널 구조

```
┌─────────────────┐                              ┌─────────────────┐
│                 │                              │                 │
│   Main/         │  ── tx_command ────────────▶ │   Viewer        │
│   External      │     (ViewerCommand)          │   (UI)          │
│   Modules       │                              │                 │
│                 │  ◀── tx_message ──────────── │                 │
│                 │     (ViewerMessage)          │                 │
└─────────────────┘                              └─────────────────┘
```

## 채널 구성요소

### ViewerChannels 구조체

```rust
pub struct ViewerChannels {
    pub tx_command: mpsc::Sender<ViewerCommand>,
    pub rx_command: Arc<Mutex<mpsc::Receiver<ViewerCommand>>>,
    pub tx_message: mpsc::Sender<ViewerMessage>,
    pub rx_message: Arc<Mutex<mpsc::Receiver<ViewerMessage>>>,
}
```

### 채널 방향

| 채널 | 방향 | 용도 |
|------|------|------|
| `tx_command` | 외부 → Viewer | 외부에서 Viewer로 명령 전송 |
| `rx_command` | Viewer가 수신 | Viewer가 외부 명령을 수신 |
| `tx_message` | Viewer → 외부 | Viewer에서 외부로 메시지 전송 |
| `rx_message` | 외부가 수신 | 외부에서 Viewer 메시지를 수신 |

## 메시지 타입

### ViewerCommand (외부 → Viewer)

```rust
pub enum ViewerCommand {
    Quit,
    // 추후 명령어 추가 가능
}
```

### ViewerMessage (Viewer → 외부)

```rust
pub enum ViewerMessage {
    Quit,
    // 추후 메시지 추가 가능
}
```

## 사용 방법

### 싱글톤 채널 접근

```rust
let channels = queues::view_command::get_viewer_channels();
```

### 외부에서 Viewer로 명령 보내기

```rust
let channels = queues::view_command::get_viewer_channels();
channels.tx_command.send(ViewerCommand::Quit).unwrap();
```

### Viewer에서 명령 수신하기

```rust
let channels = crate::queues::view_command::get_viewer_channels();
let rx = channels.rx_command.lock().unwrap();
if let Ok(command) = rx.try_recv() {
    match command {
        ViewerCommand::Quit => {
            // 종료 처리
        }
    }
}
```

### Viewer에서 외부로 메시지 보내기

```rust
let channels = crate::queues::view_command::get_viewer_channels();
channels.tx_message.send(ViewerMessage::Quit).unwrap();
```

### 외부에서 메시지 수신하기

```rust
let channels = queues::view_command::get_viewer_channels();
let rx = channels.rx_message.lock().unwrap();
if let Ok(message) = rx.try_recv() {
    match message {
        ViewerMessage::Quit => {
            // 종료 처리
        }
    }
}
```

## 싱글톤 패턴

채널은 `OnceLock`을 사용하여 프로그램 전체에서 단일 인스턴스를 공유합니다.

```rust
static VIEWER_CHANNELS: OnceLock<ViewerChannels> = OnceLock::new();

pub fn get_viewer_channels() -> &'static ViewerChannels {
    VIEWER_CHANNELS.get_or_init(|| {
        // 채널 초기화
    })
}
```

### 장점

- 어디서든 동일한 채널 인스턴스에 접근 가능
- 스레드 안전 (thread-safe)
- 지연 초기화 (lazy initialization)

## 주의사항

1. **Mutex 잠금**: `rx_command`와 `rx_message`는 `Arc<Mutex<...>>`로 감싸져 있으므로 접근 시 `.lock().unwrap()` 필요
2. **Sender Clone**: `tx_command`와 `tx_message`는 필요시 `.clone()` 가능
3. **논블로킹**: `try_recv()` 사용으로 블로킹 없이 메시지 확인

## 파일 위치

- 채널 정의: `src/queues/view_command.rs`
- 메시지 타입: `src/ui/viewer.rs`
