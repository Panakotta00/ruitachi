use crate::{
	events::EventContext,
	platform::{common::PlatformContext, Context},
};

pub struct GUIApplication {
	event: EventContext,
	platform: Context,
}

impl GUIApplication {
	pub fn new() -> Self {
		Self {
			event: EventContext::new(),
			platform: crate::platform::create_platform(),
		}
	}
}

pub trait Application {
	fn event_context(&self) -> &EventContext;
	fn event_context_mut(&mut self) -> &mut EventContext;
	fn platform_context(&self) -> &dyn PlatformContext;
	fn platform_context_mut(&mut self) -> &mut dyn PlatformContext;

	fn run(&mut self);
}

impl Application for GUIApplication {
	fn event_context(&self) -> &EventContext {
		&self.event
	}

	fn event_context_mut(&mut self) -> &mut EventContext {
		&mut self.event
	}

	fn platform_context(&self) -> &dyn PlatformContext {
		&self.platform
	}

	fn platform_context_mut(&mut self) -> &mut dyn PlatformContext {
		&mut self.platform
	}

	fn run(&mut self) {
		self.platform.run(&mut self.event);
	}
}
