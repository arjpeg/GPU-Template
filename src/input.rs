use std::{collections::HashSet, sync::Arc};

use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window},
};

/// Manages an up to date representation of all input devices.
pub struct InputState {
    /// The keys currently being held down.
    pub keys_held: HashSet<KeyCode>,

    /// The last known mouse position.
    pub last_mouse: Option<(f32, f32)>,
    /// The last known change in mouse position, without regards to acceleration or screen scale
    /// factor (useful for FPS cameras).
    pub mouse_delta: (f32, f32),

    /// The window from which events are being captured.
    window: Arc<Window>,

    /// Represents whether the app currently has focus or not.
    pub focused: bool,
}

impl InputState {
    /// Creates a new [`InputState`].
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            keys_held: HashSet::new(),
            last_mouse: None,
            mouse_delta: (0.0, 0.0),
            focused: false,
            window,
        }
    }

    /// Handles a [`WindowEvent`].
    pub fn window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                if *code == KeyCode::KeyQ {
                    self.set_focused(false);
                    return;
                }

                match state {
                    ElementState::Pressed => self.keys_held.insert(*code),
                    ElementState::Released => self.keys_held.remove(code),
                };
            }

            WindowEvent::MouseInput { .. } => {
                self.set_focused(true);
            }

            WindowEvent::CursorMoved { position, .. } => self.last_mouse = Some((*position).into()),

            _ => {}
        }
    }

    /// Handles a [`DeviceEvent`].
    pub fn device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            self.mouse_delta = (*dx as _, *dy as _);
        }
    }

    /// Sets the state of focused, updating the cursor state as needed.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;

        match focused {
            true => {
                self.window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
                self.window.set_cursor_visible(true);
            }
            false => {
                self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
                self.window.set_cursor_visible(true);
            }
        }
    }
}
