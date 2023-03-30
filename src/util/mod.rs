#![feature(coerce_unsized)]
#![feature(unsize)]

use crate::widgets::{Widget, WidgetArrangement};
use cgmath::Vector2;
use std::{
	borrow::BorrowMut,
	cell::{RefCell, RefMut},
	fmt::{Debug, Formatter},
	hash::{Hash, Hasher},
	marker::Unsize,
	ops::{CoerceUnsized},
	rc::Rc,
};
use std::cell::Ref;
use std::ops::Deref;
use std::rc::Weak;

pub type scalar = f32;

pub type WindowId = crate::platform::WindowId;

pub trait WidgetRefFromSelf {
	fn widget_weak(&self) -> WidgetWeak<dyn Widget>;

	fn widget_ref(&self) -> WidgetRef<dyn Widget> {
		self.widget_weak().upgrade().unwrap()
	}
}

pub trait WidgetRefFromSelfSpecific: WidgetRefFromSelf {
	fn set_widget_ref(&mut self, widget: &WidgetRef<Self>);
	fn self_weak(&self) -> WidgetWeak<Self>;

	fn self_ref(&self) -> WidgetRef<Self> {
		self.self_weak().upgrade().unwrap()
	}
}

/// An easy to use reference counted shared ptr.
///
/// Holds an instance of some type.
/// If cloned, only the pointer gets copied.
///
/// ```
/// # use ruitachi::util::SharedRef;
///
/// struct MyStruct(i32);
///
/// let struct1 = SharedRef::new(MyStruct(42));
/// let struct1_clone = struct1.clone();
///
/// assert_eq!(struct1_clone.get().0, 42);
/// widget1.get().0 = 69;
/// assert_eq!(struct1_clone.get().0, 69);
/// ```
pub struct SharedRef<T: ?Sized>(Rc<RefCell<T>>);

impl<T> SharedRef<T> where T: Sized {
	pub fn new(val: T) -> Self {
		Self(Rc::new(RefCell::new(val)))
	}
}

impl<T> SharedRef<T> where T: ?Sized {
	#[track_caller]
	pub fn get(&self) -> Ref<T> {
		self.0.deref().borrow()
	}

	#[track_caller]
	pub fn get_mut(&self) -> RefMut<T> {
		self.0.deref().borrow_mut()
	}

	pub fn downgrade(&self) -> SharedWeak<T> {
		SharedWeak(Rc::downgrade(&self.0))
	}
}

impl<T> Clone for SharedRef<T> where T: ?Sized {
	fn clone(&self) -> Self {
		SharedRef(self.0.clone())
	}
}


impl<T> PartialEq for SharedRef<T> where T: ?Sized {
	fn eq(&self, other: &Self) -> bool {
		self.0.as_ptr() == other.0.as_ptr()
	}
}

impl<T> Eq for SharedRef<T> where T: ?Sized {}

impl<T> Hash for SharedRef<T> where T: ?Sized {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write_usize(self.0.as_ptr() as *const () as usize);
	}
}

impl<T, U> CoerceUnsized<SharedRef<U>> for SharedRef<T>
	where
		T: Unsize<U> + ?Sized + WidgetRefFromSelf,
		U: ?Sized + WidgetRefFromSelf,
{}

/// General wrapper of SharedWeak
pub struct SharedWeak<T: ?Sized>(Weak<RefCell<T>>);

impl<T> SharedWeak<T> where T: ?Sized {
	#[track_caller]
	pub fn upgrade(&self) -> Option<SharedRef<T>> {
		self.0.upgrade().map(|v| SharedRef(v))
	}
}

impl<T> Default for SharedWeak<T> where T: Sized {
	fn default() -> Self {
		Self(Weak::new())
	}
}

impl<T> Clone for SharedWeak<T> where T: ?Sized {
	fn clone(&self) -> Self {
		SharedWeak(self.0.clone())
	}
}

impl<T, U> CoerceUnsized<SharedWeak<U>> for SharedWeak<T>
	where
		T: Unsize<U> + ?Sized + WidgetRefFromSelf,
		U: ?Sized + WidgetRefFromSelf,
{}

/// Similar to [SharedRef], but requires T to implement [WidgetRefFromSelfSpecific].
///
/// Automatically sets the self reference to the widget allowing the use of WidgetRef from self right away.
/// Should be used directly when the widget gets created and not deferred.
pub struct WidgetRef<T: ?Sized + WidgetRefFromSelf = dyn Widget>(Rc<RefCell<T>>);

impl<T> WidgetRef<T> where T: Sized + WidgetRefFromSelfSpecific {
	pub fn new(val: T) -> Self {
		let new = Self(Rc::new(RefCell::new(val)));
		new.get_mut().set_widget_ref(&new);
		new
	}
}

impl<T> WidgetRef<T> where T: ?Sized + WidgetRefFromSelf {
	#[track_caller]
	pub fn get(&self) -> Ref<T> {
		self.0.deref().borrow()
	}

	#[track_caller]
	pub fn get_mut(&self) -> RefMut<T> {
		self.0.deref().borrow_mut()
	}

	pub fn downgrade(&self) -> WidgetWeak<T> {
		WidgetWeak(Rc::downgrade(&self.0))
	}
}

impl<T> Clone for WidgetRef<T> where T: ?Sized + WidgetRefFromSelf {
	fn clone(&self) -> Self {
		WidgetRef(self.0.clone())
	}
}

impl<T> PartialEq for WidgetRef<T> where T: ?Sized + WidgetRefFromSelf {
	fn eq(&self, other: &Self) -> bool {
		self.0.as_ptr() == other.0.as_ptr()
	}
}

impl<T> Eq for WidgetRef<T> where T: ?Sized + WidgetRefFromSelf {}

impl<T> Hash for WidgetRef<T> where T: ?Sized + WidgetRefFromSelf {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write_usize(self.0.as_ptr() as *const () as usize);
	}
}

impl<T, U> CoerceUnsized<WidgetRef<U>> for WidgetRef<T>
	where
		T: Unsize<U> + ?Sized + WidgetRefFromSelf,
		U: ?Sized + WidgetRefFromSelf,
{}

/// Widget specific wrapper of WeakWrapper
pub struct WidgetWeak<T: ?Sized + WidgetRefFromSelf = dyn Widget>(Weak<RefCell<T>>);

impl<T> WidgetWeak<T> where T: ?Sized + WidgetRefFromSelf {
	#[track_caller]
	pub fn upgrade(&self) -> Option<WidgetRef<T>> {
		self.0.upgrade().map(|v| WidgetRef(v))
	}
}

impl<T> Default for WidgetWeak<T> where T: Sized + WidgetRefFromSelf {
	fn default() -> Self {
		Self(Weak::new())
	}
}

impl<T> Clone for WidgetWeak<T> where T: ?Sized + WidgetRefFromSelf {
	fn clone(&self) -> Self {
		WidgetWeak(self.0.clone())
	}
}

impl<T, U> CoerceUnsized<WidgetWeak<U>> for WidgetWeak<T>
	where
		T: Unsize<U> + ?Sized + WidgetRefFromSelf,
		U: ?Sized + WidgetRefFromSelf,
{}

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
		let local_pos = pos;
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
		if self.absolute_pos.x > pos.x || self.absolute_pos.y > pos.y {
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
