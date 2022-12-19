use cgmath::Vector2;
use skia_safe::scalar;
use crate::paint::Painter;
use crate::util::{Geometry, WidgetRef};
use crate::widgets::{Children, HorizontalAlignment, PanelWidget, VerticalAlignment, Widget, WidgetArrangement, WidgetState};

pub struct BoxPanel {
    widget: WidgetState,
    child: WidgetRef<dyn Widget>,
    v_align: VerticalAlignment,
    h_align: HorizontalAlignment,
    override_x: Option<scalar>,
    override_y: Option<scalar>,
}

pub struct BoxPanelBuilder(BoxPanel);

impl BoxPanelBuilder {
    pub fn v_align(mut self, v_align: VerticalAlignment) -> Self {
        self.0.v_align = v_align;
        self
    }

    pub fn h_align(mut self, h_align: HorizontalAlignment) -> Self {
        self.0.h_align = h_align;
        self
    }

    pub fn override_x(mut self, size_x: scalar) -> Self {
        self.0.override_x = Some(size_x);
        self
    }

    pub fn override_y(mut self, size_y: scalar) -> Self {
        self.0.override_y = Some(size_y);
        self
    }

    pub fn override_size(mut self, size: Vector2<scalar>) -> Self {
        self.0.override_x = Some(size.x);
        self.0.override_y = Some(size.y);
        self
    }

    pub fn build(self) -> WidgetRef<BoxPanel> {
        WidgetRef::new(self.0)
    }
}

impl BoxPanel {
    pub fn new(child: WidgetRef<dyn Widget>) -> BoxPanelBuilder {
        BoxPanelBuilder(BoxPanel{
            widget: WidgetState::default(),
            child,
            v_align: VerticalAlignment::Top,
            h_align: HorizontalAlignment::Left,
            override_x: None,
            override_y: None,
        })
    }
}

impl Widget for BoxPanel {
    fn widget_state(&self) -> &WidgetState {
        &self.widget
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.widget
    }

    fn paint(&self, geometry: Geometry, layer: i32, painter: &mut Painter) -> i32 {
        PanelWidget::paint(self, geometry, layer, painter)
    }

    fn get_children(&self) -> Children<'_> {
        Box::new(std::iter::once(&self.child))
    }

    fn get_desired_size(&self) -> Vector2<scalar> {
        let mut size = self.child.get().get_desired_size();
        if let Some(x) = self.override_x {
            size.x = x;
        }
        if let Some(y) = self.override_y {
            size.y = y;
        }
        size
    }

    fn arrange_children(&self, geometry: Geometry) -> Vec<WidgetArrangement> {
        let mut size = self.get_desired_size();
        let mut pos = Vector2::new(0.0, 0.0);
        match self.v_align {
            VerticalAlignment::Top => {}
            VerticalAlignment::Center => {
                pos.y = (geometry.local_size().y - size.y) / 2.0;
            }
            VerticalAlignment::Bottom => {
                pos.y = geometry.local_size().y - size.y;
            }
            VerticalAlignment::Fill => {
                size.y = geometry.local_size().y;
            }
        }
        match self.h_align {
            HorizontalAlignment::Left => {}
            HorizontalAlignment::Center => {
                pos.x = (geometry.local_size().x - size.x) / 2.0;
            }
            HorizontalAlignment::Right => {
                pos.x = geometry.local_size().x - size.x;
            }
            HorizontalAlignment::Fill => {
                size.x = geometry.local_size().x;
            }
        }
        vec![geometry.child_widget(self.child.clone(), pos, size)]
    }
}

impl PanelWidget for BoxPanel {}
