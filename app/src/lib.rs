//! ¡Fuera! on every platform: one winit loop drawing the `fuera-ui` picture with softbuffer, sounds through cpal.
//! Desktop (`main.rs`), Android (`android_main`) and the web (`start`) only differ by their [`Host`].

mod audio;
mod hosts;

use std::rc::Rc;

use fuera_core::Dir;
use fuera_ui::{App, KeyEvent, PointerEvent};
use tiny_skia::Pixmap;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, Touch, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

pub use hosts::*;

/// Drawing size in device pixels. On the web, winit reports the canvas size in CSS pixels while pointer positions
/// are in device pixels: take the CSS size times the pixel ratio.
fn physical_size(window: &Window) -> (u32, u32) {
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        if let (Some(c), Some(w)) = (window.canvas(), web_sys::window()) {
            let dpr = w.device_pixel_ratio();
            let (cw, ch) = (c.client_width() as f64 * dpr, c.client_height() as f64 * dpr);
            if cw >= 1.0 && ch >= 1.0 {
                return (cw.round() as u32, ch.round() as u32);
            }
        }
    }
    let size = window.inner_size();
    (size.width.max(1), size.height.max(1))
}

pub struct Runner {
    app: App,
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    pixmap: Option<Pixmap>,
    last: Instant,
    audio: Option<audio::Audio>,
    cursor: (f32, f32),
    mouse_down: bool,
    touch: Option<u64>,
    modifiers: winit::keyboard::ModifiersState,
    /// Voice packs already asked for (loaded, being downloaded, or missing).
    voices_tried: std::collections::HashSet<&'static str>,
}

impl Runner {
    pub fn new(app: App) -> Runner {
        Runner {
            app,
            window: None,
            surface: None,
            pixmap: None,
            last: Instant::now(),
            audio: None,
            cursor: (0.0, 0.0),
            mouse_down: false,
            touch: None,
            modifiers: Default::default(),
            voices_tried: Default::default(),
        }
    }

    fn ensure_audio(&mut self) {
        if self.audio.is_none() {
            self.audio = audio::Audio::open();
        } else if let Some(a) = &self.audio {
            a.resume();
        }
        self.load_voices();
    }

