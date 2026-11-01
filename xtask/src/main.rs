//! Build helpers: `cargo xtask <command>`.
//!
//! - `icons`                     app icons (PNG, ICO, ICNS) into assets/icons
//! - `apk`                       Android APK (no Gradle, no Java) into dist/
//! - `web [out]`                 web version (wasm + page) into dist/web (or `out`)
//! - `bundle-mac <bin> [out]`    Fuera.app (and the .dmg on macOS)
//! - `dist-windows [exe]`        Windows zip

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

type Res = Result<(), Box<dyn std::error::Error>>;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(|s| s.as_str()) {
        Some("icons") => icons(),
        Some("apk") => apk(),
        Some("web") => web(args.get(1).map(PathBuf::from)),
        Some("bundle-mac") => bundle_mac(args.get(1).map(PathBuf::from), args.get(2).map(PathBuf::from)),
        Some("dist-windows") => dist_windows(args.get(1).map(PathBuf::from)),
        _ => {
            eprintln!("usage: cargo xtask icons | apk | web [out] | bundle-mac <bin> [out] | dist-windows [exe]");
            std::process::exit(2);
        }
    };
    if let Err(e) = r {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn version() -> String {
    let t = std::fs::read_to_string(root().join("Cargo.toml")).unwrap_or_default();
    t.lines()
        .skip_while(|l| !l.starts_with("[workspace.package]"))
        .find_map(|l| l.strip_prefix("version = ").map(|v| v.trim_matches('"').to_string()))
        .unwrap_or_else(|| "0.0.0".into())
}

fn version_code(v: &str) -> u32 {
    let p: Vec<u32> = v.split('.').map(|x| x.parse().unwrap_or(0)).collect();
    p.first().unwrap_or(&0) * 10000 + p.get(1).unwrap_or(&0) * 100 + p.get(2).unwrap_or(&0)
}

fn run(cmd: &mut Command) -> Res {
    let st = cmd.status().map_err(|e| format!("{cmd:?}: {e}"))?;
    if !st.success() {
        return Err(format!("failed: {cmd:?}").into());
    }
    Ok(())
}

fn png(size: u32, rounded: bool) -> Vec<u8> {
    fuera_ui::icon::app_icon(size, rounded).encode_png().expect("png")
}

fn icons() -> Res {
    let out = root().join("assets/icons");
    std::fs::create_dir_all(&out)?;
    for size in [16, 32, 64, 128, 180, 192, 256, 512, 1024] {
        std::fs::write(out.join(format!("icon-{size}.png")), png(size, true))?;
    }
    // ICO: PNG entries.
    let sizes = [16u32, 32, 64, 256];
    let pngs: Vec<Vec<u8>> = sizes.iter().map(|&s| png(s, true)).collect();
    let mut ico = vec![0, 0, 1, 0];
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (s, p) in sizes.iter().zip(&pngs) {
        let b = if *s >= 256 { 0 } else { *s as u8 };
        ico.extend_from_slice(&[b, b, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(p.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += p.len() as u32;
    }
    for p in &pngs {
        ico.extend_from_slice(p);
    }
    std::fs::write(out.join("fuera.ico"), ico)?;
    // ICNS: ic07 (128), ic08 (256), ic09 (512), ic10 (1024).
    let mut body = Vec::new();
    for (tag, s) in [(b"ic07", 128u32), (b"ic08", 256), (b"ic09", 512), (b"ic10", 1024)] {
        let p = png(s, true);
        body.extend_from_slice(tag);
        body.extend_from_slice(&((p.len() + 8) as u32).to_be_bytes());
        body.extend_from_slice(&p);
    }
    let mut icns = b"icns".to_vec();
    icns.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
    icns.extend_from_slice(&body);
    std::fs::write(out.join("fuera.icns"), icns)?;
    std::fs::write(out.join("fuera.rc"), "1 ICON \"fuera.ico\"\n")?;
    println!("icons in {}", out.display());
    Ok(())
}

// ------------------------------------------------------------------------------------------------ Android

const TARGET: &str = "aarch64-linux-android";
const MIN_SDK: u32 = 26;
const TARGET_SDK: u32 = 34;

fn newest_dir(dir: &Path, pred: impl Fn(&str) -> bool) -> Option<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).filter(|p| p.is_dir() && p.file_name().is_some_and(|n| pred(&n.to_string_lossy()))).collect();
    v.sort();
    v.pop()
}

fn apk() -> Res {
    let root = root();
    let v = version();
    let code = version_code(&v);
    let home = std::env::var_os("ANDROID_HOME").or_else(|| std::env::var_os("ANDROID_SDK_ROOT")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/opt/android-sdk"));
    let build_tools = newest_dir(&home.join("build-tools"), |_| true).ok_or("build-tools not found (ANDROID_HOME)")?;
    let platform = newest_dir(&home.join("platforms"), |n| n.starts_with("android-")).ok_or("Android platform not found")?;
    let ndk = std::env::var_os("ANDROID_NDK_HOME").map(PathBuf::from).or_else(|| newest_dir(&home.join("ndk"), |_| true)).ok_or("NDK not found (ANDROID_NDK_HOME)")?;
    let host = if cfg!(target_os = "macos") { "darwin-x86_64" } else if cfg!(target_os = "windows") { "windows-x86_64" } else { "linux-x86_64" };
    let ndk_bin = ndk.join("toolchains/llvm/prebuilt").join(host).join("bin");
    let clang = ndk_bin.join(format!("{TARGET}{MIN_SDK}-clang"));
    let et = TARGET.replace('-', "_");
    run(Command::new(env!("CARGO"))
        .args(["build", "--release", "-p", "fuera", "--lib", "--target", TARGET])
        .current_dir(&root)
        .env(format!("CARGO_TARGET_{}_LINKER", et.to_uppercase()), &clang)
        .env(format!("CC_{et}"), &clang)
        .env(format!("AR_{et}"), ndk_bin.join("llvm-ar")))?;
    let so = root.join("target").join(TARGET).join("release/libfuera.so");
    let work = root.join("target/apk");
    let _ = std::fs::remove_dir_all(&work);
    let res = work.join("res");
    std::fs::create_dir_all(res.join("values"))?;
    let manifest = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="fr.guiwald.fuera"
    android:versionCode="{code}"
    android:versionName="{v}">
    <uses-permission android:name="android.permission.VIBRATE" />
    <application
        android:label="@string/app_name"
        android:icon="@mipmap/ic_launcher"
        android:hasCode="false"
        android:allowBackup="true"
        android:theme="@android:style/Theme.DeviceDefault.NoActionBar">
        <activity
            android:name="android.app.NativeActivity"
            android:exported="true"
            android:launchMode="singleTask"
            android:screenOrientation="sensorPortrait"
            android:configChanges="orientation|screenSize|screenLayout|smallestScreenSize|keyboard|keyboardHidden|navigation|uiMode|density|fontScale|locale|layoutDirection">
            <meta-data android:name="android.app.lib_name" android:value="fuera" />
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
"#
    );
    std::fs::write(work.join("AndroidManifest.xml"), manifest)?;
    std::fs::write(res.join("values/strings.xml"), "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<resources><string name=\"app_name\">¡Fuera!</string></resources>\n")?;
    for (dpi, size) in [("mdpi", 48), ("hdpi", 72), ("xhdpi", 96), ("xxhdpi", 144), ("xxxhdpi", 192)] {
        let d = res.join(format!("mipmap-{dpi}"));
        std::fs::create_dir_all(&d)?;
        std::fs::write(d.join("ic_launcher.png"), png(size, true))?;
    }
    let aapt2 = build_tools.join("aapt2");
    run(Command::new(&aapt2).arg("compile").arg("--dir").arg(&res).arg("-o").arg(work.join("res.zip")))?;
    let base = work.join("base.apk");
    run(Command::new(&aapt2)
        .arg("link")
        .arg("-o")
        .arg(&base)
        .arg("-I")
        .arg(platform.join("android.jar"))
        .arg("--manifest")
        .arg(work.join("AndroidManifest.xml"))
        .args(["--min-sdk-version", &MIN_SDK.to_string(), "--target-sdk-version", &TARGET_SDK.to_string()])
        .args(["--version-code", &code.to_string(), "--version-name", &v])
        .arg(work.join("res.zip")))?;
    let unaligned = work.join("unaligned.apk");
    {
        let mut src = zip::ZipArchive::new(std::fs::File::open(&base)?)?;
        let mut dst = zip::ZipWriter::new(std::fs::File::create(&unaligned)?);
        for i in 0..src.len() {
            dst.raw_copy_file(src.by_index(i)?)?;
        }
        let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        dst.start_file("lib/arm64-v8a/libfuera.so", stored)?;
        let mut bytes = Vec::new();
        std::fs::File::open(&so)?.read_to_end(&mut bytes)?;
        dst.write_all(&bytes)?;
        dst.finish()?;
    }
    let aligned = work.join("aligned.apk");
    run(Command::new(build_tools.join("zipalign")).args(["-f", "-P", "16", "4"]).arg(&unaligned).arg(&aligned))?;
    let (ks, pass) = match (std::env::var_os("FUERA_KEYSTORE"), std::env::var("FUERA_KEYSTORE_PASS")) {
        (Some(k), Ok(p)) => (PathBuf::from(k), p),
        _ => {
            eprintln!("FUERA_KEYSTORE not set: signing with a throw-away debug key");
            let ks = work.join("debug.p12");
            run(Command::new("keytool")
                .args(["-genkeypair", "-keystore"])
                .arg(&ks)
                .args(["-storetype", "PKCS12", "-storepass", "android", "-keypass", "android", "-alias", "debug", "-keyalg", "RSA", "-keysize", "2048", "-validity", "3650", "-dname", "CN=Fuera debug"]))?;
            (ks, "android".to_string())
        }
    };
    let dist = root.join("dist");
    std::fs::create_dir_all(&dist)?;
    let out = dist.join(format!("Fuera-{v}.apk"));
    run(Command::new(build_tools.join("apksigner"))
        .args(["sign", "--ks"])
        .arg(&ks)
        .args(["--ks-pass", "env:FUERA_APKSIGNER_PASS", "--key-pass", "env:FUERA_APKSIGNER_PASS", "--out"])
        .arg(&out)
        .arg(&aligned)
        .env("FUERA_APKSIGNER_PASS", &pass))?;
    let _ = std::fs::remove_file(out.with_extension("apk.idsig"));
    println!("{}", out.display());
    Ok(())
}

// ------------------------------------------------------------------------------------------------ web

fn web(out: Option<PathBuf>) -> Res {
    let root = root();
    let out = out.unwrap_or_else(|| root.join("dist/web"));
    run(Command::new(env!("CARGO")).args(["build", "--release", "--config", "profile.release.strip=false", "-p", "fuera", "--lib", "--target", "wasm32-unknown-unknown"]).current_dir(&root))?;
    std::fs::create_dir_all(&out)?;
    let wasm = root.join("target/wasm32-unknown-unknown/release/fuera.wasm");
    let bindgen = std::env::var_os("WASM_BINDGEN").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("wasm-bindgen"));
    run(Command::new(bindgen).args(["--target", "web", "--no-typescript", "--out-dir"]).arg(&out).arg(&wasm))?;
    if let Ok(opt) = which("wasm-opt") {
        let f = out.join("fuera_bg.wasm");
        let _ = run(Command::new(opt).args(["-Oz", "--enable-bulk-memory", "--enable-nontrapping-float-to-int", "-o"]).arg(&f).arg(&f));
    }
    let v = version();
    std::fs::write(out.join("index.html"), WEB_PAGE.replace("{v}", &v))?;
    // Recorded pronunciations, fetched by the page in the background.
    std::fs::copy(root.join("assets/es/voces/voces-es.fvox"), out.join("voces-es.fvox"))?;
    std::fs::write(out.join("icon-192.png"), png(192, true))?;
    std::fs::write(out.join("icon-512.png"), png(512, true))?;
    std::fs::write(
        out.join("manifest.webmanifest"),
        r##"{"name":"¡Fuera!","short_name":"¡Fuera!","start_url":".","display":"standalone","orientation":"portrait","background_color":"#0C3352","theme_color":"#0C3352","icons":[{"src":"icon-192.png","sizes":"192x192","type":"image/png"},{"src":"icon-512.png","sizes":"512x512","type":"image/png"}]}"##,
    )?;
    println!("{}", out.display());
    Ok(())
}

fn which(name: &str) -> Result<PathBuf, ()> {
    std::env::var_os("PATH").and_then(|p| std::env::split_paths(&p).map(|d| d.join(name)).find(|f| f.exists())).ok_or(())
}

const WEB_PAGE: &str = r##"<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover">
<meta name="theme-color" content="#0C3352">
<title>¡Fuera!</title>
<link rel="manifest" href="manifest.webmanifest">
<link rel="icon" href="icon-192.png">
<style>
  html, body { margin: 0; height: 100%; background: #0C3352; overflow: hidden; touch-action: none; overscroll-behavior: none; }
  canvas { display: block; width: 100vw; height: 100vh; height: 100dvh; outline: none; touch-action: none; }
  #loading { position: fixed; inset: 0; display: flex; align-items: center; justify-content: center; color: #fff; font: 600 20px system-ui, sans-serif; }
</style>
</head>
<body>
<div id="loading">¡Fuera! …</div>
<script type="module">
  import init from "./fuera.js?v={v}";
  init({ module_or_path: "./fuera_bg.wasm?v={v}" }).then(() => {
    document.getElementById("loading").remove();
    const c = document.querySelector("canvas");
    if (c) c.focus();
  });
</script>
</body>
</html>
"##;

// ------------------------------------------------------------------------------------------------ desktop

fn bundle_mac(binary: Option<PathBuf>, out_dir: Option<PathBuf>) -> Res {
    let root = root();
    let v = version();
    let binary = binary.ok_or("missing binary")?;
    let out_dir = out_dir.unwrap_or_else(|| root.join("dist"));
    let app = out_dir.join("Fuera.app");
    let _ = std::fs::remove_dir_all(&app);
    let contents = app.join("Contents");
    std::fs::create_dir_all(contents.join("MacOS"))?;
    std::fs::create_dir_all(contents.join("Resources"))?;
    std::fs::copy(&binary, contents.join("MacOS/fuera"))?;
    std::fs::copy(root.join("assets/icons/fuera.icns"), contents.join("Resources/fuera.icns"))?;
    let build = version_code(&v);
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>Fuera</string>
    <key>CFBundleDisplayName</key><string>¡Fuera!</string>
    <key>CFBundleIdentifier</key><string>fr.guiwald.fuera</string>
    <key>CFBundleVersion</key><string>{build}</string>
    <key>CFBundleShortVersionString</key><string>{v}</string>
    <key>CFBundleExecutable</key><string>fuera</string>
    <key>CFBundleIconFile</key><string>fuera</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>LSApplicationCategoryType</key><string>public.app-category.puzzle-games</string>
    <key>NSHumanReadableCopyright</key><string>MIT — github.com/Guiwald/fuera</string>
</dict>
</plist>
"#
    );
    std::fs::write(contents.join("Info.plist"), plist)?;
    println!("{}", app.display());
    #[cfg(target_os = "macos")]
    {
        run(Command::new("codesign").args(["--force", "--deep", "--sign", "-"]).arg(&app))?;
        let dmg = out_dir.join(format!("Fuera-{v}.dmg"));
        let _ = std::fs::remove_file(&dmg);
        let stage = out_dir.join("dmg-stage");
        let _ = std::fs::remove_dir_all(&stage);
        std::fs::create_dir_all(&stage)?;
        run(Command::new("cp").arg("-R").arg(&app).arg(&stage))?;
        std::os::unix::fs::symlink("/Applications", stage.join("Applications"))?;
        run(Command::new("hdiutil").args(["create", "-volname", "Fuera", "-fs", "HFS+", "-format", "UDZO", "-srcfolder"]).arg(&stage).arg(&dmg))?;
        let _ = std::fs::remove_dir_all(&stage);
        println!("{}", dmg.display());
    }
    Ok(())
}

fn dist_windows(exe: Option<PathBuf>) -> Res {
    let root = root();
    let v = version();
    let exe = match exe {
        Some(e) => e,
        None => {
            // Cross-compile first, so the zip never holds a stale build.
            run(Command::new(env!("CARGO")).args(["build", "--release", "-p", "fuera", "--bin", "fuera", "--target", "x86_64-pc-windows-gnu"]).current_dir(&root))?;
            root.join("target/x86_64-pc-windows-gnu/release/fuera.exe")
        }
    };
    std::fs::create_dir_all(root.join("dist"))?;
    let out = root.join("dist").join(format!("Fuera-{v}-windows-x64.zip"));
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&out)?);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let dir = format!("Fuera-{v}");
    zip.start_file(format!("{dir}/Fuera.exe"), opts)?;
    zip.write_all(&std::fs::read(&exe)?)?;
    zip.start_file(format!("{dir}/LISEZMOI-README.txt"), opts)?;
    zip.write_all(README_WINDOWS.replace("{v}", &v).as_bytes())?;
    zip.start_file(format!("{dir}/LICENSE.txt"), opts)?;
    zip.write_all(&std::fs::read(root.join("LICENSE"))?)?;
    zip.finish()?;
    println!("{}", out.display());
    Ok(())
}

const README_WINDOWS: &str = "¡Fuera! {v}\r\n\r\nFR : double-clique sur Fuera.exe. Si Windows affiche « Windows a protégé votre ordinateur », clique sur « Informations complémentaires » puis « Exécuter quand même » (le programme n'est pas signé).\r\n\r\nEN: double-click Fuera.exe. If Windows shows \"Windows protected your PC\", click \"More info\" then \"Run anyway\" (the program is not signed).\r\n\r\nhttps://github.com/Guiwald/fuera\r\n";
