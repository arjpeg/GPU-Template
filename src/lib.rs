pub mod application;
pub mod input;
pub mod renderer;
pub mod timer;

use crate::application::AppHandler;
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// The main entry point into the application.
pub fn run() {
    let event_loop = EventLoop::with_user_event().build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::UnwrapThrowExt;

        console_error_panic_hook::set_once();
        console_log::init().unwrap_throw();

        let proxy = event_loop.create_proxy();

        event_loop.run_app(&mut AppHandler::new(proxy)).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::builder()
            .filter(Some("gpu_template"), log::LevelFilter::Debug)
            .format_timestamp(None)
            .init();

        event_loop.run_app(&mut AppHandler::new()).unwrap();
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() {
    run();
}