    /// Hands the recorded pronunciations the current course needs to the game once sound works: built in on Mac,
    /// Windows, Linux and Android; downloaded next to the page on the web (`voces-<pack>.fvox`), only for the courses
    /// played.
    fn load_voices(&mut self) {
        if self.audio.is_none() {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let ready: Vec<(String, Vec<u8>)> = WEB_VOICES.with(|v| v.borrow_mut().drain().collect());
            for (id, d) in ready {
                self.app.set_voices(&id, d);
            }
        }
        let Some(id) = self.app.wanted_voice_pack() else { return };
        if !self.voices_tried.insert(id) {
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(pack) = builtin_pack(id) {
            self.app.set_voices(id, pack.to_vec());
        }
        #[cfg(target_arch = "wasm32")]
        fetch_voices(id);
    }

    fn redraw(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else { return };
        let (w, h) = physical_size(window);
        if self.pixmap.as_ref().is_none_or(|p| p.width() != w || p.height() != h) {
            self.pixmap = Pixmap::new(w, h);
            let (top, bottom) = hosts::safe_insets();
            self.app.resize(w, h, top, bottom);
            let _ = surface.resize(w.try_into().unwrap(), h.try_into().unwrap());
        }
        let Some(pm) = &mut self.pixmap else { return };
        self.app.draw(pm);
        let Ok(mut buf) = surface.buffer_mut() else { return };
        let src = pm.data();
        for (dst, px) in buf.iter_mut().zip(src.chunks_exact(4)) {
            *dst = (px[0] as u32) << 16 | (px[1] as u32) << 8 | px[2] as u32;
        }
        let _ = buf.present();
    }

    fn pointer(&mut self, ev: PointerEvent, x: f32, y: f32) {
        log::debug!("pointer {ev:?} {x} {y}");
        if ev == PointerEvent::Down {
            self.ensure_audio();
        }
        self.app.pointer(ev, x, y);
        self.request();
    }

    fn request(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() && self.surface.is_some() {
            return;
        }
        let window = match &self.window {
            Some(w) => w.clone(),
            None => {
                let attrs = hosts::window_attributes();
                let w = Rc::new(el.create_window(attrs).expect("window"));
                self.window = Some(w.clone());
                w
            }
        };
        let ctx = softbuffer::Context::new(window.clone()).expect("display");
        self.surface = Some(softbuffer::Surface::new(&ctx, window.clone()).expect("surface"));
        self.pixmap = None;
        self.last = Instant::now();
        #[cfg(not(target_arch = "wasm32"))]
        self.ensure_audio();
        window.request_redraw();
    }

    fn suspended(&mut self, _el: &ActiveEventLoop) {
        // Android: the surface goes away when the app is in the background.
        self.surface = None;
        self.audio = None;
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if !matches!(event, WindowEvent::RedrawRequested) {
            log::debug!("event {event:?}");
        }
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => self.request(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as f32, position.y as f32);
                if self.mouse_down {
                    self.pointer(PointerEvent::Move, self.cursor.0, self.cursor.1);
                }
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                self.mouse_down = state == ElementState::Pressed;
                let ev = if self.mouse_down { PointerEvent::Down } else { PointerEvent::Up };
                self.pointer(ev, self.cursor.0, self.cursor.1);
            }
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::Touch(Touch { phase, location, id, .. }) => {
                let (x, y) = (location.x as f32, location.y as f32);
                match phase {
                    TouchPhase::Started if self.touch.is_none() => {
                        self.touch = Some(id);
                        self.pointer(PointerEvent::Down, x, y);
                    }
                    TouchPhase::Moved if self.touch == Some(id) => self.pointer(PointerEvent::Move, x, y),
                    TouchPhase::Ended if self.touch == Some(id) => {
                        self.touch = None;
                        self.pointer(PointerEvent::Up, x, y);
                    }
                    TouchPhase::Cancelled if self.touch == Some(id) => {
                        self.touch = None;
                        self.pointer(PointerEvent::Cancel, x, y);
                    }
                    _ => {}
                }
            }
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let k = match event.logical_key.as_ref() {
                    Key::Named(NamedKey::ArrowLeft) => Some(KeyEvent::Arrow(Dir::W)),
                    Key::Named(NamedKey::ArrowRight) => Some(KeyEvent::Arrow(Dir::E)),
                    Key::Named(NamedKey::ArrowUp) => Some(KeyEvent::Arrow(Dir::N)),
                    Key::Named(NamedKey::ArrowDown) => Some(KeyEvent::Arrow(Dir::S)),
                    Key::Named(NamedKey::Escape) | Key::Named(NamedKey::GoBack) | Key::Named(NamedKey::BrowserBack) => Some(KeyEvent::Back),
                    Key::Named(NamedKey::Backspace) => Some(KeyEvent::Undo),
                    Key::Named(NamedKey::Enter) | Key::Named(NamedKey::Space) => Some(KeyEvent::Enter),
                    Key::Named(NamedKey::Tab) => Some(KeyEvent::NextBlock),
                    Key::Character(c) => match c.to_lowercase().as_str() {
                        "z" | "u" => Some(KeyEvent::Undo),
                        "r" => Some(KeyEvent::Restart),
                        "h" => Some(KeyEvent::Hint),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(k) = k {
                    self.app.key(k);
                    self.request();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32();
        self.last = now;
        self.app.update(dt);
        let sounds = self.app.take_sounds();
        if let Some(a) = &self.audio {
            for s in sounds {
                a.play(s);
            }
        }
        self.load_voices();
        let clips = self.app.take_clips();
        if let Some(a) = &self.audio {
            for (pcm, rate) in clips {
                a.play_pcm(&pcm, rate);
            }
        }
        if self.app.needs_redraw() {
            self.request();
            el.set_control_flow(ControlFlow::WaitUntil(now + web_time::Duration::from_millis(16)));
        } else {
            el.set_control_flow(ControlFlow::Wait);
        }
    }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("fuera"));
    std::panic::set_hook(Box::new(|info| log::error!("panic: {info}")));
    let event_loop = match winit::event_loop::EventLoop::builder().with_android_app(app.clone()).build() {
        Ok(e) => e,
        Err(e) => {
            log::error!("event loop: {e}");
            return;
        }
    };
    let host = hosts::AndroidHost::new(app);
    let mut runner = Runner::new(App::new(Box::new(host)));
    if let Err(e) = event_loop.run_app(&mut runner) {
        log::error!("{e}");
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    use winit::platform::web::EventLoopExtWebSys;
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(if cfg!(debug_assertions) { log::Level::Debug } else { log::Level::Info });
    let event_loop = winit::event_loop::EventLoop::new().expect("event loop");
    let runner = Runner::new(App::new(Box::new(hosts::WebHost)));
    event_loop.spawn_app(runner);
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_VOICES: std::cell::RefCell<std::collections::HashMap<String, Vec<u8>>> = std::cell::RefCell::new(Default::default());
}

/// Downloads a voice pack next to the page (10 to 15 MB) in the background.
#[cfg(target_arch = "wasm32")]
fn fetch_voices(id: &'static str) {
    wasm_bindgen_futures::spawn_local(async move {
        use wasm_bindgen::JsCast;
        let Some(w) = web_sys::window() else { return };
        let Ok(resp) = wasm_bindgen_futures::JsFuture::from(w.fetch_with_str(&format!("voces-{id}.fvox"))).await else { return };
        let Ok(resp) = resp.dyn_into::<web_sys::Response>() else { return };
        if !resp.ok() {
            return;
        }
        let Ok(buf) = resp.array_buffer() else { return };
        let Ok(buf) = wasm_bindgen_futures::JsFuture::from(buf).await else { return };
        let bytes = js_sys::Uint8Array::new(&buf).to_vec();
        WEB_VOICES.with(|v| v.borrow_mut().insert(id.to_string(), bytes));
    });
}

/// The voice packs built into the native apps.
#[cfg(not(target_arch = "wasm32"))]
fn builtin_pack(id: &str) -> Option<&'static [u8]> {
    static ES: &[u8] = include_bytes!("../../assets/es/voces/voces-es.fvox");
    static ES_419: &[u8] = include_bytes!("../../assets/es/voces/voces-es-419.fvox");
    static FR: &[u8] = include_bytes!("../../assets/fr/voces/voces-fr.fvox");
    match id {
        "es" => Some(ES),
        "es-419" => Some(ES_419),
        "fr" => Some(FR),
        _ => None,
    }
}
