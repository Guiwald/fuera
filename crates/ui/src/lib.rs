//! ¡Fuera! — everything the player sees and hears, independent of the platform: the host (desktop window, Android
//! activity, web page) feeds pointer events and time, and gets back a picture and sounds to play.

pub mod app;
pub mod board;
pub mod gfx;
pub mod i18n;
pub mod icon;
pub mod icons;
pub mod save;
pub mod sfx;
pub mod text;
pub mod theme;

pub use app::{App, Host, KeyEvent, PointerEvent};
