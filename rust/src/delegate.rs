use std::cell::RefCell;
use std::rc::Rc;

use servo::embedder_traits::resources::ResourceReader;
use servo::*;

use crate::shared::{SharedState, SignalEvent};

pub struct BrowserWebViewDelegate {
    pub state: Rc<RefCell<SharedState>>,
}

impl WebViewDelegate for BrowserWebViewDelegate {
    fn notify_url_changed(&self, _wv: WebView, url: Url) {
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::UrlChanged(url.to_string()));
    }

    fn notify_page_title_changed(&self, _wv: WebView, title: Option<String>) {
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::PageTitleChanged(title));
    }

    fn notify_load_status_changed(&self, _wv: WebView, status: LoadStatus) {
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::LoadStatusChanged(format!("{:?}", status)));
    }

    fn notify_cursor_changed(&self, _wv: WebView, cursor: Cursor) {
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::CursorChanged(format!("{:?}", cursor)));
    }

    fn notify_new_frame_ready(&self, _wv: WebView) {
        let mut s = self.state.borrow_mut();
        s.frame_ready = true;
        s.events.push(SignalEvent::NewFrameReady);
    }

    fn notify_history_changed(&self, _wv: WebView, entries: Vec<Url>, current: usize) {
        self.state.borrow_mut().events.push(SignalEvent::HistoryChanged(
            entries.iter().map(|u| u.to_string()).collect(),
            current,
        ));
    }

    fn notify_focus_changed(&self, _wv: WebView, focused: bool) {
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::FocusChanged(focused));
    }

    fn notify_status_text_changed(&self, _wv: WebView, status: Option<String>) {
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::StatusTextChanged(status));
    }

    fn notify_favicon_changed(&self, _wv: WebView) {
        self.state.borrow_mut().events.push(SignalEvent::FaviconChanged);
    }

    fn notify_closed(&self, _wv: WebView) {
        self.state.borrow_mut().events.push(SignalEvent::Closed);
    }

    fn notify_crashed(&self, _wv: WebView, reason: String, _backtrace: Option<String>) {
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::Crashed(reason));
    }

    fn notify_fullscreen_state_changed(&self, _wv: WebView, fs: bool) {
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::FullscreenStateChanged(fs));
    }

    fn show_console_message(&self, _wv: WebView, level: ConsoleLogLevel, message: String) {
        self.state.borrow_mut().events.push(SignalEvent::ConsoleMessage(
            format!("{:?}", level),
            message,
        ));
    }

    fn request_navigation(&self, _wv: WebView, req: NavigationRequest) {
        let url_str = req.url().map(|u| u.to_string()).unwrap_or_default();
        self.state
            .borrow_mut()
            .events
            .push(SignalEvent::NavigationRequested(url_str));
        req.allow();
    }

    fn request_permission(&self, _wv: WebView, req: PermissionRequest) {
        self.state.borrow_mut().events.push(
            SignalEvent::PermissionRequested(format!("{:?}", req.feature())),
        );
        req.deny();
    }

    fn request_authentication(&self, _wv: WebView, req: AuthenticationRequest) {
        self.state.borrow_mut().events.push(
            SignalEvent::AuthenticationRequested(req.realm().unwrap_or("").to_string()),
        );
    }

    fn notify_media_session_event(&self, _wv: WebView, _event: MediaSessionEvent) {}

    fn notify_traversal_complete(&self, _wv: WebView, _id: TraversalId) {}
    fn notify_input_event_handled(
        &self,
        _wv: WebView,
        _id: InputEventId,
        _result: InputEventResult,
    ) {
    }
    fn screen_geometry(&self, _wv: WebView) -> Option<ScreenGeometry> {
        None
    }
    fn request_unload(&self, _wv: WebView, req: AllowOrDenyRequest) {
        req.allow();
    }
    fn request_move_to(&self, _wv: WebView, _pt: DeviceIntPoint) {}
    fn request_resize_to(&self, _wv: WebView, _sz: DeviceIntSize) {}
    fn request_create_new(&self, _parent: WebView, _req: CreateNewWebViewRequest) {}
    fn request_protocol_handler(
        &self,
        _wv: WebView,
        _reg: ProtocolHandlerRegistration,
        req: AllowOrDenyRequest,
    ) {
        req.deny();
    }
    fn show_bluetooth_device_dialog(&self, _wv: WebView, _req: BluetoothDeviceSelectionRequest) {}
    fn show_embedder_control(&self, _wv: WebView, _ctl: EmbedderControl) {}
    fn hide_embedder_control(&self, _wv: WebView, _id: EmbedderControlId) {}
    fn load_web_resource(&self, _wv: WebView, _load: WebResourceLoad) {}
    fn show_notification(&self, _wv: WebView, _notif: Notification) {}
    fn notify_accessibility_tree_update(&self, _wv: WebView, _upd: TreeUpdate) {}
}

pub struct BrowserServoDelegate {
    pub state: Rc<RefCell<SharedState>>,
}

impl ServoDelegate for BrowserServoDelegate {
    fn notify_error(&self, error: ServoError) {
        godot_error!("Servo error: {:?}", error);
    }
    fn notify_devtools_server_started(&self, _port: u16, _token: String) {}
    fn request_devtools_connection(&self, req: AllowOrDenyRequest) {
        req.deny();
    }
    fn load_web_resource(&self, _load: WebResourceLoad) {}
    fn show_notification(&self, _notif: Notification) {}
    fn show_console_message(&self, _level: ConsoleLogLevel, _message: String) {}
}
