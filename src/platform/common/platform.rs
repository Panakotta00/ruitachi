use crate::{events::EventContext, util::WidgetRef, widgets::Window};

pub trait PlatformContext {
	fn add_window(&mut self, window: &WidgetRef<dyn Window>);
	fn remove_window(&mut self, window: &WidgetRef<dyn crate::widgets::Window>) -> bool;
	fn run(&mut self, event_context: &mut EventContext);

	fn set_capture_cursor(&mut self, cursor: usize, should_capture: bool);
}
