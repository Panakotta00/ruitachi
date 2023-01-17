use cgmath::Vector2;

pub enum HorizontalAlignment {
	Left,
	Center,
	Right,
	Fill,
}

pub enum VerticalAlignment {
	Top,
	Center,
	Bottom,
	Fill,
}

pub enum Growth {
	Fill,
	Fit,
	Val(f32),
}

pub enum Axis {
	Vertical,
	Horizontal,
}

impl Axis {
	pub fn get_vec_axis<T>(&self, val: Vector2<T>) -> (T, T) {
		match self {
			Axis::Vertical => (val.y, val.x),
			Axis::Horizontal => (val.x, val.y),
		}
	}

	pub fn create_vec<T>(&self, axis: T, other: T) -> Vector2<T> {
		match self {
			Axis::Vertical => Vector2::new(other, axis),
			Axis::Horizontal => Vector2::new(axis, other),
		}
	}
}
