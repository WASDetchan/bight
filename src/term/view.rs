pub mod editor {
    use crossterm::{
        cursor::{MoveTo, RestorePosition, SavePosition},
        queue,
        style::Print,
    };

    use crate::{
        editor::{EditorState, display_sequence},
        key::Key,
    };

    use super::DrawRect;

    pub fn draw(buf: &mut impl std::io::Write, rect: DrawRect, state: &EditorState, seq: &[Key]) {
        let mode = state.mode.to_string();
        let seq = display_sequence(seq);
        let width = rect.end_x - rect.start_x + 1;
        if mode.len() + seq.len() > width as usize {
            panic!("Not enough editor width!"); // TODO: handle this error
        }
        let padding_width = width as usize - mode.len() - seq.len();

        queue!(
            buf,
            SavePosition,
            MoveTo(rect.start_x, rect.end_y),
            Print(format!("{mode}{:-<width$}{seq}", "", width = padding_width)),
            RestorePosition,
        )
        .unwrap();
    }
}

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
}
