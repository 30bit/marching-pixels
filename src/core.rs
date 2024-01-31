use ::core::{iter::FusedIterator, mem};

type Primitive = u16;

const EMPTY: Primitive = 0;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct Cell(Primitive);

impl Cell {
    pub const EMPTY: Self = Self(EMPTY);

    const fn _as_first_pass_symbol(self) -> char {
        match self.0 {
            EMPTY => '░',
            BOTTOM_LEFT => '▖',
            BOTTOM_RIGHT => '▗',
            TOP_RIGHT => '▝',
            TOP_LEFT => '▘',
            kind if kind == BOTTOM_LEFT | BOTTOM_RIGHT => '▄',
            kind if kind == BOTTOM_RIGHT | TOP_RIGHT => '▐',
            kind if kind == TOP_RIGHT | TOP_LEFT => '▀',
            kind if kind == TOP_LEFT | BOTTOM_LEFT => '▌',
            kind if kind == BOTTOM_LEFT | TOP_RIGHT => '▞',
            kind if kind == BOTTOM_RIGHT | TOP_LEFT => '▚',
            kind if kind == BOTTOM_LEFT | BOTTOM_RIGHT | TOP_RIGHT => '▟',
            kind if kind == BOTTOM_RIGHT | TOP_RIGHT | TOP_LEFT => '▜',
            kind if kind == TOP_RIGHT | TOP_LEFT | BOTTOM_LEFT => '▛',
            kind if kind == TOP_LEFT | BOTTOM_LEFT | BOTTOM_RIGHT => '▙',
            kind if kind == BOTTOM_LEFT | BOTTOM_RIGHT | TOP_RIGHT | TOP_LEFT => '█',
            _ => char::REPLACEMENT_CHARACTER,
        }
    }
}

impl Default for Cell {
    #[inline]
    fn default() -> Self {
        Self::EMPTY
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Cell {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Deserialize<'a> for Cell {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        Primitive::deserialize(deserializer).map(Self)
    }
}

#[must_use]
pub const fn capacity(width: usize, height: usize) -> usize {
    if width == 0 || height == 0 {
        0
    } else {
        (width + 1) * (height + 1)
    }
}

pub fn clear(cells: &mut [Cell]) {
    cells.fill(Cell::EMPTY);
}

#[allow(clippy::transmute_ptr_to_ptr)]
fn primitive_slice(cells: &mut [Cell]) -> &mut [Primitive] {
    unsafe { mem::transmute(cells) }
}

const BOTTOM_LEFT: Primitive = 1 << 0;
const BOTTOM_RIGHT: Primitive = 1 << 1;
const TOP_LEFT: Primitive = 1 << 2;
const TOP_RIGHT: Primitive = 1 << 3;

fn first_pass(cells: &mut [Cell], width: usize, pixels: impl IntoIterator<Item = bool>) {
    let (cells, mut cell_index, mut column_index) = (primitive_slice(cells), 0, 0);
    for pixel in pixels {
        if pixel {
            cells[cell_index] |= BOTTOM_RIGHT;
            cells[cell_index + 1] |= BOTTOM_LEFT;
            cells[cell_index + 1 + width] |= TOP_RIGHT;
            cells[cell_index + 2 + width] = TOP_LEFT;
        }
        if column_index + 1 == width {
            cell_index += 2;
            column_index = 0;
        } else {
            cell_index += 1;
            column_index += 1;
        }
    }
}

const DOUBLE: Primitive = 1 << 0;

fn second_pass(cells: &mut [Cell]) {
    let (cells, mut vertex_index) = (primitive_slice(cells), 1);
    for cell in cells {
        if *cell == BOTTOM_LEFT
            || *cell == BOTTOM_RIGHT
            || *cell == TOP_LEFT
            || *cell == TOP_RIGHT
            || *cell == BOTTOM_LEFT | BOTTOM_RIGHT | TOP_LEFT
            || *cell == BOTTOM_RIGHT | TOP_LEFT | TOP_RIGHT
            || *cell == TOP_LEFT | TOP_RIGHT | BOTTOM_LEFT
            || *cell == TOP_RIGHT | BOTTOM_LEFT | BOTTOM_RIGHT
        {
            *cell = vertex_index << 1;
            vertex_index += 1;
        } else if *cell == BOTTOM_LEFT | TOP_RIGHT || *cell == BOTTOM_RIGHT | TOP_LEFT {
            *cell = (vertex_index << 1) ^ DOUBLE;
            vertex_index += 1;
        } else {
            *cell = EMPTY;
        }
    }
}

/// # Panics
/// If `cells.len()` less than [`capacity()`]
pub fn set(cells: &mut [Cell], width: usize, pixels: impl IntoIterator<Item = bool>) {
    first_pass(cells, width, pixels);
    second_pass(cells);
}

#[derive(Clone, Debug, Default)]
pub struct Vertices<'a> {
    cells: &'a [Cell],
    num_cell_columns: Primitive,
    cell_column_index: Primitive,
    cell_row_index: Primitive,
}

