use crate::{
	events::{Reply, WidgetEvent},
	paint::Painter,
	util::{Geometry, WidgetRef},
};
use cgmath::Vector2;

use skia_safe::scalar;

/// Holds a widget and its respective geometry it got when last arranged.
#[derive(Clone)]
pub struct WidgetArrangement {
	pub widget: WidgetRef<dyn Widget>,
	pub geometry: Geometry,
}

impl WidgetArrangement {
	pub fn new(widget: WidgetRef<dyn Widget>, geometry: Geometry) -> Self {
		Self { widget, geometry }
	}
}

/// Boxed iterator over widgets
pub type Children<'a> = Box<dyn Iterator<Item = &'a WidgetRef<dyn Widget>> + 'a>;

/// The basic widget state and data every widget in the widget graph has.
///
/// Holds the internal state of the widget and is mostly used by the default implementations of the widget trait.
#[derive(Default)]
pub struct WidgetState {
	pub parent: Option<WidgetRef<dyn Widget>>,
	pub cached_geometry: Geometry,
	pub arranged_children: Vec<WidgetArrangement>,
}

/// A widget is the basic trait needed for any GUI "Element" to correctly interface
/// with other widgets and systems like I/O and caching.
/// A Widget represents a single "node" in a widget tree.
/// Due to this behaviour, it is necessary to support multiple "owners" and this mainly achieved
/// using a reference counted mutable reference. For easier use there is [WidgetRef].
/// Be aware that using this type of referencing can cause panic's due to multiple
/// simultaneous mutable dereferencing the reference.
/// This can be especially dangerous, if you intend to call a function of a widget,
/// that might call a function to another widget,
/// that in-turn again calls a function of the former widget.
/// Without special considerations this might lead to a panic.
/// This can happen lightly, if f.e. a child widget calls the parent widget to update something based on the children,
/// and in that same call the parent widget indeed dereferences each of its children to update something.
/// As long as you keep the child as immutable deref (which is possible as it can read the parent as
/// mutable anyway due to reference counting) you actually should be safe.
///
/// Every widget has a widget state that is used to store runtime information necessary to f.e.
/// build up the widget tree or to optimize performance.
///
/// Most of the default implementations are geared towards a typical behaviour of a widget
/// without any child widgets.
///
/// # Life Cycle
/// A Widget can be in different states along its life time.
/// These states are mostly self explanatory and dont need any specific considerations when using widgets.
/// It might be more necessary to know these, if you intend to create your own widgets including
/// composite widgets.
///
/// ## After Creating
/// When a widget gets created, it stays in a partially initialized state.
/// This state should be complete as much as possible the point of when it gets attached to a parent.
///
/// ## Attached to a Parent
/// Once the widget gets attached to the parent, its widget sate will store information accordingly.
/// This is generally done by passing the [WidgetRef] of the widget, to the parent, so that the
/// parent in turn can save the reference accordingly, it will then also call the [set_parent()]
/// function of this widget, to inform it, of its new parent.
/// Removing a widget from its parent is done the same way, but instead the [set_parent()] function
/// will be called with None as parent.
/// When the [set_parent()] function gets called in the process of adding the widget to the parent
/// is depended on the parent's implementation. This means the parent's state unstable.
/// Because of this nature, you also should not dereference the passed parent reference,
/// as its almost certain the parent it self is already dereferenced as mutable and you would cause a panic.
/// The trait provides a default implementation that simply stores the passed parent in the
/// widget state, which is then also returned by the [get_parent()] default implementation.
pub trait Widget {
	/// Returns the widget state of this widget as immutable.
	///
	/// You usually dont need this, as it is mostly for internal state handling.
	fn widget_state(&self) -> &WidgetState;

	/// Returns the widget state of this widget as mutable.
	///
	/// You usually dont need this, as it is mostly for internal state handling.
	fn widget_state_mut(&mut self) -> &mut WidgetState;

	/// Returns the parent of this widget in the widget tree.
	/// If the widget is a root element (like a window) or is not yet/anymore attached to a parent
	/// it returns None.
	///
	/// # Default Implementation
	/// Returns the parent stored in the widget state
	fn get_parent(&self) -> Option<WidgetRef<dyn Widget>> {
		self.widget_state().parent.clone()
	}

	/// Allows to change the parent of this widget.
	/// This should only be called by a widget that is about to add it to its children.
	/// The caller has to ensure the widget does not have a parent already.
	///
	/// # Default Implementation
	/// Stores the new parent in the widget state and invalidates the widget fully.
	fn set_parent(&mut self, parent: Option<WidgetRef<dyn Widget>>) {
		self.widget_state_mut().parent = parent;
	}

	/// Provides the geometry, layer and needed painter to start drawing the widget it self.
	/// Caller has to ensure the widget has already arranged its children properly before calling
	/// this function. And it's recommended to encapsulate the painter settings.
	///
	/// # Default Implementation
	/// Does nothing and just returns the layer as it was passed in.
	fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
		geometry;
		painter;
		layer
	}

	/// Returns the desired size of the widget which is mostly used in the alignment process
	/// of the children of a parent widget like panels etc.
	/// It's not guaranteed the widget will get desired space but it helps to arrange it as best as possible.
	///
	/// # Default Implementation
	/// Returns a desired size of 0.0 x 0.0
	fn get_desired_size(&self) -> Vector2<scalar> {
		Vector2::new(0.0, 0.0)
	}

	/// Allows to retrieve an boxed iterator for all children this widget has.
	///
	/// # Default Implementation
	/// Returns an empty iterator.
	fn get_children(&self) -> Children<'_> {
		Box::new(std::iter::empty())
	}

	/// Called to arrange all child widgets based on the geometry and should be returned.
	/// Not for direct use to get the arranged widgets! Use [get_arranged_children()] instead!
	///
	/// This function gets called by the system after the widget's validation state is marked as
	/// dirty layout and should rearrange all child widgets based on the passed new geometry.
	/// The arranged children should then be returned.
	/// The caller usually then stores the arranged children in the widget state.
	/// Hence this function is immutable.
	///
	/// # Default Implementation
	/// Returns an empty list.
	fn arrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
		Vec::new()
	}

	/// Retrieves the arranged children of this widget.
	///
	/// This function does not arrange the children. It just returns the cached arrangement.
	/// To rearrange the child widgets, invalidate the layout of this widget.
	///
	/// # Default Implementation
	/// Returns the cached arranged children of the widget state.
	fn get_arranged_children(&self) -> &Vec<WidgetArrangement> {
		&self.widget_state().arranged_children
	}

	/// Called by the system when an event (mostly user input) occurs.
	///
	/// # Default Implementation
	/// Replies to the event as unhandled.
	fn on_event(&mut self, _event: &WidgetEvent) -> Reply {
		Reply::unhandled()
	}

	/// Returns the geometry to which the children were arranged to last and that is used for painting.
	///
	/// # Default Implementation
	/// Returns the cached geometry from the widget state.
	fn cached_geometry(&self) -> Geometry {
		self.widget_state().cached_geometry
	}
}

impl dyn Widget {
	/// Arranges the widget's children and stores them in the widget state.
	///
	/// Mostly called by the invalidation mechanism when the widget's layout got invalidated.
	pub fn calculate_arrange_children(&mut self, geometry: Geometry) -> &Vec<WidgetArrangement> {
		let widgets = self.arrange_children(geometry);
		let state = self.widget_state_mut();
		state.cached_geometry = geometry;
		state.arranged_children = widgets;
		&state.arranged_children
	}
}
