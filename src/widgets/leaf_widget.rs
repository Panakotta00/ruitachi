use std::cell::{Ref, RefMut};
use crate::{paint::Painter, util::Geometry, widgets::Widget};

use skia_safe::Vector;
use crate::widgets::{Arrangements, Children, WidgetArrangement, WidgetState};

#[derive(Default, Clone)]
pub struct LeafState {
    pub widget: WidgetState,
    cached_geometry: Geometry,
}

pub trait LeafWidget: Widget {
    fn leaf_state(&self) -> Ref<LeafState>;
    fn leaf_state_mut(&mut self) -> RefMut<LeafState>;

    /// Leaf implementation of [Widget.get_children()]
    ///
    /// # Default Implementation
    /// Returns an empty iterator
    fn leaf_get_children(&self) -> Children {
        Vec::new()
    }

    /// Leaf implementation of [Widget.arrange_children()]
    ///
    /// # Default Implementation
    /// Does nothing just safes the geometry as cached to leaf state
    fn leaf_arrange_children(&mut self, geometry: Geometry){
        let mut state = self.leaf_state_mut();
        state.cached_geometry = geometry;
    }

    /// Leaf Implementation of [Widget::get_arranged_children()]
    ///
    /// # Default Implementation
    /// Returns the stored widget arrangement in the panel state
    fn leaf_get_arranged_children(&self) -> Arrangements {
        Vec::new()
    }

    /// Panel Implementation of [Widget::cached_geometry()]
    ///
    /// # Default Implementation
    /// Returns the stored cached geometry in the panel state
    fn leaf_cached_geometry(&self) -> Geometry {
        let state = self.leaf_state();
        state.cached_geometry
    }
}
