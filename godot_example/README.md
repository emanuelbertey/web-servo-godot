# WebServo Example — Navegador Web en Godot

Ejemplo completo de cómo usar el addon `rust-gdextension` (WebViewBrowser) en un proyecto Godot 4.

## Qué muestra este ejemplo

- Un navegador web funcional dentro de Godot
- Barra de dirección con navegación por URL
- Botones de retroceso, adelante y recarga
- Control de zoom (slider + gestos de pinza)
- Campo para ejecutar JavaScript directamente
- Barra de estado con título y URL
- Input completo: mouse, teclado, scroll

## Requisitos

1. Godot 4.3+
2. La librería nativa compilada (ver README principal)

## Estructura

```
godot_example/
├── project.godot                         # Configuración del proyecto
├── addons/
│   └── rust-gdextension/
│       └── rust.gdextension             # Descriptor (apunta a ../rust/target/)
├── scenes/
│   └── main.tscn                        # Escena principal
├── scripts/
│   └── browser_ui.gd                    # Script del navegador
└── README.md
```

## Cómo usar

### 1. Compilar la librería

Desde la raíz del proyecto:

```bash
cd rust
cargo build --release
```

### 2. Abrir en Godot

Abrir la carpeta `godot_example/` con Godot 4.3+. El addon está configurado para encontrar la librería en `../rust/target/`.

### 3. Ejecutar

Presionar F5. Se abrirá la ventana con el navegador mostrando `https://example.com`.

## Funcionalidades del ejemplo

### Navegación por URL

Escribe cualquier URL en la barra y presiona Enter:
```
https://google.com
wikipedia.org
github.com
```

Si no empieza con `http://` o `https://`, se agrega `https://` automáticamente.

### JavaScript

Escribe JavaScript en el campo y presiona el botón ▶ o Enter:
```javascript
document.title = "Hola desde Godot!"
document.body.style.backgroundColor = "lightblue"
window.location.href
```

Los resultados se imprimen en la consola de Godot.

### Zoom

- Usa el slider en la barra de herramientas
- En trackpad: gesto de pinza sobre el área web
- Rango: 25% a 400%

### Input

El script captura todo el input del `TextureRect` y lo envía al browser:
- Mouse move y clicks
- Scroll (rueda del mouse / pan gesture)
- Teclado
- IME (texto)

## Personalización

### Cambiar URL inicial

En el editor, selecciona el nodo `WebViewBrowser` en la escena `main.tscn` y cambia `Initial URL` en el inspector.

O en el script `_ready()`:
```gdscript
browser.load_url("https://tusitio.com")
```

### Agregar más señales

Consultar la lista completa de señales en el README principal. Ejemplo:
```gdscript
browser.console_message.connect(_on_console)
browser.crashed.connect(_on_crash)

func _on_console(level: String, msg: String) -> void:
    print("JS: [%s] %s" % [level, msg])

func _on_crash(reason: String) -> void:
    push_error("Browser crashed: " + reason)
```

### Renderizar a textura personalizada

Puedes usar la textura en cualquier nodo:
```gdscript
# En un Sprite2D
$Sprite2D.texture = browser.get_texture()

# En un Material
$MeshInstance3D.get_surface_override_material(0).albedo_texture = browser.get_texture()
```