impl<'a> Iterator for Vertices<'a> {
    type Item = [Primitive; 2];

    fn next(&mut self) -> Option<Self::Item> {
        for (cell_index, &Cell(cell)) in self.cells.iter().enumerate() {
            let maybe_vertex = [self.cell_column_index, self.cell_row_index];
            self.cell_column_index += 1;
            if self.cell_column_index == self.num_cell_columns {
                self.cell_column_index = 0;
                self.cell_row_index += 1;
            }
            if cell != EMPTY {
                self.cells = &self.cells[cell_index + 1..];
                return Some(maybe_vertex);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.cells.len()))
    }
}

impl<'a> FusedIterator for Vertices<'a> {}

fn next_index_pair(maybe_prev_index: &mut Primitive, cell: Primitive) -> Option<[Primitive; 2]> {
    if cell == EMPTY {
        None
    } else {
        let index = cell >> 1;
        if *maybe_prev_index == EMPTY {
            *maybe_prev_index = index;
            None
        } else if cell & DOUBLE != 0 {
            Some([mem::replace(maybe_prev_index, index) - 1, index - 1])
        } else {
            Some([mem::take(maybe_prev_index) - 1, index - 1])
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HorizontalIndices<'a> {
    cells: &'a [Cell],
    maybe_prev: Primitive,
}

impl<'a> Iterator for HorizontalIndices<'a> {
    type Item = [Primitive; 2];

    fn next(&mut self) -> Option<Self::Item> {
        for (cell_index, &Cell(cell)) in self.cells.iter().enumerate() {
            let maybe_index_pair = next_index_pair(&mut self.maybe_prev, cell);
            if maybe_index_pair.is_some() {
                self.cells = &self.cells[cell_index + 1..];
                return maybe_index_pair;
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.cells.len() / 2))
    }
}

impl<'a> FusedIterator for HorizontalIndices<'a> {}

#[derive(Clone, Debug, Default)]
pub struct VerticalIndices<'a> {
    cells: &'a [Cell],
    num_cell_columns: Primitive,
    cell_column_index: Primitive,
    cell_row_index: Primitive,
    maybe_prev: Primitive,
}

impl<'a> Iterator for VerticalIndices<'a> {
    type Item = [Primitive; 2];

    fn next(&mut self) -> Option<Self::Item> {
        for cell_column_index in self.cell_column_index..self.num_cell_columns {
            for cell_index in (cell_column_index as _..self.cells.len())
                .step_by(self.num_cell_columns as _)
                .skip(self.cell_row_index as _)
            {
                let maybe_index_pair =
                    next_index_pair(&mut self.maybe_prev, self.cells[cell_index].0);
                if maybe_index_pair.is_some() {
                    if (self.cells.len() as Primitive)
                        <= cell_index as Primitive + self.num_cell_columns
                    {
                        self.cell_column_index = cell_column_index + 1;
                        self.cell_row_index = 0;
                    } else {
                        self.cell_column_index = cell_column_index;
                        self.cell_row_index += 1;
                    }
                    return maybe_index_pair;
                }
                self.cell_row_index += 1;
            }
            self.cell_row_index = 0;
        }
        self.cell_column_index = self.num_cell_columns;
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_cell_rows = (self.cells.len() as Primitive).saturating_div(self.num_cell_columns);
        let upper_bound =
            (self.num_cell_columns - self.cell_column_index) * num_cell_rows - self.cell_row_index;
        (0, Some(upper_bound as _))
    }
}

impl<'a> FusedIterator for VerticalIndices<'a> {}

#[must_use]
pub const fn get(cells: &[Cell], width: usize) -> (Vertices, HorizontalIndices, VerticalIndices) {
    let vertices = Vertices {
        cells,
        num_cell_columns: width as Primitive + 1,
        cell_column_index: 0,
        cell_row_index: 0,
    };
    let horizontal_indices = HorizontalIndices {
        cells,
        maybe_prev: 0,
    };
    let vertical_indices = VerticalIndices {
        cells,
        num_cell_columns: width as Primitive + 1,
        cell_column_index: 0,
        cell_row_index: 0,
        maybe_prev: 0,
    };
    (vertices, horizontal_indices, vertical_indices)
}
