//! Shape placement (radius and center position).

use crate::area::AreaName;

/// Shape of the main figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeType {
    Circle,
    Diamond,
    Pentagon,
}

impl ShapeType {
    pub const ALL: [ShapeType; 3] = [ShapeType::Circle, ShapeType::Diamond, ShapeType::Pentagon];
    pub const COUNT: u32 = Self::ALL.len() as u32;

    pub fn from_index(index: u32) -> Self {
        Self::ALL[(index % Self::COUNT) as usize]
    }
}

/// Combination of size and direction for a shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryType {
    LargeUpLeft,
    LargeUpRight,
    LargeDownLeft,
    LargeDownRight,
    MiddleUpLeft,
    MiddleUpRight,
    MiddleDownLeft,
    MiddleDownRight,
    SmallUpLeft,
    SmallUpRight,
    SmallDownLeft,
    SmallDownRight,
}

impl GeometryType {
    pub const ALL: [GeometryType; 12] = [
        GeometryType::LargeUpLeft,
        GeometryType::LargeUpRight,
        GeometryType::LargeDownLeft,
        GeometryType::LargeDownRight,
        GeometryType::MiddleUpLeft,
        GeometryType::MiddleUpRight,
        GeometryType::MiddleDownLeft,
        GeometryType::MiddleDownRight,
        GeometryType::SmallUpLeft,
        GeometryType::SmallUpRight,
        GeometryType::SmallDownLeft,
        GeometryType::SmallDownRight,
    ];
    pub const COUNT: u32 = Self::ALL.len() as u32;

    pub fn from_index(index: u32) -> Self {
        Self::ALL[(index % Self::COUNT) as usize]
    }

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|t| *t == self)
            .expect("ALL contains every variant")
    }
}

/// Placement of a shape. Every value is a multiple of the icon size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    /// Radius. Values above 0.5 overflow the frame, but the overflow is not shown.
    pub radius: f32,
    /// Offset of the center from the icon center.
    pub center_x: f32,
    pub center_y: f32,
}

impl Geometry {
    pub const fn new(radius: f32, center_x: f32, center_y: f32) -> Self {
        Self {
            radius,
            center_x,
            center_y,
        }
    }

    pub const fn offset(self, offset_x: f32, offset_y: f32) -> Self {
        Self {
            radius: self.radius,
            center_x: self.center_x + offset_x,
            center_y: self.center_y + offset_y,
        }
    }

    /// Picks a placement for the area from a hash value.
    pub fn generate_from_hash(area: AreaName, hash: u32) -> Self {
        Self::lookup(area, GeometryType::from_index(hash))
    }

    /// Looks up the placement for an area and type.
    /// `Fg` has no table of its own, so it falls back to `BgMain`.
    pub fn lookup(area: AreaName, geometry_type: GeometryType) -> Self {
        let table = match area {
            AreaName::BgMain | AreaName::Fg => &BG_MAIN,
            AreaName::BgSub1 => &BG_SUB1,
            AreaName::BgSub2 => &BG_SUB2,
        };
        table[geometry_type.index()]
    }
}

// BgMain centers move along the diagonals.
// UpLeft has the largest radius and DownRight the smallest.
const BG_MAIN: [Geometry; 12] = [
    Geometry::new(0.9, -0.3, -0.3),
    Geometry::new(0.8, 0.3, -0.3),
    Geometry::new(0.7, -0.3, 0.3),
    Geometry::new(0.6, 0.3, 0.3),
    Geometry::new(0.46, -0.2, -0.2),
    Geometry::new(0.44, 0.2, -0.2),
    Geometry::new(0.42, -0.2, 0.2),
    Geometry::new(0.40, 0.2, 0.2),
    Geometry::new(0.36, -0.1, -0.1),
    Geometry::new(0.34, 0.1, -0.1),
    Geometry::new(0.32, -0.1, 0.1),
    Geometry::new(0.30, 0.1, 0.1),
];

// BgSub1 centers move along the diagonals and then slightly downward.
// DownRight has the largest radius and UpLeft the smallest.
const BG_SUB1: [Geometry; 12] = [
    Geometry::new(0.24, -0.25, -0.25).offset(-0.1, -0.08),
    Geometry::new(0.26, 0.25, -0.25).offset(0.1, -0.08),
    Geometry::new(0.28, -0.25, 0.25).offset(-0.1, 0.12),
    Geometry::new(0.3, 0.25, 0.25).offset(0.1, 0.12),
    Geometry::new(0.16, -0.25, -0.25).offset(-0.1, -0.08),
    Geometry::new(0.18, 0.25, -0.25).offset(0.1, -0.08),
    Geometry::new(0.20, -0.25, 0.25).offset(-0.1, 0.12),
    Geometry::new(0.22, 0.25, 0.25).offset(0.1, 0.12),
    Geometry::new(0.06, -0.25, -0.25).offset(-0.1, -0.08),
    Geometry::new(0.08, 0.25, -0.25).offset(0.1, -0.08),
    Geometry::new(0.10, -0.25, 0.25).offset(-0.1, 0.12),
    Geometry::new(0.12, 0.25, 0.25).offset(0.1, 0.12),
];

// BgSub2 centers move along the diagonals and then slightly upward.
// UpLeft has the largest radius and DownRight the smallest.
const BG_SUB2: [Geometry; 12] = [
    Geometry::new(0.3, -0.25, -0.25).offset(-0.1, -0.12),
    Geometry::new(0.28, 0.25, -0.25).offset(0.1, -0.12),
    Geometry::new(0.26, -0.25, 0.25).offset(-0.1, 0.08),
    Geometry::new(0.24, 0.25, 0.25).offset(0.1, 0.08),
    Geometry::new(0.22, -0.25, -0.25).offset(-0.1, -0.12),
    Geometry::new(0.20, 0.25, -0.25).offset(0.1, -0.12),
    Geometry::new(0.18, -0.25, 0.25).offset(-0.1, 0.08),
    Geometry::new(0.16, 0.25, 0.25).offset(0.1, 0.08),
    Geometry::new(0.12, -0.25, -0.25).offset(-0.1, -0.08),
    Geometry::new(0.10, 0.25, -0.25).offset(0.1, -0.08),
    Geometry::new(0.08, -0.25, 0.25).offset(-0.1, 0.08),
    Geometry::new(0.06, 0.25, 0.25).offset(0.1, 0.08),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_moves_center_only() {
        let g = Geometry::new(0.5, 0.1, 0.2).offset(0.1, -0.1);
        assert_eq!(g.radius, 0.5);
        assert!((g.center_x - 0.2).abs() < 1e-6);
        assert!((g.center_y - 0.1).abs() < 1e-6);
    }

    #[test]
    fn lookup_returns_table_entry() {
        let g = Geometry::lookup(AreaName::BgSub1, GeometryType::LargeDownRight);
        assert_eq!(g.radius, 0.3);
        assert!((g.center_x - 0.35).abs() < 1e-6);
        assert!((g.center_y - 0.37).abs() < 1e-6);
    }

    #[test]
    fn generate_from_hash_wraps_by_count() {
        let a = Geometry::generate_from_hash(AreaName::BgMain, 1);
        let b = Geometry::generate_from_hash(AreaName::BgMain, 13);
        assert_eq!(a, b);
    }
}
