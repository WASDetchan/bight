pub mod table {

    use crossterm::{cursor::MoveTo, queue, style::Print};

    use crate::{
        evaluator::EvaluatorTable,
        table::{cell::CellPos, slice::table::TableSlice},
    };

    use super::DrawRect;

    pub fn draw_grid(buf: &mut impl std::io::Write, rect: DrawRect) {
        enum Line {
            Sep,
            Cells,
        }
        let width = (rect.end_x + 1 - rect.start_x) as usize;
        let mut line = Line::Sep;
        let sep_line = format!("{:-<width$}", "");
        let cell = String::from("|         ");
        let cell_line = format!("{:width$}", format!("{}|", cell.repeat(width / cell.len())));
        for y in rect.start_y..=rect.end_y {
            queue!(
                buf,
                MoveTo(rect.start_x, y),
                Print(match line {
                    Line::Sep => {
                        line = Line::Cells;
                        &sep_line
                    }
                    Line::Cells => {
                        line = Line::Sep;
                        &cell_line
                    }
                })
            )
            .unwrap();
        }
    }

    pub fn draw_table(
        buf: &mut impl std::io::Write,
        rect: DrawRect,
        slice: TableSlice<'_, EvaluatorTable>,
    ) {
        let empty_cell = String::from("         ");
        let mut posy = rect.start_y + 1;
        for row in slice.rows() {
            let mut posx = rect.start_x + 1;
            for cell in row {
                queue!(buf, MoveTo(posx, posy),).unwrap();
                posx += 10; // TODO: make real styling and not hardcoded strs and magic numbers
                if posx > rect.end_x {
                    break;
                }

                let w = std::cmp::min(9, rect.end_x - posx + 1) as usize;

                if let Some(cont) = cell {
                    let mut form = format!(
                        "{:>w$}",
                        cont.try_read()
                            .expect("No guards are held after evaluation")
                            .as_ref()
                            .expect("All caches are valid if invalid_caches is empty")
                    );
                    form.truncate(w); // idk if it really works fine with chars and not
                    // just bytes  FIXME:
                    queue!(buf, Print(&form)).unwrap();
                } else {
                    queue!(buf, Print(&empty_cell)).unwrap();
                };
            }

            posy += 2;
        }
    }

    pub fn draw_expand_cursor(
        buf: &mut impl std::io::Write,
        rect: DrawRect,
        pos: impl Into<CellPos>,
        slice: TableSlice<'_, EvaluatorTable>,
    ) {
        let pos: CellPos = pos.into();
        set_cursor(buf, rect, pos);
        if let Some(Some(cont)) = slice.get(pos) {
            queue!(
                buf,
                Print(&format!(
                    "{}",
                    cont.try_read()
                        .expect("No guards are held after evaluation")
                        .as_ref()
                        .expect("All caches are valid if invalid_caches is empty")
                ))
            )
            .unwrap();
        }
    }
    pub fn set_cursor(buf: &mut impl std::io::Write, rect: DrawRect, pos: impl Into<CellPos>) {
        let pos: CellPos = pos.into();
        let y = rect.start_y + 1 + 2 * (pos.y as u16);
        let x = rect.start_x + 1 + 10 * (pos.x as u16);

        queue!(buf, MoveTo(x, y)).unwrap();
    }
}

pub mod editor {

    use crossterm::{cursor::MoveTo, queue, style::Print};

    use crate::{
        editor::{EditorState, display_sequence},
        evaluator::EvaluatorTable,
        key::Key,
        table::slice::table::TableSlice,
    };

    use super::{DrawRect, table};

    pub fn draw(
        buf: &mut impl std::io::Write,
        rect: DrawRect,
        state: &EditorState,
        seq: &[Key],
        data: TableSlice<'_, EvaluatorTable>,
    ) {
        let mode = state.mode.to_string();
        let seq = display_sequence(seq);
        let width = rect.end_x - rect.start_x + 1;
        if mode.len() + seq.len() > width as usize {
            panic!("Not enough editor width!"); // TODO: handle this error
        }
        let padding_width = width as usize - mode.len() - seq.len();

        let table_rect = DrawRect {
            end_y: rect.end_y - 1,
            ..rect
        };

        table::draw_grid(buf, table_rect);
        table::draw_table(buf, table_rect, data);
        if state.expand {
            table::draw_expand_cursor(buf, table_rect, state.cursor, data);
        }
        queue!(
            buf,
            MoveTo(rect.start_x, rect.end_y),
            Print(format!(
                "{mode}{:-<width$}{seq}",
                state.expand,
                width = padding_width
            )),
        )
        .unwrap();

        table::set_cursor(buf, table_rect, state.cursor);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DrawRect {
    pub start_x: u16,
    pub end_x: u16,
    pub start_y: u16,
    pub end_y: u16,
}

impl DrawRect {
    pub fn full_term() -> Self {
        let size = crossterm::terminal::size().unwrap();
        Self {
            start_x: 0,
            start_y: 0,
            end_x: size.0 - 1,
            end_y: size.1 - 1,
        }
    }

    pub fn width(&self) -> u16 {
        self.end_x - self.start_x + 1
    }
}
