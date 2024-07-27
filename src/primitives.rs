use std::ops::{BitOr, BitOrAssign};

pub struct Pos2 {
    pub x: f64,
    pub y: f64,
}

impl Bounded2D for Pos2 {
    fn bounds(&self) -> Bounds {
        Bounds {
            x: Range::new(self.x),
            y: Range::new(self.y),
        }
    }
}

pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub struct Pos3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Pos3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Pos3 { x, y, z }
    }
}

pub struct Triangle<T> {
    pub v1: T,
    pub v2: T,
    pub v3: T,
}

impl Triangle<Pos3> {
    fn remove_z(&self) -> Triangle<Pos2> {
        Triangle {
            v1: Pos2 {
                x: self.v1.x,
                y: self.v1.y,
            },
            v2: Pos2 {
                x: self.v2.x,
                y: self.v2.y,
            },
            v3: Pos2 {
                x: self.v3.x,
                y: self.v3.y,
            },
        }
    }
}

impl<T> Triangle<T> {
    pub fn new(v1: T, v2: T, v3: T) -> Self {
        Triangle { v1, v2, v3 }
    }
}

impl Bounded2D for Triangle<Pos2> {
    fn bounds(&self) -> Bounds {
        self.v1.bounds() | self.v2.bounds() | self.v3.bounds()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl BitOr<Range> for Range {
    type Output = Range;

    fn bitor(self, rhs: Range) -> Self::Output {
        Range {
            min: f64::min(self.min, rhs.min),
            max: f64::max(self.max, rhs.max),
        }
    }
}

impl BitOrAssign<Range> for Range {
    fn bitor_assign(&mut self, rhs: Range) {
        self.min = f64::min(self.min, rhs.min);
        self.max = f64::max(self.max, rhs.max);
    }
}

impl Range {
    pub fn new(v: f64) -> Range {
        Range { min: v, max: v }
    }

    pub fn size(&self) -> f64 {
        self.max - self.min
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub x: Range,
    pub y: Range,
}

impl BitOrAssign<&Bounds> for Bounds {
    fn bitor_assign(&mut self, rhs: &Bounds) {
        self.x |= rhs.x;
        self.y |= rhs.y;
    }
}

impl Bounds {
    pub fn from_items<T>(items: &[T]) -> Option<Bounds>
    where
        T: Bounded2D,
    {
        let mut res = match items.first() {
            Some(item) => item.bounds(),
            None => return None,
        };

        for item in &items[1..] {
            res |= &item.bounds();
        }

        Some(res)
    }
}

impl BitOr<Bounds> for Bounds {
    type Output = Bounds;

    fn bitor(self, rhs: Bounds) -> Self::Output {
        Bounds {
            x: self.x | rhs.x,
            y: self.y | rhs.y,
        }
    }
}

// Any object that has 2D axis-aligned bounds
pub trait Bounded2D {
    fn bounds(&self) -> Bounds;
}

impl Bounded2D for (f64, f64) {
    fn bounds(&self) -> Bounds {
        Bounds {
            x: Range::new(self.0),
            y: Range::new(self.1),
        }
    }
}

pub struct Ray {
    pub start: Pos2,
    pub dir: Vec2,
}

#[derive(Clone, Copy)]
pub enum BoundingBox {
    Empty,
    Valid(Bounds),
}

impl BoundingBox {
    /*
    fn extend_by(&mut self, point: (f64, f64)) {
        self.min.0 = f64::min(self.min.0, point.0);
        self.min.1 = f64::min(self.min.1, point.1);
        self.max.0 = f64::max(self.max.0, point.0);
        self.max.1 = f64::max(self.max.1, point.1);
    }*/

    fn new() -> BoundingBox {
        BoundingBox::Empty
    }

    pub fn from_items<T>(items: &[T]) -> BoundingBox
    where
        T: Bounded2D,
    {
        let mut bounds = BoundingBox::Empty;
        for item in items {
            bounds = bounds | item.bounds();
        }

        bounds
    }

    fn width(&self) -> f64 {
        match self {
            BoundingBox::Empty => 0.0,
            BoundingBox::Valid(rect) => rect.x.size(),
        }
    }

    fn height(&self) -> f64 {
        match self {
            BoundingBox::Empty => 0.0,
            BoundingBox::Valid(rect) => rect.y.size(),
        }
    }
}

impl BitOrAssign<BoundingBox> for BoundingBox {
    fn bitor_assign(&mut self, rhs: BoundingBox) {
        *self = match self {
            BoundingBox::Empty => rhs,
            BoundingBox::Valid(bounds) => rhs | *bounds,
        }
    }
}

impl BitOr<BoundingBox> for BoundingBox {
    type Output = BoundingBox;

    fn bitor(self, other: Self) -> Self::Output {
        match (self, other) {
            (a, BoundingBox::Empty) => a,
            (BoundingBox::Empty, b) => b,
            (BoundingBox::Valid(lhs), BoundingBox::Valid(rhs)) => BoundingBox::Valid(Bounds {
                x: lhs.x | rhs.x,
                y: lhs.y | rhs.y,
            }),
        }
    }
}

impl BitOr<Bounds> for BoundingBox {
    type Output = BoundingBox;

    fn bitor(self, rhs: Bounds) -> Self::Output {
        match self {
            BoundingBox::Empty => BoundingBox::Valid(rhs),
            BoundingBox::Valid(lhs) => BoundingBox::Valid(lhs | rhs),
        }
    }
}
