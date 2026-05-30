use std::cell::RefCell;
use std::rc::Rc;

use dpi::PhysicalSize;
use euclid;
use godot::classes::{Image, ImageTexture, Node};
use godot::prelude::*;
use servo::embedder_traits::resources::ResourceReader;
use servo::*;
use url::Url;

use crate::delegate::{BrowserServoDelegate, BrowserWebViewDelegate};
use crate::input_map::map_key;
use crate::shared::{BrowserWaker, SharedState, SignalEvent};

#[derive(GodotClass)]
#[class(base=Node)]
struct WebViewBrowser {
    state: Rc<RefCell<SharedState>>,
    servo: Option<Servo>,
    webview: Option<WebView>,
    render_ctx: Option<Rc<dyn RenderingContext>>,
    texture: Gd<ImageTexture>,
    image: Gd<Image>,
    tex_width: u32,
    tex_height: u32,
    #[var]
    initial_url: GodotString,
    base: Base<Node>,
}

#[godot_api]
impl WebViewBrowser {
    #[signal]
    fn url_changed(url: String);

    #[signal]
    fn page_title_changed(title: String);

    #[signal]
    fn load_status_changed(status: String);

    #[signal]
    fn cursor_changed(cursor: String);

    #[signal]
    fn new_frame_ready();

    #[signal]
    fn history_changed(entries: Array, current: i64);

    #[signal]
    fn focus_changed(focused: bool);

    #[signal]
    fn status_text_changed(status: String);

    #[signal]
    fn favicon_changed();

    #[signal]
    fn webview_closed();

    #[signal]
    fn crashed(reason: String);

    #[signal]
    fn fullscreen_state_changed(fullscreen: bool);

    #[signal]
    fn console_message(level: String, message: String);

    #[signal]
    fn navigation_requested(url: String);

    #[signal]
    fn permission_requested(feature: String);

    #[signal]
    fn authentication_requested(realm: String);

    #[func]
    fn get_texture(&self) -> Gd<ImageTexture> {
        self.texture.clone()
    }

    #[func]
    fn get_current_url(&self) -> GodotString {
        self.webview
            .as_ref()
            .and_then(|wv| wv.url())
            .map(|u| u.to_string().into())
            .unwrap_or_default()
    }

    #[func]
    fn get_page_title(&self) -> GodotString {
        self.webview
            .as_ref()
            .and_then(|wv| wv.page_title())
            .map(|t| t.into())
            .unwrap_or_default()
    }

    #[func]
    fn get_load_status(&self) -> GodotString {
        self.webview
            .as_ref()
            .map(|wv| format!("{:?}", wv.load_status()).into())
            .unwrap_or_default()
    }

    #[func]
    fn can_go_back(&self) -> bool {
        self.webview.as_ref().map_or(false, |wv| wv.can_go_back())
    }

    #[func]
    fn can_go_forward(&self) -> bool {
        self.webview.as_ref().map_or(false, |wv| wv.can_go_forward())
    }

    #[func]
    fn load_url(&self, url: GodotString) {
        if let (Some(ref wv), Ok(parsed)) = (self.webview.as_ref(), Url::parse(&url.to_string()))
        {
            wv.load(parsed);
        }
    }

    #[func]
    fn reload(&self) {
        if let Some(ref wv) = self.webview {
            wv.reload();
        }
    }

    #[func]
    fn go_back(&self) {
        if let Some(ref wv) = self.webview {
            if wv.can_go_back() {
                wv.go_back(1);
            }
        }
    }

    #[func]
    fn go_forward(&self) {
        if let Some(ref wv) = self.webview {
            if wv.can_go_forward() {
                wv.go_forward(1);
            }
        }
    }

    #[func]
    fn resize(&self, width: i32, height: i32) {
        let w = width.max(1) as u32;
        let h = height.max(1) as u32;
        if let Some(ref ctx) = self.render_ctx {
            ctx.resize(PhysicalSize::new(w, h));
        }
        if let Some(ref wv) = self.webview {
            wv.resize(PhysicalSize::new(w, h));
        }
    }

    #[func]
    fn evaluate_javascript(&self, script: GodotString) {
        if let Some(ref wv) = self.webview {
            let s = script.to_string();
            wv.evaluate_javascript(s, |result| match result {
                Ok(val) => godot_print!("JS: {:?}", val),
                Err(e) => godot_print!("JS err: {:?}", e),
            });
        }
    }

