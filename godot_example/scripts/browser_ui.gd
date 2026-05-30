extends Control

## Ejemplo de uso del WebViewBrowser con barra de navegación completa.

@onready var browser: WebViewBrowser = %WebViewBrowser
@onready var url_bar: LineEdit = %UrlBar
@onready var title_label: Label = %TitleLabel
@onready var status_label: Label = %StatusLabel
@onready var web_texture: TextureRect = %WebTexture
@onready var back_btn: Button = %BackBtn
@onready var forward_btn: Button = %ForwardBtn
@onready var reload_btn: Button = %ReloadBtn
@onready var zoom_slider: HSlider = %ZoomSlider
@onready var zoom_label: Label = %ZoomLabel
@onready var js_input: LineEdit = %JsInput
@onready var js_btn: Button = %JsBtn

var is_dragging := false
var last_mouse_pos := Vector2.ZERO

func _ready() -> void:
	# Conectar señales del browser
	browser.url_changed.connect(_on_url_changed)
	browser.page_title_changed.connect(_on_title_changed)
	browser.load_status_changed.connect(_on_load_status)
	browser.new_frame_ready.connect(_on_new_frame)
	browser.console_message.connect(_on_console_message)
	browser.crashed.connect(_on_crashed)
	browser.history_changed.connect(_on_history_changed)
	browser.favicon_changed.connect(_on_favicon_changed)

	# Conectar UI
	back_btn.pressed.connect(_on_back)
	forward_btn.pressed.connect(_on_forward)
	reload_btn.pressed.connect(_on_reload)
	url_bar.text_submitted.connect(_on_url_submitted)
	zoom_slider.value_changed.connect(_on_zoom_changed)
	js_btn.pressed.connect(_on_js_execute)
	js_input.text_submitted.connect(_on_js_submitted)

	# Inicializar zoom slider
	zoom_slider.value = 1.0
	_on_zoom_changed(1.0)

	# Cargar URL por defecto
	browser.initial_url = "https://example.com"
	browser.load_url("https://example.com")


func _process(_delta: float) -> void:
	# Redimensionar el browser al tamaño del TextureRect
	var size = web_texture.size
	if size.x > 0 and size.y > 0:
		browser.resize(int(size.x), int(size.y))


func _input(event: InputEvent) -> void:
	if not web_texture.get_global_rect().has_point(get_global_mouse_position()):
		return

	if event is InputEventMouseButton:
		var local = web_texture.make_input_local(event as InputEventMouseButton)
		var pos = local.position
		var btn = (event as InputEventMouseButton).button_index
		var pressed = (event as InputEventMouseButton).pressed
		var button_id = 0
		match btn:
			MOUSE_BUTTON_LEFT: button_id = 0
			MOUSE_BUTTON_MIDDLE: button_id = 1
			MOUSE_BUTTON_RIGHT: button_id = 2
		browser.send_mouse_click(pos.x, pos.y, button_id, pressed)

	elif event is InputEventMouseMotion:
		var local = web_texture.make_input_local(event as InputEventMouseMotion)
		browser.send_mouse_move(local.position.x, local.position.y)

	elif event is InputEventKey:
		browser.send_key((event as InputEventKey).keycode, (event as InputEventKey).pressed)

	elif event is InputEventPanGesture:
		var local = web_texture.make_input_local(event as InputEventPanGesture)
		browser.send_scroll(local.position.x, local.position.y, 0.0, event.delta.y * 30.0)

	elif event is InputEventMagnifyGesture:
		var new_zoom = zoom_slider.value + event.factor * 0.1
		zoom_slider.value = clampf(new_zoom, 0.25, 4.0)


# ── Señales del browser ──────────────────────────────────────────

func _on_url_changed(url: String) -> void:
	url_bar.text = url


func _on_title_changed(title: String) -> void:
	title_label.text = title if title != "" else "WebServo Example"


func _on_load_status(status: String) -> void:
	status_label.text = status


func _on_new_frame() -> void:
	web_texture.texture = browser.get_texture()


func _on_console_message(level: String, message: String) -> void:
	print("[JS %s] %s" % [level, message])


func _on_crashed(reason: String) -> void:
	status_label.text = "CRASH: " + reason


func _on_history_changed(_entries: Array, _current: int) -> void:
	back_btn.disabled = not browser.can_go_back()
	forward_btn.disabled = not browser.can_go_forward()


func _on_favicon_changed() -> void:
	pass


# ── Navegación ───────────────────────────────────────────────────

func _on_back() -> void:
	browser.go_back()


func _on_forward() -> void:
	browser.go_forward()


func _on_reload() -> void:
	browser.reload()


func _on_url_submitted(text: String) -> void:
	var url = text.strip_edges()
	if not url.begins_with("http://") and not url.begins_with("https://"):
		url = "https://" + url
	browser.load_url(url)
	browser.focus()


func _on_zoom_changed(value: float) -> void:
	browser.set_page_zoom(value)
	zoom_label.text = "%d%%" % int(value * 100)


func _on_js_execute() -> void:
	browser.evaluate_javascript(js_input.text)


func _on_js_submitted(text: String) -> void:
	browser.evaluate_javascript(text)
