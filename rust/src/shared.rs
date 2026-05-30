use servo::EventLoopWaker;

pub enum SignalEvent {
    UrlChanged(String),
    PageTitleChanged(Option<String>),
    LoadStatusChanged(String),
    CursorChanged(String),
    NewFrameReady,
    HistoryChanged(Vec<String>, usize),
    FocusChanged(bool),
    StatusTextChanged(Option<String>),
    FaviconChanged,
    Closed,
    Crashed(String),
    FullscreenStateChanged(bool),
    ConsoleMessage(String, String),
    NavigationRequested(String),
    PermissionRequested(String),
    AuthenticationRequested(String),
}

pub struct SharedState {
    pub events: Vec<SignalEvent>,
    pub frame_ready: bool,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            frame_ready: false,
        }
    }
}

pub struct BrowserWaker;

impl EventLoopWaker for BrowserWaker {
    fn wake(&self) {}
}
