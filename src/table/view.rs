use std::fmt::Display;

use cursive::{
    view::{Resizable, ViewWrapper},
    views::{LinearLayout, TextView},
};

use crate::table::{Table, cell::CellContent, slice::table::TableSlice};

pub const TABLE_CELL_PLACEHOLDER: &'static str = "TABLE";

/// Enum that defines the alignment of the contents of the cell
pub enum ContentAlignment {
    Left,
    Center,
    Right,
}

///
/// A struct that defines TableView's style
///
pub struct TableStyle<'a> {
    // Cells' contents' alignment
    pub align: ContentAlignment,
    /// The width of the contents (not including separator)
    pub cell_width: usize,
    /// The height of the contents (not including separator)
    /// TODO: Actually support width
    pub cell_height: usize,
    /// The separator between the columns (will be placed after each one)
    pub col_separator: &'a str,
    /// The separator between the rows (will be repeated for the whole line after each row)
    pub row_separator: &'a str,
}

impl<'a> Default for TableStyle<'a> {
    fn default() -> Self {
        Self {
            align: ContentAlignment::Right,
            cell_width: 10,
            cell_height: 1,
            col_separator: "|",
            row_separator: "-",
        }
    }
}

pub struct TableView {
    view: LinearLayout,
}

impl TableView {
    pub fn from_table_slice<'a, T: Table<Item: Display>>(
        slice: TableSlice<'a, T>,
        style: TableStyle<'_>,
    ) -> Self {
        let width = style.cell_width;

        let row_separator_width =
            style.cell_width * slice.width() + style.col_separator.len() * (slice.width() + 1);

        let row_seprator_count =
            (row_separator_width + style.row_separator.len() - 1) / style.row_separator.len();

        let row_separator = style.row_separator.repeat(row_seprator_count);

        let mut layout = LinearLayout::vertical()
            .child(TextView::new(&row_separator).fixed_size((row_separator_width, 1)));

        for row in slice.rows() {
            let mut hor_layout = LinearLayout::horizontal().child(
                TextView::new(style.col_separator).fixed_size((style.col_separator.len(), 1)),
            );

            for cell in row {
                let content = match cell {
                    None => format!(""),
                    Some(c) => match c {
                        CellContent::Table(_) => format!("{TABLE_CELL_PLACEHOLDER}"),
                        CellContent::Value(v) => format!("{v}"),
                    },
                };

                let content = match style.align {
                    ContentAlignment::Left => {
                        format!("{:<width$}{}", content, style.col_separator)
                    }
                    ContentAlignment::Center => {
                        format!("{:^width$}{}", content, style.col_separator)
                    }
                    ContentAlignment::Right => {
                        format!("{:>width$}{}", content, style.col_separator)
                    }
                };
                hor_layout = hor_layout.child(
                    TextView::new(content)
                        .fixed_size((style.col_separator.len() + style.cell_width, 1)),
                );
            }
            layout = layout
                .child(hor_layout)
                .child(TextView::new(&row_separator).fixed_size((row_separator_width, 1)));
        }
        Self { view: layout }
    }
}

impl ViewWrapper for TableView {
    cursive::wrap_impl!(self.view: LinearLayout);
}
