use std::collections::HashMap;
use glam::Vec2;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;

pub struct InputHelper {
	keysCurrent: HashMap<Keycode, bool>,
	keysLast: HashMap<Keycode, bool>,
	
	mousePos: Vec2,
	mouseBtnsCurrent: HashMap<MouseButton, bool>,
	mouseBtnsLast: HashMap<MouseButton, bool>,
}

impl InputHelper {
	pub fn new() -> Self {
		InputHelper {
			keysCurrent: HashMap::new(),
			keysLast: HashMap::new(),
			
			mousePos: Vec2::ZERO,
			mouseBtnsCurrent: HashMap::new(),
			mouseBtnsLast: HashMap::new(),
		}
	}
	
	pub fn update(&mut self) {
		self.keysLast = self.keysCurrent.clone();
		self.mouseBtnsLast = self.mouseBtnsCurrent.clone();
	}
	
	pub fn handleEvents(&mut self, event: &Event) {
		match *event {
			Event::KeyDown { keycode, .. } => {
				if keycode.is_some() {
					self.keysCurrent.insert(keycode.unwrap(), true);
				}
			},
			Event::KeyUp { keycode, .. } => {
				if keycode.is_some() {
					self.keysCurrent.insert(keycode.unwrap(), false);
				}
			},
			Event::MouseMotion { x, y, .. } => {
				self.mousePos.x = x;
				self.mousePos.y = y;
			},
			Event::MouseButtonDown { mouse_btn, .. } => {
				self.mouseBtnsCurrent.insert(mouse_btn, true);
			},
			Event::MouseButtonUp { mouse_btn, .. } => {
				self.mouseBtnsCurrent.insert(mouse_btn, false);
			},
			_ => {},
		}
	}
	
	pub fn isKeyJustPressed(&self, keycode: Keycode) -> bool {
		self.keysCurrent.get(&keycode).unwrap_or(&false).clone() && !self.keysLast.get(&keycode).unwrap_or(&false).clone()
	}
	
	pub fn isKeyJustReleased(&self, keycode: Keycode) -> bool {
		!self.keysCurrent.get(&keycode).unwrap_or(&false).clone() && self.keysLast.get(&keycode).unwrap_or(&false).clone()
	}
	
	pub fn isKeyPressed(&self, keycode: Keycode) -> bool {
		self.keysCurrent.get(&keycode).unwrap_or(&false).clone() && self.keysLast.get(&keycode).unwrap_or(&false).clone()
	}
	
	pub fn mousePos(&self) -> Vec2 {
		self.mousePos
	}
	
	pub fn isMouseJustPressed(&self, mouseBtn: MouseButton) -> bool {
		self.mouseBtnsCurrent.get(&mouseBtn).unwrap_or(&false).clone() && !self.mouseBtnsLast.get(&mouseBtn).unwrap_or(&false).clone()
	}
	
	pub fn isMouseJustReleased(&self, mouseBtn: MouseButton) -> bool {
		!self.mouseBtnsCurrent.get(&mouseBtn).unwrap_or(&false).clone() && self.mouseBtnsLast.get(&mouseBtn).unwrap_or(&false).clone()
	}
	
	pub fn isMousePressed(&self, mouseBtn: MouseButton) -> bool {
		self.mouseBtnsCurrent.get(&mouseBtn).unwrap_or(&false).clone() && self.mouseBtnsLast.get(&mouseBtn).unwrap_or(&false).clone()
	}
}