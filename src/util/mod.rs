#![feature(coerce_unsized)]
#![feature(unsize)]

use crate::widgets::{Widget, WidgetArrangement};
use cgmath::Vector2;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::Unsize;
use std::ops::{CoerceUnsized, Deref, DerefMut};
use std::os::raw::c_void;
use std::rc::Rc;

pub struct WidgetRef<T: ?Sized>(Rc<RefCell<T>>);

type scalar = f32;

impl<T: Sized> WidgetRef<T> {
	pub fn new(val: T) -> Self {
		Self(Rc::new(RefCell::new(val)))
	}
}

impl<T: ?Sized> Clone for WidgetRef<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T, U> CoerceUnsized<WidgetRef<U>> for WidgetRef<T>
where
	T: Unsize<U> + ?Sized,
	U: ?Sized,
{}

impl<T: ?Sized, U: ?Sized> PartialEq<WidgetRef<U>> for WidgetRef<T> {
	fn eq(&self, other: &WidgetRef<U>) -> bool {
		self.0.as_ptr() as *mut u8 == other.0.as_ptr() as *mut u8
	}
}

impl<T: ?Sized> Eq for WidgetRef<T> {}

impl<T: ?Sized> Hash for WidgetRef<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write_usize(self.0.as_ptr() as *const () as usize);
	}
}

impl<T: ?Sized> Debug for WidgetRef<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0.as_ptr())
	}
}

impl<T: ?Sized> WidgetRef<T> {
	pub fn get(&self) -> RefMut<'_, T> {
		self.0.deref().borrow_mut()
	}
}

#[derive(Clone, Copy)]
pub struct Geometry {
	absolute_pos: Vector2<scalar>,
	local_pos: Vector2<scalar>,
	local_size: Vector2<scalar>,
	scale: Vector2<scalar>,
}

impl Geometry {
	pub fn new(
		local_pos: Vector2<scalar>,
		local_size: Vector2<scalar>,
		absolute_pos: Vector2<scalar>,
		scale: Vector2<scalar>,
	) -> Self {
		Self {
			absolute_pos,
			local_pos,
			local_size,
			scale,
		}
	}

	pub fn local_pos(&self) -> Vector2<scalar> {
		self.local_pos
	}

	pub fn set_local_pos(&mut self, pos: Vector2<scalar>) {
		self.local_pos = pos;
	}

	pub fn local_size(&self) -> Vector2<scalar> {
		self.local_size
	}

	pub fn set_local_size(&mut self, size: Vector2<scalar>) {
		self.local_size = size;
	}

	pub fn absolute_pos(&self) -> Vector2<scalar> {
		self.local_pos
	}

	pub fn set_absolute_pos(&mut self, pos: Vector2<scalar>) {
		self.absolute_pos = pos;
	}

	pub fn scale(&self) -> Vector2<scalar> {
		self.scale
	}

	pub fn set_scale(&mut self, scale: Vector2<scalar>) {
		self.scale = scale;
	}

	pub fn child_widget(
		&self,
		child: WidgetRef<dyn Widget>,
		pos: Vector2<scalar>,
		size: Vector2<scalar>,
	) -> WidgetArrangement {
		let local_pos = self.local_pos + pos;
		WidgetArrangement {
			widget: child,
			geometry: Geometry {
				absolute_pos: self.absolute_pos + pos,
				local_pos,
				local_size: size,
				scale: Vector2::new(0.0, 0.0),
			},
		}
	}

	pub fn contains_absolute_pos(&self, pos: &Vector2<scalar>) -> bool {
		if self.absolute_pos.x > pos.x  || self.absolute_pos.y > pos.y {
			return false;
		}
		let max = self.absolute_pos + self.local_size;
		if max.x < pos.x || max.y < pos.y {
			return false;
		}
		true
	}
}

impl Default for Geometry {
	fn default() -> Self {
		Geometry {
			absolute_pos: Vector2::new(0.0, 0.0),
			local_pos: Vector2::new(0.0, 0.0),
			local_size: Vector2::new(0.0, 0.0),
			scale: Vector2::new(0.0, 0.0),
		}
	}
}
