# WebServo — Navegador Web para Godot Engine

GDExtension que integra el motor de renderizado web **Servo** (Mozilla) directamente en Godot 4. Permite mostrar páginas web en vivo dentro de una aplicación Godot, con rendering completo, manejo de input y ejecución de JavaScript.

## Características

- Rendering HTML/CSS/JS completo via Servo (software rasterization)
- Nodo `WebViewBrowser` integrado en el árbol de escenas de Godot
- Soporte multiplataforma: Linux, Windows, macOS (x86_64 + arm64), Android (arm64 + x86_64)
- Navegación (URL, historial, recarga)
- Input completo: mouse, scroll, teclado, IME
- Ejecución de JavaScript desde GDScript
- 16 señales para events (URL, título, carga, consola, crashes, etc.)
- Requiere Godot 4.3+

## Compilación

### Requisitos

- [Rust](https://rustup.rs/) (stable)
- Godot 4.3+
- Linux: `libdbus-1-dev`, `libssl-dev`, `libfontconfig-dev`, `libx11-dev`, `libgl1-mesa-dev`, etc.
- macOS: `brew install pkg-config`
- Windows: MinGW (ya incluido en CI)

### Build local

```bash
cd rust
cargo build --release
```

La librería se genera en:
- Linux: `rust/target/release/librust_gdextension.so`
- Windows: `rust/target/release/rust_gdextension.dll`
- macOS: `rust/target/release/librust_gdextension.dylib`

### CI/CD

El workflow de GitHub Actions (`build.yml`) compila automáticamente para todas las plataformas en cada push a `main`. El resultado es un `webtouri.zip` con todos los binarios.

## Uso en Godot

### 1. Instalar el addon

Copia la carpeta `addons/rust-gdextension/` dentro de tu proyecto Godot.

### 2. Agregar el nodo

En el editor de Godot, agrega un nodo `WebViewBrowser` a tu escena.

### 3. Configurar la URL inicial

En el inspector, establece la propiedad `Initial URL` (ej: `https://example.com`).

### 4. Obtener la textura

La textura renderizada se obtiene con `get_texture()`. Asignarla a un `TextureRect` o `Sprite2D`:

```gdscript
$TextureRect.texture = $WebViewBrowser.get_texture()
```

## API del WebViewBrowser

### Propiedades exportadas

| Propiedad | Tipo | Descripción |
|---|---|---|
| `initial_url` | String | URL a cargar al iniciar (`about:blank` por defecto) |

### Métodos

| Método | Descripción |
|---|---|
| `get_texture()` → `ImageTexture` | Textura renderizada |
| `get_current_url()` → `String` | URL actual |
| `get_page_title()` → `String` | Título de la página |
| `load_url(url)` | Navegar a una URL |
| `reload()` | Recargar página |
| `go_back()` / `go_forward()` | Navegación historial |
| `resize(width, height)` | Redimensionar viewport |
| `evaluate_javascript(script)` | Ejecutar JavaScript |
| `focus()` / `blur()` | Focus/blur del webview |
| `set_page_zoom(zoom)` | Zoom de página |
| `send_mouse_move(x, y)` | Evento mouse move |
| `send_mouse_click(x, y, button, pressed)` | Click (0=Left, 1=Middle, 2=Right) |
| `send_scroll(x, y, dx, dy)` | Scroll wheel |
| `send_key(keycode, pressed)` | Evento teclado |
| `send_text(text)` | Texto via IME |
| `can_go_back()` / `can_go_forward()` | Verificar historial |
| `take_screenshot_async()` | Captura de pantalla |

### Señales

| Señal | Parámetros | Descripción |
|---|---|---|
| `url_changed` | `url: String` | URL cambió |
| `page_title_changed` | `title: String` | Título cambió |
| `load_status_changed` | `status: String` | Estado de carga cambió |
| `new_frame_ready` | — | Nuevo frame renderizado |
| `console_message` | `level, message` | Mensaje de consola JS |
| `crashed` | `reason: String` | El webview crashó |
| `navigation_requested` | `url: String` | Navegación solicitada |
| `history_changed` | `entries, current` | Historial cambió |
| `focus_changed` | `focused: bool` | Focus cambió |
| `cursor_changed` | `cursor: String` | Cursor cambió |
| `webview_closed` | — | Webview cerrado |
| `favicon_changed` | — | Favicon cambió |
| `fullscreen_state_changed` | `fullscreen: bool` | Fullscreen cambió |
| `status_text_changed` | `status: String` | Texto de status cambió |
| `permission_requested` | `feature: String` | Permiso solicitado |
| `authentication_requested` | `realm: String` | Autenticación HTTP solicitada |

## Arquitectura

```
┌─────────────────────────────────────────────────────┐
│                    Godot Engine                      │
│  ┌───────────────────────────────────────────────┐  │
│  │              WebViewBrowser (Node)             │  │
│  │  ┌─────────┐  ┌──────────┐  ┌─────────────┐  │  │
│  │  │ texture │  │ signals  │  │ input funcs │  │  │
│  │  └────┬────┘  └─────┬────┘  └──────┬──────┘  │  │
│  └───────┼─────────────┼──────────────┼──────────┘  │
│          │             │              │              │
│  ┌───────┴─────────────┴──────────────┴──────────┐  │
│  │            SharedState (Rc<RefCell>)           │  │
│  │  events: Vec<SignalEvent>  │  frame_ready: bool│  │
│  └──────────────────────────┬─────────────────────┘  │
└─────────────────────────────┼────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │   Servo Engine     │
                    │  ┌──────────────┐  │
                    │  │ WebView      │  │
                    │  │ Delegates    │──┼──→ push SignalEvents
                    │  │ paint()      │  │
                    │  └──────────────┘  │
                    │  RenderingContext   │
                    │  (Software)        │
                    └───────────────────┘
```

### Ciclo de vida

1. `init()` — Crea `SharedState`, texturas vacías 1x1
2. `on_ready()` — Construye Servo, `SoftwareRenderingContext` (800x600), `WebView`, carga URL
3. `_process(delta)` — Cada frame:
   - `servo.spin_event_loop()`
   - `drain_signals()` — Emite señales Godot desde la cola de eventos
   - `paint()` — Si hay frame nuevo: `WebView::paint()` → `read_to_image()` → `Image::set_data()` → `ImageTexture::update()`

### Pipeline de rendering

```
WebView::paint()
  → RenderingContext::read_to_image(rect)
    → Image::set_data(w, h, false, RGBA8, raw_pixels)
      → ImageTexture::update(&image)
```

## Estructura del proyecto

```
web-servo-godot/
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Entry point GDExtension
│       ├── servo_browser.rs    # Clase WebViewBrowser
│       ├── delegate.rs         # Delegados Servo
│       ├── shared.rs           # Estado compartido
│       ├── input_map.rs        # Mapeo de teclas
│       └── test.rs             # Clase Player (demo)
├── addons/
│   └── rust-gdextension/
│       ├── rust.gdextension   # Descriptor GDExtension
│       └── icon.png
├── godot_example/              # Ejemplo completo
├── .github/
│   └── workflows/
│       └── build.yml           # CI multiplataforma
├── project.godot
└── control.tscn
```

## Licencia

Ver [LICENSE](LICENSE).
