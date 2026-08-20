use crate::paper::{Dictionary, Paper, PdfObject};

#[test]
fn opens_valid_pdf() {
    let _ = Paper::open("testdata/001-trivial/minimal-document.pdf").expect("PDF should open");
}

#[test]
fn parses_header() {
    let mut paper =
        Paper::open("testdata/001-trivial/minimal-document.pdf").expect("PDF should open");
    let header = paper.read_header().expect("Failed to read header");
    assert_eq!(header, String::from("%PDF-1.5"));
}

#[test]
fn parses_xref_offset() {
    let mut paper =
        Paper::open("testdata/001-trivial/minimal-document.pdf").expect("PDF should open");
    let xref_offset = paper.find_xref_start().unwrap();

    assert_eq!(xref_offset, 16675);
}

#[test]
fn parses_plain_xref_table() {
    let mut paper = Paper::open("testdata/008-reportlab-inline-image/inline-image.pdf")
        .expect("PDF should open");
    paper.extract().unwrap();
    let xref_offset = paper.find_xref_start().unwrap();
    let xref_table = paper
        .read_xref_table(xref_offset)
        .expect("xref table should parse");

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
    let mut paper = Paper::open("testdata/008-reportlab-inline-image/inline-image.pdf")
        .expect("PDF should open");
    let xref_offset = paper.find_xref_start().unwrap();
    let _ = paper
        .read_xref_table(xref_offset)
        .expect("xref table should parse");
    let trailer = paper.read_trailer().unwrap();
    assert_eq!(
        Dictionary::from([
            (
                String::from("ID"),
                PdfObject::Array(vec![
                    PdfObject::HexString(Vec::from(b"e592e1aa567158bd21e449678b7a736a")),
                    PdfObject::HexString(Vec::from(b"e592e1aa567158bd21e449678b7a736a"))
                ])
            ),
            (
                String::from("Info"),
                PdfObject::ObjectRef(crate::paper::ObjectRef {
                    object_number: 5,
                    generation_number: 0
                })
            ),
            (
                String::from("Root"),
                PdfObject::ObjectRef(crate::paper::ObjectRef {
                    object_number: 4,
                    generation_number: 0
                })
            ),
            (String::from("Size"), PdfObject::Integer(8))
        ]),
        trailer
    )
}
