use crate::events::EventContext;
use crate::platform::common::PlatformContext;
use crate::util::WidgetRef;

pub struct GUIApplication<P: PlatformContext> {
	event: EventContext,
	platform: P,
}

impl<P: PlatformContext> GUIApplication<P> {
	pub fn new(platform: P) -> Self {
		Self {
			event: EventContext::new(),
			platform,
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

impl<P: PlatformContext> Application for GUIApplication<P> {
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
