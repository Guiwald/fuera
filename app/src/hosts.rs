//! What each platform provides: where progress is saved, vibration, speech, window settings.

use fuera_ui::Host;
use winit::window::WindowAttributes;

/// Top and bottom safe-area insets, in pixels.
pub fn safe_insets() -> (f32, f32) {
    (0.0, 0.0)
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
    speech: Option<std::process::Child>,
    /// macOS: name of an installed voice for each locale asked for (es-ES, es-MX, fr-FR), looked up once.
    #[cfg(target_os = "macos")]
    voices: std::collections::HashMap<String, Option<String>>,
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
impl DesktopHost {
    pub fn new() -> DesktopHost {
        let dir = std::env::var_os("FUERA_DATA")
            .map(std::path::PathBuf::from)
            .or_else(|| directories::ProjectDirs::from("fr", "guiwald", "fuera").map(|d| d.data_dir().to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let _ = std::fs::create_dir_all(&dir);
        DesktopHost {
            path: dir.join("progress.txt"),
            speech: None,
            #[cfg(target_os = "macos")]
            voices: Default::default(),
        }
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
        now_days_local().floor() as i64
    }

    fn now_days(&self) -> f64 {
        now_days_local()
    }

    /// Speech through the system's own voices: `say` (macOS), System.Speech (Windows), espeak-ng (Linux), in the
    /// language of the course (`es-ES`, `es-MX`, `fr-FR`; the nearest voice of the same language otherwise).
    fn speak(&mut self, text: &str, lang: &str) {
        if let Some(mut c) = self.speech.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        let (l, region) = lang.split_once('-').unwrap_or((lang, ""));
        #[cfg(target_os = "macos")]
        {
            if !self.voices.contains_key(lang) {
                let out = std::process::Command::new("say").args(["-v", "?"]).output().ok();
                let list = out.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
                // Lines like "Mónica              es_ES    # Hola…": take the name before the locale.
                let pick = |loc: &str| {
                    list.lines().find(|x| x.contains(loc)).map(|x| x.split(loc).next().unwrap_or("").trim().to_string()).filter(|n| !n.is_empty())
                };
                let mut v = pick(&format!(" {l}_{region}"));
                if v.is_none() && l == "es" && region != "ES" {
                    v = pick(" es_US").or_else(|| pick(" es_419"));
                }
                let v = v.or_else(|| pick(&format!(" {l}_")));
                self.voices.insert(lang.to_string(), v);
            }
            if let Some(Some(v)) = self.voices.get(lang) {
                self.speech = std::process::Command::new("say").args(["-v", v, "-r", "165", text]).spawn().ok();
            }
        }
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let t = text.replace('\'', "''");
            let script = format!(
                "Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $all = $s.GetInstalledVoices(); $v = $all | Where-Object {{ $_.VoiceInfo.Culture.Name -eq '{lang}' }} | Select-Object -First 1; if (-not $v) {{ $v = $all | Where-Object {{ $_.VoiceInfo.Culture.Name -like '{l}-*' }} | Select-Object -First 1 }}; if ($v) {{ $s.SelectVoice($v.VoiceInfo.Name); $s.Rate = -1; $s.Speak('{t}') }}"
            );
            self.speech = std::process::Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]).creation_flags(0x0800_0000).spawn().ok();
        }
        #[cfg(target_os = "linux")]
        {
            let voice = if l == "es" && region != "ES" { "es-419" } else { l };
            self.speech = std::process::Command::new("espeak-ng")
                .args(["-v", voice, "-s", "150", text])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .or_else(|_| std::process::Command::new("spd-say").args(["-l", l, text]).spawn())
                .ok();
        }
        let _ = (l, region);
    }

    /// Opens a link (the feedback mail) with the system.
    fn open_url(&mut self, url: &str) {
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(url).spawn();
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let _ = std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", url]).creation_flags(0x0800_0000).spawn();
        }
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Local time in days since 1970-01-01 (with the fraction of the day).
#[cfg(not(target_arch = "wasm32"))]
fn now_days_local() -> f64 {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
    // The local offset is not known without a time-zone library; UTC days are close enough for streaks and reviews.
    secs / 86_400.0
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
    fn platform(&self) -> &'static str {
        "web"
    }

    /// `navigator.share` where the browser has it (phones): the system's share sheet.
    fn share(&mut self, subject: &str, body: &str) -> bool {
        use wasm_bindgen::JsCast;
        let Some(w) = web_sys::window() else { return false };
        let nav = w.navigator();
        let Ok(f) = js_sys::Reflect::get(&nav, &"share".into()) else { return false };
        let Some(f) = f.dyn_ref::<js_sys::Function>() else { return false };
        let data = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&data, &"title".into(), &subject.into());
        let _ = js_sys::Reflect::set(&data, &"text".into(), &body.into());
        match f.call1(&nav, &data) {
            Ok(p) => {
                // The promise rejects when the player closes the sheet: nothing to do.
                if let Ok(p) = p.dyn_into::<js_sys::Promise>() {
                    let ignore = wasm_bindgen::closure::Closure::<dyn FnMut(wasm_bindgen::JsValue)>::new(|_| {});
                    let _ = p.catch(&ignore);
                    ignore.forget();
                }
                true
            }
            Err(_) => false,
        }
    }

    fn open_url(&mut self, url: &str) {
        if let Some(w) = web_sys::window() {
            let _ = w.location().set_href(url);
        }
    }

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
        self.now_days().floor() as i64
    }

    fn now_days(&self) -> f64 {
        let d = js_sys::Date::new_0();
        let local_ms = js_sys::Date::now() - d.get_timezone_offset() * 60_000.0;
        local_ms / 86_400_000.0
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
    /// android.speech.tts.TextToSpeech, created at start-up so that it is ready when needed.
    tts: Option<jni::objects::GlobalRef>,
}

