mod platform;

use crate::util::WidgetRef;
use crate::widgets::Window;

pub use platform::PlatformContext;

pub enum PlatformMessage {
    NewWindow(WidgetRef<dyn Window>),
    RemoveWindow(WidgetRef<dyn Window>),
}