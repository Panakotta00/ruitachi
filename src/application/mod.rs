use std::cell::{OnceCell, Ref, RefCell, RefMut};
use std::ops::Deref;
use std::sync::{LazyLock, Mutex};
use std::thread::ThreadId;
use send_wrapper::SendWrapper;
use crate::{
	events::EventContext,
	platform::{common::PlatformContext, Context},
};
use crate::platform::common::PlatformMessage;
use crate::util::WidgetRef;
use crate::widgets::Window;

pub struct GUIApplication {
	event: RefCell<EventContext>,
	platform: RefCell<Context>,
}

static mut INSTANCE: Option<SendWrapper<GUIApplication>> = None;

impl GUIApplication {
	/// Returns the instance of the GUI Application.
	/// This instance can only be created once at first call and only for a single thread.
	/// Consecutive calls within the same thread are possible,
	/// but if it already got created in one thread and you call it from another, it panics.
	pub fn get() -> &'static GUIApplication {
		unsafe {
			INSTANCE.get_or_insert_with(|| SendWrapper::new(GUIApplication {
				event: RefCell::new(EventContext::new()),
				platform: RefCell::new(crate::platform::create_platform()),
			}))
		}
	}

	pub fn add_window(&self, window: WidgetRef<dyn Window>) {
		self.platform.borrow().message(PlatformMessage::NewWindow(window));
	}

	pub fn remove_window(&self, window: WidgetRef<dyn Window>) {
		self.platform.borrow().message(PlatformMessage::RemoveWindow(window));
	}

	pub fn event_context(&self) -> Ref<EventContext> {
		self.event.borrow()
	}

	pub fn event_context_mut(&self) -> RefMut<EventContext> {
		self.event.borrow_mut()
	}

	pub fn platform_context(&self) -> Ref<dyn PlatformContext> {
		self.platform.borrow()
	}

	pub fn platform_context_mut(&self) -> RefMut<dyn PlatformContext> {
		self.platform.borrow_mut()
	}

	pub fn run(&self) {
		PlatformContext::run(&self.platform,&self.event);
	}
}
