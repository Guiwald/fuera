//! ¡Fuera! on Mac, Windows and Linux.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let event_loop = winit::event_loop::EventLoop::new().expect("display");
    let mut runner = fuera::Runner::new(fuera_ui::App::new(Box::new(fuera::DesktopHost::new())));
    event_loop.run_app(&mut runner).expect("event loop");
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
fn main() {}
