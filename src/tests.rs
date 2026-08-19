use crate::paper::Paper;
use std::fs::File;

#[test]
fn opens_valid_pdf() {
    let paper = Paper::open("testdata/001-trivial/minimal-document.pdf").expect("PDF should open");

    assert_eq!(paper.version, "%PDF-1.5");
}

#[test]
fn parses_xref_offset() {
    let mut file =
        File::open("testdata/001-trivial/minimal-document.pdf").expect("PDF should open");
    let xref_offset = Paper::find_xref_start(&mut file).unwrap();

    assert_eq!(xref_offset, 16675);
}

#[test]
fn parses_plain_xref_table() {
    let mut file = File::open("testdata/008-reportlab-inline-image/inline-image.pdf")
        .expect("PDF should open");
    let xref_offset = Paper::find_xref_start(&mut file).unwrap();
    let xref_table =
        Paper::read_xref_table(&mut file, xref_offset).expect("xref table should parse");

    let expected = [
        (0, 65535, true),
        (73, 0, false),
        (104, 0, false),
        (211, 0, false),
        (414, 0, false),
        (482, 0, false),
        (778, 0, false),
        (837, 0, false),
    ];
    assert_eq!(xref_table.len(), expected.len());
    for (entry, (offset, generation_number, is_free)) in xref_table.iter().zip(expected) {
        assert_eq!(entry.offset, offset);
        assert_eq!(entry.generation_number, generation_number);
        assert_eq!(
            matches!(&entry.status_flag, crate::paper::XRefEntryType::Free),
            is_free
        );
    }
}

#[test]
fn parses_plain_trailer_dictionary() {
    let mut file = File::open("testdata/008-reportlab-inline-image/inline-image.pdf")
        .expect("PDF should open");
    let xref_offset = Paper::find_xref_start(&mut file).unwrap();
    let xref_table =
        Paper::read_xref_table(&mut file, xref_offset).expect("xref table should parse");

    let trailer = Paper::read_trailer(&mut file).unwrap();
}
