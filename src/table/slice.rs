pub mod col;
pub mod row;
pub mod table;

use std::ops::Range;

use super::cell::CellPos;

pub type IdxRange = Range<usize>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlicePos {
    pub start: CellPos,
    pub end: CellPos,
}

impl SlicePos {
    pub fn new<A: Into<CellPos>, B: Into<CellPos>>(start: A, end: B) -> Self {
        let mut start: CellPos = start.into();
        let mut end: CellPos = end.into();

        if start.x > end.x {
            std::mem::swap(&mut start.x, &mut end.x);
        }
        if start.y > end.y {
            std::mem::swap(&mut start.y, &mut end.y);
        }

        Self { start, end }
    }
    pub fn is_inside(&self, pos: impl Into<CellPos>) -> bool {
        let p: CellPos = pos.into();
        (p.x >= self.start.x) && (p.y >= self.start.y) && (p.x < self.end.x) && (p.y < self.end.y)
    }

    pub fn is_valid_shift(&self, shift: CellPos) -> bool {
        let pos: CellPos = (self.start.x + shift.x, self.start.y + shift.y).into();
        self.is_inside(pos)
    }

    pub fn shift_to_pos(&self, shift: CellPos) -> Option<CellPos> {
        let pos: CellPos = (self.start.x + shift.x, self.start.y + shift.y).into();
        self.is_inside(pos).then_some(pos)
    }

    pub fn columns(&self) -> IdxRange {
        0..(self.end.x - self.start.x)
    }

    pub fn rows(&self) -> IdxRange {
        0..(self.end.y - self.start.y)
    }
}

impl<A: Into<CellPos>, B: Into<CellPos>> From<(A, B)> for SlicePos {
    fn from(value: (A, B)) -> Self {
        Self::new(value.0, value.1)
    }
}
