use servo::EventLoopWaker;
use servo::webview_delegate::{
    ColorPicker, ContextMenu, EmbedderControl, FilePicker, SelectElement, SimpleDialog,
};

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
    // Dialogs
    DialogAlert(String),
    DialogConfirm(String),
    DialogPrompt(String, String),
    // File picker
    FilePickerRequest(Vec<String>, bool),
    // Select element
    SelectElementRequest(Vec<String>),
    // Color picker
    ColorPickerRequest(Option<u32>),
    // Context menu
    ContextMenuRequest(i32, i32, Vec<String>),
}

pub enum PendingControl {
    Alert(servo::webview_delegate::AlertDialog),
    Confirm(servo::webview_delegate::ConfirmDialog),
    Prompt(servo::webview_delegate::PromptDialog),
    FilePicker(FilePicker),
    SelectElement(SelectElement),
    ColorPicker(ColorPicker),
    ContextMenu(ContextMenu),
}

pub struct SharedState {
    pub events: Vec<SignalEvent>,
    pub frame_ready: bool,
    pub pending_control: Option<PendingControl>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            frame_ready: false,
            pending_control: None,
        }
    }
}

pub struct BrowserWaker;

impl EventLoopWaker for BrowserWaker {
    fn wake(&self) {}
}