    #[func]
    fn focus(&self) {
        if let Some(ref wv) = self.webview {
            wv.focus();
        }
    }

    #[func]
    fn blur(&self) {
        if let Some(ref wv) = self.webview {
            wv.blur();
        }
    }

    #[func]
    fn set_page_zoom(&self, zoom: f64) {
        if let Some(ref wv) = self.webview {
            wv.set_page_zoom(zoom as f32);
        }
    }

    #[func]
    fn send_mouse_move(&self, x: f32, y: f32) {
        if let Some(ref wv) = self.webview {
            use euclid::TypedPoint2D;
            let pt = WebViewPoint::Device(TypedPoint2D::new(x, y));
            let ev = MouseMoveEvent {
                point: pt,
                ..Default::default()
            };
            wv.notify_input_event(InputEvent::MouseMove(ev));
        }
    }

    #[func]
    fn send_mouse_click(&self, x: f32, y: f32, button: i32, pressed: bool) {
        if let Some(ref wv) = self.webview {
            use euclid::TypedPoint2D;
            let btn = match button {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                _ => return,
            };
            let action = if pressed {
                MouseButtonAction::Down
            } else {
                MouseButtonAction::Up
            };
            let pt = WebViewPoint::Device(TypedPoint2D::new(x, y));
            let ev = MouseButtonEvent {
                point: pt,
                button: btn,
                action,
                ..Default::default()
            };
            wv.notify_input_event(InputEvent::MouseButton(ev));
        }
    }

    #[func]
    fn send_scroll(&self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
        if let Some(ref wv) = self.webview {
            use euclid::TypedPoint2D;
            let pt = WebViewPoint::Device(TypedPoint2D::new(x, y));
            let ev = WheelEvent {
                point: pt,
                delta: WheelDelta {
                    x: delta_x,
                    y: delta_y,
                    z: 0.0,
                },
                mode: WheelMode::DeltaLine,
            };
            wv.notify_input_event(InputEvent::Wheel(ev));
        }
    }

    #[func]
    fn send_key(&self, keycode: i32, pressed: bool) {
        if let Some(ref wv) = self.webview {
            let key = map_key(keycode);
            let state = if pressed {
                KeyState::Down
            } else {
                KeyState::Up
            };
            let ev = KeyboardEvent {
                state,
                key,
                ..Default::default()
            };
            wv.notify_input_event(InputEvent::Keyboard(ev));
        }
    }

    #[func]
    fn send_text(&self, text: GodotString) {
        if let Some(ref wv) = self.webview {
            wv.notify_input_event(InputEvent::Ime(ImeEvent {
                state: CompositionState::CompositionUpdate,
                text: Some(text.to_string()),
                ..Default::default()
            }));
            wv.notify_input_event(InputEvent::Ime(ImeEvent {
                state: CompositionState::CompositionEnd,
                text: Some(text.to_string()),
                ..Default::default()
            }));
        }
    }

    #[func]
    fn take_screenshot_async(&self) {
        if let Some(ref wv) = self.webview {
            wv.take_screenshot(None, |result| match result {
                Ok(img) => godot_print!("Screenshot {}x{}", img.width(), img.height()),
                Err(e) => godot_print!("Screenshot err: {:?}", e),
            });
        }
    }
}

#[godot_api]
impl Node for WebViewBrowser {
    fn init(base: Base<Node>) -> Self {
        let state = Rc::new(RefCell::new(SharedState::new()));

        let image = Image::create_empty(1, 1, false, ImageFormat::RGBA8);
        let texture = ImageTexture::create_from_image(image.clone());

        Self {
            state,
            servo: None,
            webview: None,
            render_ctx: None,
            texture,
            image,
            tex_width: 1,
            tex_height: 1,
            initial_url: GodotString::from("about:blank"),
            base,
        }
    }

