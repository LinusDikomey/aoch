use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use color_format::cwrite;
use vecm::{PolyVec2, Vec2i};

pub const DIRS4: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];
pub const DIRS8: [(i32, i32); 8] = [
    (0, -1),
    (-1, 0),
    (1, 0),
    (0, 1),
    (-1, -1),
    (1, -1),
    (-1, 1),
    (1, 1),
];

pub struct Grid<T> {
    buf: Box<[T]>,
    width: usize,
    height: usize,
}
impl<T> Grid<T> {
    pub fn from_nested(v: Vec<Vec<T>>) -> Self {
        let height = v.len();
        let width = v[0].len();
        let buf: Box<[T]> = v.into_iter().flatten().collect();
        assert_eq!(buf.len(), width * height, "mismatched buffer row lengths");
        Self { buf, width, height }
    }

    pub fn from_nested_slice(v: &[Vec<T>]) -> Self
    where
        T: Clone,
    {
        let height = v.len();
        let width = v[0].len();
        let buf: Box<[T]> = v.iter().flatten().cloned().collect();
        assert_eq!(buf.len(), width * height, "mismatched buffer row lengths");
        Self { buf, width, height }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn map<F: FnMut(T) -> U, U>(self, f: F) -> Grid<U> {
        Grid {
            buf: IntoIterator::into_iter(self.buf).map(f).collect(),
            width: self.width,
            height: self.height,
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        (0..self.height).map(|i| &self.buf[i * self.width..(i + 1) * self.width])
    }

    pub fn positions(&self) -> impl Iterator<Item = Vec2i> {
        let width = self.width;
        (0..self.height as i32).flat_map(move |y| (0..width as i32).map(move |x| Vec2i::new(x, y)))
    }

    pub fn neighbor_positions4(&self, pos: Vec2i) -> impl Iterator<Item = Vec2i> {
        let width = self.width;
        let height = self.height;
        DIRS4.into_iter().filter_map(move |(a, b)| {
            let pos = (pos.x + a, pos.y + b);
            ((0..width as i32).contains(&pos.0) && (0..height as i32).contains(&pos.1))
                .then_some(Vec2i::new(pos.0, pos.1))
        })
    }

    pub fn neighbor_positions8(&self, pos: Vec2i) -> impl Iterator<Item = Vec2i> {
        let width = self.width;
        let height = self.height;
        DIRS8.into_iter().filter_map(move |(a, b)| {
            let pos = (pos.x + a, pos.y + b);
            ((0..width as i32).contains(&pos.0) && (0..height as i32).contains(&pos.1))
                .then_some(Vec2i::new(pos.0, pos.1))
        })
    }

    pub fn pretty(&self) -> PrettyGrid<T> {
        PrettyGrid::new(self)
    }
}
impl<T: Display> Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.rows() {
            for (i, item) in row.iter().enumerate() {
                if i != 0 {
                    write!(f, " ")?;
                }
                write!(f, "{item}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
impl Grid<char> {
    pub fn from_str_chars(s: &str) -> Self {
        let mut buf = Vec::with_capacity(s.len());
        let mut height = 0;
        let mut prev_width = None;
        for line in s.lines() {
            height += 1;
            let mut width = 0;
            for c in line.chars() {
                width += 1;
                buf.push(c);
            }
            if let Some(prev) = prev_width {
                assert_eq!(prev, width, "differing widths");
            } else {
                prev_width = Some(width);
            }
        }

        Self {
            buf: buf.into_boxed_slice(),
            width: prev_width.expect("got empty grid"),
            height,
        }
    }
}
impl<T: PartialEq> Grid<T> {
    pub fn from_separated(s: impl IntoIterator<Item = T>, sep: T) -> Grid<T> {
        let s = s.into_iter();
        let mut buf = Vec::with_capacity(s.size_hint().0);
        let mut height = 1;
        let mut width = 0;
        let mut prev_width = None;
        for item in s {
            if item == sep {
                if let Some(prev) = prev_width {
                    assert_eq!(prev, width, "differing width in line {height}");
                } else {
                    prev_width = Some(width);
                }
                width = 0;
                height += 1;
                continue;
            }
            width += 1;
            buf.push(item);
        }

        let final_width = prev_width.expect("got empty grid");
        assert_eq!(final_width, width, "differing width in line {height}");
        debug_assert_eq!(buf.len(), height * prev_width.unwrap());

        Self {
            buf: buf.into_boxed_slice(),
            width: final_width,
            height,
        }
    }
}
impl Grid<u8> {
    pub fn from_str_bytes(s: &str) -> Self {
        Self::from_separated(s.as_bytes().iter().copied(), b'\n')
    }
}

impl<T, I> Index<PolyVec2<I>> for Grid<T>
where
    usize: TryFrom<I>,
{
    type Output = T;

    fn index(&self, index: PolyVec2<I>) -> &Self::Output {
        let [Ok(x), Ok(y)] = [index.x, index.y].map(usize::try_from) else {
            panic!("conversion to usize failed while indexing grid");
        };
        &self[(x, y)]
    }
}
impl<T, I> IndexMut<PolyVec2<I>> for Grid<T>
where
    usize: TryFrom<I>,
{
    fn index_mut(&mut self, index: PolyVec2<I>) -> &mut Self::Output {
        let [Ok(x), Ok(y)] = [index.x, index.y].map(usize::try_from) else {
            panic!("conversion to usize failed while indexing grid");
        };
        &mut self[(x, y)]
    }
}
impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        assert!(x < self.width, "x index out of range");
        assert!(y < self.height, "y index out of range");
        &self.buf[y * self.width + x]
    }
}
impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        assert!(x < self.width, "x index out of range");
        assert!(y < self.height, "y index out of range");
        &mut self.buf[y * self.width + x]
    }
}

pub struct PrettyGrid<'a, T> {
    grid: &'a Grid<T>,
    with_red: Option<Box<dyn Fn((usize, usize)) -> bool + 'a>>,
    with_green: Option<Box<dyn Fn((usize, usize)) -> bool + 'a>>,
}

