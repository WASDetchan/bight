use std::{fmt::Display, io::Write};

use crate::table::{Table, slice::table::TableSlice};

pub fn write_slice_to_csv(
    slice: TableSlice<'_, impl Table<Item: Display>>,
    writer: &mut csv::Writer<impl Write>,
) -> Result<(), csv::Error> {
    for x in slice.col_indexes() {
        writer.write_record(slice.row_indexes().map(|y| {
            slice
                .get((x, y))
                .expect("The position is inside the slice")
                .map(|v| v.to_string())
                .unwrap_or_default()
        }))?;
    }
    Ok(())
}

pub fn slice_to_csv_string(slice: TableSlice<'_, impl Table<Item: Display>>) -> String {
    let mut s = Vec::<u8>::new();
    let mut v = csv::WriterBuilder::new().from_writer(&mut s);

    write_slice_to_csv(slice, &mut v).expect("The writer configuration was valid");
    v.flush().expect("The writer configuration is valid");
    drop(v);

    String::from_utf8(s).expect("No non-utf8 data was written")
}

#[cfg(test)]
mod test {
    use crate::table::{DataTable, TableMut};

    use super::*;

    #[test]
    fn csv() {
        let mut table = DataTable::new();
        table.set((0, 0).into(), Some("Hello, "));
        table.set((1, 1).into(), Some("World!"));

        let csv = slice_to_csv_string(table.full_slice());

        assert_eq!(csv, "\"Hello, \",\n,World!\n");
    }

    #[test]
    fn cyrillic() {
        let mut table = DataTable::new();
        table.set((0, 0).into(), Some("Привет, "));
        table.set((1, 1).into(), Some("мир!"));

        let csv = slice_to_csv_string(table.full_slice());

        assert_eq!(csv, "\"Привет, \",\n,мир!\n");
    }
}
