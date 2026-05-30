# WebViewBrowser — Arquitectura

## Módulos

| Archivo | Contenido | Depende de Godot? |
|---|---|---|
| `lib.rs` | `mod` declarations + `ExtensionLibrary` | Sí (mínimo) |
| `shared.rs` | `SharedState`, `SignalEvent`, `BrowserWaker` | No |
| `delegate.rs` | `BrowserWebViewDelegate`, `BrowserServoDelegate` | No |
| `input_map.rs` | `map_key(code)` — Godot keycode → `Key` | No |
| `servo_browser.rs` | `WebViewBrowser` (clase Godot) | Sí |

## shared.rs — Estado compartido

```rust
enum SignalEvent {          // Eventos delegates → clase Godot
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

struct SharedState {
    events: Vec<SignalEvent>,   // cola de eventos pendientes
    frame_ready: bool,          // flag para nuevo frame listo
}

struct BrowserWaker;            // EventLoopWaker (no-op)
```

## delegate.rs — Delegados de Servo

- `BrowserWebViewDelegate`: implementa `WebViewDelegate`, empuja `SignalEvent` al `SharedState`
- `BrowserServoDelegate`: implementa `ServoDelegate`, logea errores con `godot_error!`

Ambos reciben `Rc<RefCell<SharedState>>` para comunicación thread-safe (single-thread).

## input_map.rs — Mapa de teclas

`map_key(i32) -> Key` traduce keycodes de Godot (ej. `4194305` → `Key::Backspace`) a tipos `keyboard_types::Key`.

## servo_browser.rs — WebViewBrowser (clase Godot)

### Campos principales

| Campo | Tipo | Propósito |
|---|---|---|
| `state` | `Rc<RefCell<SharedState>>` | Comunicación delegates → clase |
| `servo` | `Option<Servo>` | Instancia Servo |
| `webview` | `Option<WebView>` | WebView activa |
| `render_ctx` | `Option<Rc<dyn RenderingContext>>` | Contexto software rendering |
| `texture` | `Gd<ImageTexture>` | Textura expuesta a Godot |
| `image` | `Gd<Image>` | Buffer de píxeles intermedio |

### 16 señales

`url_changed`, `page_title_changed`, `load_status_changed`, `cursor_changed`, `new_frame_ready`, `history_changed`, `focus_changed`, `status_text_changed`, `favicon_changed`, `webview_closed`, `crashed`, `fullscreen_state_changed`, `console_message`, `navigation_requested`, `permission_requested`, `authentication_requested`.

### Métodos `#[func]` principales

| Método | Descripción |
|---|---|
| `get_texture()` | Devuelve `ImageTexture` para mostrar en un `TextureRect` |
| `load_url(url)` | Navega a una URL |
| `reload()` / `go_back()` / `go_forward()` | Navegación |
| `resize(w, h)` | Cambia tamaño del viewport |
| `evaluate_javascript(script)` | Ejecuta JS (resultado por consola) |
| `send_mouse_move(x, y)` | Mouse move → Servo |
| `send_mouse_click(x, y, button, pressed)` | Click (button: 0=Left,1=Middle,2=Right) |
| `send_scroll(x, y, dx, dy)` | Scroll wheel |
| `send_key(keycode, pressed)` | Tecla (keycode Godot) |
| `send_text(text)` | Entrada de texto vía IME |
| `take_screenshot_async()` | Captura asíncrona |

### Ciclo de vida

1. `init()` — crea `SharedState`, texturas vacías
2. `on_ready()` — construye `Servo`, `SoftwareRenderingContext`, `WebView`, carga URL inicial
3. `_process(delta)` — `servo.spin_event_loop()` → `drain_signals()` → `paint()` si hay frame nuevo

### Pipeline de rendering

```
WebView::paint()
  → RenderingContext::read_to_image(rect)
    → Image::set_data(w, h, false, RGBA8, raw_pixels)
      → ImageTexture::update(&image)
```

### drain_signals()

Toma `SignalEvent`s del `SharedState`, hace `match` y emite la señal Godot correspondiente vía `base_mut().emit_signal(...)`.
