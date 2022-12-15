mod events;

pub struct Reply {
	handled: bool,
}

impl Reply {
	pub fn unhandled() -> Reply {
		Reply { handled: false }
	}

	pub fn handled() -> Reply {
		Reply { handled: true }
	}
}

pub use events::*;
