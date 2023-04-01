use std::cell::RefCell;
use crate::{events::EventContext, util::WidgetRef, widgets::Window};
use crate::platform::common::PlatformMessage;

pub trait PlatformContext {
	fn message(&self, message: PlatformMessage);
	fn run(this: &RefCell<Self>, event_context: &RefCell<EventContext>) where Self: Sized;

	fn set_capture_cursor(&mut self, cursor: usize, should_capture: bool);
}
