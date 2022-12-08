#![feature(coerce_unsized )]
#![feature(unsize)]

use std::borrow::{Borrow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::marker::Unsize;
use std::ops::{CoerceUnsized, Deref, DerefMut};
use std::rc::Rc;
use crate::widgets::Widget;

#[derive(Clone)]
pub struct WidgetRef<T: ?Sized>(Rc<RefCell<T>>);

impl<T: Sized> WidgetRef<T> {
    pub fn new(val: T) -> Self {
        Self(Rc::new(RefCell::new(val)))
    }
}

impl<T, U> CoerceUnsized<WidgetRef<U>> for WidgetRef<T>
    where T: Unsize<U> + ?Sized, U: ?Sized {}

impl<T: ?Sized> WidgetRef<T> {
    pub fn get(&self) -> RefMut<'_, T> {
        self.0.deref().borrow_mut()
    }
}
