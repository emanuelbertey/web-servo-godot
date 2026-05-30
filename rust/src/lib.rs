mod shared;
mod delegate;
mod input_map;
mod servo_browser;

use godot::prelude::*;

struct WebServo;

#[gdextension]
unsafe impl ExtensionLibrary for WebServo {}
