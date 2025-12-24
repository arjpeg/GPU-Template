use std::sync::Arc;

use glam::vec3;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{
    input::InputState,
    renderer::{Renderer, camera::Camera},
    timer::FrameTimer,
};

/// Manages all subsystems and handles incoming events.
pub struct App {
    /// The primary window being rendered onto.
    pub window: Arc<Window>,
    /// The renderer responsible drawing all game content to the world.
    pub renderer: Renderer,
    /// The primary camera describing the player's orientation.
    pub camera: Camera,

    /// The state of all input systems.
    pub input: InputState,
    /// The timer keeping track of frame durations.
    pub timer: FrameTimer,
}

impl App {
    /// Creates a new [`App`], targetting the given window.
    pub async fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(Arc::clone(&window)).await.unwrap();

        let camera = Camera {
            position: vec3(0.0, 0.0, 2.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 45.0f32.to_radians(),
            aspect_ratio: 0.0,
            movement_sensitivity: 2.0,
            mouse_sensitivity: 0.005,
        };

        let input = InputState::new(Arc::clone(&window));
        let timer = FrameTimer::new();

        Self {
            window,
            renderer,
            camera,
            input,
            timer,
        }
    }

    /// Processes an incoming [`WindowEvent`].
    pub fn window_event(&mut self, event_loop: &ActiveEventLoop, event: &WindowEvent) {
        self.input.window_event(event);

        match event {
            WindowEvent::Resized(size) => self.resize(*size),

            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => self.update(),

            _ => {}
        }
    }

    /// Processes an incoming [`DeviceEvent`].
    pub fn device_event(&mut self, event: &DeviceEvent) {
        self.input.device_event(event);

        if self.input.focused {
            self.camera.update_orientation(self.input.mouse_delta);
        }
    }

    /// Runs the render and update cycle of the app.
    fn update(&mut self) {
        self.timer.tick();

        let dt = self.timer.dt.as_secs_f32();

        if self.input.focused {
            self.camera
                .update_position(|k| self.input.keys_held.contains(k), dt);
        }

        self.renderer
            .render(&self.camera, || self.window.pre_present_notify());

        self.window.request_redraw();
    }

    /// Resizes the state of the app to match the new window size.
    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.renderer.resize(size);
        self.camera.resize(size);
    }
}

/// Manages the creation and lifecycle of the actual [`App`].
pub struct AppHandler {
    /// A proxy to create the app when dealing with async events (only needed on web).
    #[cfg(target_arch = "wasm32")]
    proxy: Option<EventLoopProxy<App>>,

    /// The initialized Some(app), or None if the window hasn't been created yet.
    app: Option<App>,
}

impl AppHandler {
    /// Creates a new [`AppHandler`], the main entry point to the app.
    pub fn new(#[cfg(target_arch = "wasm32")] proxy: EventLoopProxy<App>) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            proxy: Some(proxy),
            app: None,
        }
    }
}

impl ApplicationHandler<App> for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{JsCast, UnwrapThrowExt};
            use web_sys::window;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let canvas = window()
                .and_then(|window| window.document())
                .and_then(|document| document.get_element_by_id(CANVAS_ID))
                .unwrap_throw()
                .unchecked_into();

            window_attributes = window_attributes.with_canvas(Some(canvas));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.app = Some(pollster::block_on(App::new(window)));
        }

        #[cfg(target_arch = "wasm32")]
        {
            let proxy = self.proxy.take().unwrap();

            wasm_bindgen_futures::spawn_local(async move {
                assert!(proxy.send_event(App::new(window).await).is_ok());
            });
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, mut app: App) {
        app.resize(app.window.inner_size());
        self.app = Some(app);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(app) = &mut self.app {
            app.window_event(event_loop, &event);
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let Some(app) = &mut self.app {
            app.device_event(&event);
        }
    }
}
