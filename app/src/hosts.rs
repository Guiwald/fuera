//! What each platform provides: where progress is saved, vibration, speech, window settings.

use fuera_ui::Host;
use winit::window::WindowAttributes;

/// Top and bottom safe-area insets, in pixels.
pub fn safe_insets() -> (f32, f32) {
    (0.0, 0.0)
}

fn days_since_epoch(ms: f64) -> i64 {
    (ms / 86_400_000.0).floor() as i64
}

// ------------------------------------------------------------------------------------------------ desktop

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub fn window_attributes() -> WindowAttributes {
    WindowAttributes::default()
        .with_title("¡Fuera!")
        .with_inner_size(winit::dpi::LogicalSize::new(440.0, 900.0))
        .with_min_inner_size(winit::dpi::LogicalSize::new(320.0, 520.0))
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub struct DesktopHost {
    path: std::path::PathBuf,
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
impl DesktopHost {
    pub fn new() -> DesktopHost {
        let dir = std::env::var_os("FUERA_DATA")
            .map(std::path::PathBuf::from)
            .or_else(|| directories::ProjectDirs::from("fr", "guiwald", "fuera").map(|d| d.data_dir().to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let _ = std::fs::create_dir_all(&dir);
        DesktopHost { path: dir.join("progress.txt") }
    }
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
impl Default for DesktopHost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
impl Host for DesktopHost {
    fn load(&mut self) -> Option<String> {
        std::fs::read_to_string(&self.path).ok()
    }

    fn store(&mut self, text: &str) {
        let tmp = self.path.with_extension("tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, &self.path);
        }
    }

    fn locale(&self) -> String {
        sys_locale::get_locale().unwrap_or_else(|| "en".into())
    }

    fn today(&self) -> i64 {
        let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0);
        days_since_epoch(ms)
    }
}

// ------------------------------------------------------------------------------------------------ web

#[cfg(target_arch = "wasm32")]
pub fn window_attributes() -> WindowAttributes {
    use winit::platform::web::WindowAttributesExtWebSys;
    WindowAttributes::default().with_title("¡Fuera!").with_append(true)
}

#[cfg(target_arch = "wasm32")]
pub struct WebHost;

#[cfg(target_arch = "wasm32")]
impl Host for WebHost {
    fn load(&mut self) -> Option<String> {
        let s = web_sys::window()?.local_storage().ok()??;
        s.get_item("fuera-progress").ok()?
    }

    fn store(&mut self, text: &str) {
        if let Some(Ok(Some(s))) = web_sys::window().map(|w| w.local_storage()) {
            let _ = s.set_item("fuera-progress", text);
        }
    }

    fn speak(&mut self, text: &str, lang: &str) {
        let Some(w) = web_sys::window() else { return };
        let Ok(synth) = w.speech_synthesis() else { return };
        if let Ok(u) = web_sys::SpeechSynthesisUtterance::new_with_text(text) {
            u.set_lang(lang);
            u.set_rate(0.9);
            synth.cancel();
            synth.speak(&u);
        }
    }

    fn vibrate(&mut self, ms: u32) {
        if let Some(w) = web_sys::window() {
            let _ = w.navigator().vibrate_with_duration(ms);
        }
    }

    fn locale(&self) -> String {
        web_sys::window().and_then(|w| w.navigator().language()).unwrap_or_else(|| "en".into())
    }

    fn today(&self) -> i64 {
        let d = js_sys::Date::new_0();
        let local_ms = js_sys::Date::now() - d.get_timezone_offset() * 60_000.0;
        days_since_epoch(local_ms)
    }
}

// ------------------------------------------------------------------------------------------------ android

#[cfg(target_os = "android")]
pub fn window_attributes() -> WindowAttributes {
    WindowAttributes::default()
}

#[cfg(target_os = "android")]
pub struct AndroidHost {
    app: winit::platform::android::activity::AndroidApp,
    path: std::path::PathBuf,
}

#[cfg(target_os = "android")]
impl AndroidHost {
    pub fn new(app: winit::platform::android::activity::AndroidApp) -> AndroidHost {
        let dir = app.internal_data_path().unwrap_or_else(|| std::path::PathBuf::from("/data/local/tmp/fuera"));
        let _ = std::fs::create_dir_all(&dir);
        AndroidHost { path: dir.join("progress.txt"), app }
    }

    fn with_env<T>(&self, f: impl FnOnce(&mut jni::JNIEnv, &jni::objects::JObject) -> jni::errors::Result<T>) -> Option<T> {
        let vm = unsafe { jni::JavaVM::from_raw(self.app.vm_as_ptr() as *mut jni::sys::JavaVM) }.ok()?;
        let mut env = vm.attach_current_thread().ok()?;
        let activity = unsafe { jni::objects::JObject::from_raw(self.app.activity_as_ptr() as jni::sys::jobject) };
        let r = f(&mut env, &activity);
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_describe();
            let _ = env.exception_clear();
        }
        r.map_err(|e| log::warn!("JNI: {e}")).ok()
    }
}

#[cfg(target_os = "android")]
impl Host for AndroidHost {
    fn load(&mut self) -> Option<String> {
        std::fs::read_to_string(&self.path).ok()
    }

    fn store(&mut self, text: &str) {
        let tmp = self.path.with_extension("tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, &self.path);
        }
    }

    fn vibrate(&mut self, ms: u32) {
        use jni::objects::JValue;
        self.with_env(|env, activity| {
            let name = env.new_string("vibrator")?;
            let vib = env.call_method(activity, "getSystemService", "(Ljava/lang/String;)Ljava/lang/Object;", &[JValue::Object(&name)])?.l()?;
            if vib.is_null() {
                return Ok(());
            }
            let effect = env
                .call_static_method("android/os/VibrationEffect", "createOneShot", "(JI)Landroid/os/VibrationEffect;", &[JValue::Long(ms as i64), JValue::Int(-1)])?
                .l()?;
            env.call_method(&vib, "vibrate", "(Landroid/os/VibrationEffect;)V", &[JValue::Object(&effect)])?;
            Ok(())
        });
    }

    fn quit(&mut self) {
        self.with_env(|env, activity| {
            env.call_method(activity, "finish", "()V", &[])?;
            Ok(())
        });
    }

    fn locale(&self) -> String {
        sys_locale::get_locale().unwrap_or_else(|| "en".into())
    }

    fn today(&self) -> i64 {
        let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as f64).unwrap_or(0.0);
        days_since_epoch(ms)
    }
}