impl<'a, T> PrettyGrid<'a, T> {
    pub fn new(grid: &'a Grid<T>) -> Self {
        Self {
            grid,
            with_red: None,
            with_green: None,
        }
    }
    pub fn with_red(mut self, f: impl Fn((usize, usize)) -> bool + 'a) -> Self {
        self.with_red = Some(Box::new(f));
        self
    }
    pub fn with_green(mut self, f: impl Fn((usize, usize)) -> bool + 'a) -> Self {
        self.with_green = Some(Box::new(f));
        self
    }
}

impl<T: Display> Display for PrettyGrid<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_cell_len = self
            .grid
            .buf
            .iter()
            .map(|i| i.to_string().len())
            .max()
            .unwrap_or(0);
        for (y, row) in self.grid.rows().enumerate() {
            for (x, item) in row.iter().enumerate() {
                let len = item.to_string().len();
                if max_cell_len > 1 {
                    write!(f, "{:<width$}", "", width = max_cell_len - len + 1)?;
                }
                if self.with_red.as_ref().map_or(false, |f| f((x, y))) {
                    cwrite!(f, "#bold<#red<{item}>>")?;
                } else if self.with_green.as_ref().map_or(false, |f| f((x, y))) {
                    cwrite!(f, "#bold<#green<{item}>>")?;
                } else {
                    cwrite!(f, "#rgb(192,192,192)<{item}>")?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_from_str() {
        let s = "abc\ndef\nghi\njkl";
        let g = Grid::from_str_chars(s);
        assert_eq!(g.width(), 3);
        assert_eq!(g.height(), 4);
        assert_eq!(g[(0, 0)], 'a');
        assert_eq!(g[(1, 2)], 'h');
        assert_eq!(g[(2, 3)], 'l');
    }

    #[test]
    fn grid_from_str_bytes() {
        let s = "abc\ndef\nghi\njkl";
        let g = Grid::from_str_bytes(s);
        assert_eq!(g.width(), 3);
        assert_eq!(g.height(), 4);
        assert_eq!(g[(0, 0)], b'a');
        assert_eq!(g[(1, 2)], b'h');
        assert_eq!(g[(2, 3)], b'l');
    }
}