#[cfg(target_os = "android")]
impl AndroidHost {
    pub fn new(app: winit::platform::android::activity::AndroidApp) -> AndroidHost {
        let dir = app.internal_data_path().unwrap_or_else(|| std::path::PathBuf::from("/data/local/tmp/fuera"));
        let _ = std::fs::create_dir_all(&dir);
        let mut h = AndroidHost { path: dir.join("progress.txt"), app, tts: None };
        h.tts = h.with_env(|env, activity| {
            // No listener: TextToSpeech checks for null before calling it.
            let tts = env.new_object(
                "android/speech/tts/TextToSpeech",
                "(Landroid/content/Context;Landroid/speech/tts/TextToSpeech$OnInitListener;)V",
                &[jni::objects::JValue::Object(activity), jni::objects::JValue::Object(&jni::objects::JObject::null())],
            )?;
            env.new_global_ref(tts)
        });
        h
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

    fn platform(&self) -> &'static str {
        "android"
    }

    fn speak(&mut self, text: &str, lang: &str) {
        use jni::objects::{JObject, JValue};
        let Some(tts) = self.tts.clone() else { return };
        let (l, region) = lang.split_once('-').unwrap_or((lang, ""));
        self.with_env(|env, _| {
            let ll = env.new_string(l)?;
            let cc = env.new_string(region)?;
            let locale = env.new_object("java/util/Locale", "(Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&ll), JValue::Object(&cc)])?;
            let _ = env.call_method(tts.as_obj(), "setLanguage", "(Ljava/util/Locale;)I", &[JValue::Object(&locale)])?;
            let _ = env.call_method(tts.as_obj(), "setSpeechRate", "(F)I", &[JValue::Float(0.9)])?;
            let t = env.new_string(text)?;
            let id = env.new_string("fuera")?;
            env.call_method(
                tts.as_obj(),
                "speak",
                "(Ljava/lang/CharSequence;ILandroid/os/Bundle;Ljava/lang/String;)I",
                &[JValue::Object(&t), JValue::Int(0), JValue::Object(&JObject::null()), JValue::Object(&id)],
            )?;
            Ok(())
        });
    }

    fn quit(&mut self) {
        self.with_env(|env, activity| {
            env.call_method(activity, "finish", "()V", &[])?;
            Ok(())
        });
    }

    /// The system's share sheet (ACTION_SEND, text): the player picks WhatsApp, a mail app… and a person.
    fn share(&mut self, subject: &str, body: &str) -> bool {
        use jni::objects::{JObject, JValue};
        self.with_env(|env, activity| {
            let action = env.new_string("android.intent.action.SEND")?;
            let intent = env.new_object("android/content/Intent", "(Ljava/lang/String;)V", &[JValue::Object(&action)])?;
            let mime = env.new_string("text/plain")?;
            env.call_method(&intent, "setType", "(Ljava/lang/String;)Landroid/content/Intent;", &[JValue::Object(&mime)])?;
            for (k, v) in [("android.intent.extra.SUBJECT", subject), ("android.intent.extra.TEXT", body)] {
                let k = env.new_string(k)?;
                let v = env.new_string(v)?;
                env.call_method(&intent, "putExtra", "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;", &[JValue::Object(&k), JValue::Object(&v)])?;
            }
            let title = env.new_string(subject)?;
            let chooser = env
                .call_static_method("android/content/Intent", "createChooser", "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;", &[JValue::Object(&intent), JValue::Object(&title)])?
                .l()?;
            env.call_method(activity, "startActivity", "(Landroid/content/Intent;)V", &[JValue::Object(&chooser)])?;
            let _ = JObject::null();
            Ok(())
        })
        .is_some()
    }

    fn open_url(&mut self, url: &str) {
        use jni::objects::JValue;
        self.with_env(|env, activity| {
            let s = env.new_string(url)?;
            let uri = env.call_static_method("android/net/Uri", "parse", "(Ljava/lang/String;)Landroid/net/Uri;", &[JValue::Object(&s)])?.l()?;
            let action = env.new_string("android.intent.action.VIEW")?;
            let intent = env.new_object("android/content/Intent", "(Ljava/lang/String;Landroid/net/Uri;)V", &[JValue::Object(&action), JValue::Object(&uri)])?;
            env.call_method(activity, "startActivity", "(Landroid/content/Intent;)V", &[JValue::Object(&intent)])?;
            Ok(())
        });
    }

    fn locale(&self) -> String {
        sys_locale::get_locale().unwrap_or_else(|| "en".into())
    }

    fn today(&self) -> i64 {
        now_days_local().floor() as i64
    }

    fn now_days(&self) -> f64 {
        now_days_local()
    }
}