    fn on_ready(&mut self) {
        servo::setup_logging();
        let url = Url::parse(&self.initial_url.to_string())
            .unwrap_or_else(|_| Url::parse("about:blank").unwrap());

        let waker = Box::new(BrowserWaker);
        let servo = ServoBuilder::default().event_loop_waker(waker).build();

        let servo_delegate = BrowserServoDelegate {
            state: self.state.clone(),
        };
        servo.set_delegate(Rc::new(servo_delegate));

        let size = PhysicalSize::new(800, 600);
        let ctx = match SoftwareRenderingContext::new(size) {
            Ok(c) => c,
            Err(e) => {
                godot_error!("SoftwareRenderingContext failed: {:?}", e);
                return;
            }
        };
        let ctx: Rc<dyn RenderingContext> = Rc::new(ctx);

        let wv_delegate = BrowserWebViewDelegate {
            state: self.state.clone(),
        };

        let wv = WebViewBuilder::new(&servo, ctx.clone())
            .delegate(Rc::new(wv_delegate))
            .url(url)
            .build();
        wv.show();

        self.servo = Some(servo);
        self.webview = Some(wv);
        self.render_ctx = Some(ctx);

        let img = Image::create_empty(800, 600, false, ImageFormat::RGBA8);
        self.texture = ImageTexture::create_from_image(img.clone());
        self.image = img;
        self.tex_width = 800;
        self.tex_height = 600;
    }

    fn process(&mut self, _delta: f64) {
        if let Some(ref servo) = self.servo {
            servo.spin_event_loop();
            self.drain_signals();
            if self.state.borrow().frame_ready {
                self.state.borrow_mut().frame_ready = false;
                self.paint();
            }
        }
    }
}

impl WebViewBrowser {
    fn paint(&mut self) {
        if let (Some(ref wv), Some(ref ctx)) = (self.webview.as_ref(), self.render_ctx.as_ref()) {
            wv.paint();
            let size = ctx.size();
            let rect = euclid::Box2D::from_size(euclid::Size2D::new(
                size.width as i32,
                size.height as i32,
            ));
            if let Some(buf) = ctx.read_to_image(rect) {
                let (w, h) = buf.dimensions();
                let raw = buf.into_raw();
                if w != self.tex_width || h != self.tex_height {
                    self.image = Image::create_empty(w as i64, h as i64, false, ImageFormat::RGBA8);
                    self.tex_width = w;
                    self.tex_height = h;
                }
                self.image.set_data(
                    w as i64,
                    h as i64,
                    false,
                    ImageFormat::RGBA8,
                    raw.into(),
                );
                self.texture.update(&self.image);
            }
        }
    }

    fn drain_signals(&mut self) {
        let events = std::mem::take(&mut self.state.borrow_mut().events);
        let mut base = self.base_mut();
        for e in events {
            match e {
                SignalEvent::UrlChanged(u) => base.emit_signal("url_changed", &[u.to_variant()]),
                SignalEvent::PageTitleChanged(t) => {
                    base.emit_signal("page_title_changed", &[t.unwrap_or_default().to_variant()])
                }
                SignalEvent::LoadStatusChanged(s) => {
                    base.emit_signal("load_status_changed", &[s.to_variant()])
                }
                SignalEvent::CursorChanged(c) => {
                    base.emit_signal("cursor_changed", &[c.to_variant()])
                }
                SignalEvent::NewFrameReady => base.emit_signal("new_frame_ready", &[]),
                SignalEvent::HistoryChanged(entries, current) => {
                    let arr: Array = entries.iter().map(|s| s.to_variant()).collect();
                    base.emit_signal("history_changed", &[arr.to_variant(), (current as i64).to_variant()])
                }
                SignalEvent::FocusChanged(f) => {
                    base.emit_signal("focus_changed", &[f.to_variant()])
                }
                SignalEvent::StatusTextChanged(s) => {
                    base.emit_signal("status_text_changed", &[s.unwrap_or_default().to_variant()])
                }
                SignalEvent::FaviconChanged => base.emit_signal("favicon_changed", &[]),
                SignalEvent::Closed => base.emit_signal("webview_closed", &[]),
                SignalEvent::Crashed(r) => base.emit_signal("crashed", &[r.to_variant()]),
                SignalEvent::FullscreenStateChanged(fs) => {
                    base.emit_signal("fullscreen_state_changed", &[fs.to_variant()])
                }
                SignalEvent::ConsoleMessage(lvl, msg) => {
                    base.emit_signal("console_message", &[lvl.to_variant(), msg.to_variant()])
                }
                SignalEvent::NavigationRequested(u) => {
                    base.emit_signal("navigation_requested", &[u.to_variant()])
                }
                SignalEvent::PermissionRequested(f) => {
                    base.emit_signal("permission_requested", &[f.to_variant()])
                }
                SignalEvent::AuthenticationRequested(r) => {
                    base.emit_signal("authentication_requested", &[r.to_variant()])
                }
            }
        }
    }
}
