pub mod paper {
    use regex::Regex;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, Read, Result, Seek, SeekFrom};

    use crate::paper::XRefEntryType::{Free, InUse};

    pub struct Paper {
        filename: String,
        file: BufReader<File>,
        pub version: String,
    }

    #[derive(Debug)]
    struct ObjectRef {
        object_number: usize,
        generation_number: usize,
    }

    #[derive(Debug)]
    enum TrailerValue {
        Int(u32),
        Ref(ObjectRef),
    }

    enum XRefEntryType {
        InUse,
        Free,
    }

    struct XRefTableEntry {
        offset: usize,
        generation_number: usize,
        status_flag: XRefEntryType,
    }
    type XRefTable = Vec<XRefTableEntry>;
    type Trailer = HashMap<String, TrailerValue>;

    impl Paper {
        pub fn open(filename: impl Into<String>) -> io::Result<Self> {
            // open file
            let filename = filename.into();
            let mut file = File::open(&filename)?;

            let version = Self::read_header(&file)?;

            let xref_offset = Self::find_xref_start(&mut file)?;

            let xref_table = Self::read_xref_table(&mut file, xref_offset);

            Ok(Self {
                filename,
                file: BufReader::new(file),
                version,
            })
        }

        fn read_header(file: &File) -> io::Result<String> {
            // read first line
            let mut reader = BufReader::new(file);
            let mut version_line = String::new();

            if reader.read_line(&mut version_line)? == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "file is empty",
                ));
            }
            let version = version_line.trim().to_string();

            if !regex::Regex::new(r"^%PDF-1.[0-7]")
                .unwrap()
                .is_match(&version)
            {
                panic!("invalid version: {}", version);
            }

            return Ok(version);
        }

        fn find_xref_start(file: &mut File) -> io::Result<u64> {
            let file_len = file.seek(SeekFrom::End(0))?;
            let window = file_len.min(64 * 1024);
            file.seek(SeekFrom::End(-(window as i64)))?;

            let mut tail = vec![0; window as usize];
            file.read_exact(&mut tail)?;

            let marker = b"startxref";
            let pos = tail
                .windows(marker.len())
                .rposition(|chunk| chunk == marker)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no startxref"))?;

            let after_marker = &tail[pos + marker.len()..];
            let offset_text = std::str::from_utf8(after_marker)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "non-UTF-8 xref offset"))?
                .trim_start()
                .lines()
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing xref offset"))?;

            offset_text
                .trim()
                .parse::<u64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid xref offset"))
        }

        fn read_xref_table(file: &mut File, start_xref: u64) -> io::Result<XRefTable> {
            file.seek(SeekFrom::Start(start_xref))?;

            let mut xref_table = XRefTable::new();
            let mut reader = BufReader::new(file);

            let mut line_buf = String::new();
            let _ = reader.skip_until(b'\n');
            if reader.read_line(&mut line_buf)? == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Unexpected end of file",
                ));
            }
            let (first_obj_num, num_entries) = line_buf.split_once(" ").ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "cannot parse xref table first line",
                )
            })?;

            let (first_obj_num, num_entries) = (
                first_obj_num.trim().parse::<u64>().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("cannot parse {} into u64", first_obj_num),
                    )
                })?,
                num_entries.trim().parse::<u64>().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("cannot parse {} into u64", num_entries),
                    )
                })?,
            );

            for _ in first_obj_num..first_obj_num + num_entries {
                line_buf.clear();
                if reader.read_line(&mut line_buf)? == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Unexpected end of file",
                    ));
                }
                let mut iter = line_buf.split_whitespace();
                let offset = iter
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "unable to parse object offset")
                    })?
                    .parse::<usize>()
                    .map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "unable to parse object offset")
                    })?;
                let gen_num = iter
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "cannot parse generation number")
                    })?
                    .parse::<usize>()
                    .map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "unable to parse object offset")
                    })?;
                let status_flag = match iter
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "missing status flag")
                    })?
                    .trim()
                {
                    "n" => InUse,
                    "f" => Free,
                    x => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("invalid status flag value {}", x),
                        ));
                    }
                };

                xref_table.push(XRefTableEntry {
                    offset,
                    generation_number: gen_num,
                    status_flag,
                })
            }

            return Ok(xref_table);
        }

        fn read_trailer(file: &mut File) -> io::Result<Trailer> {
            todo!("implement")
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::paper::Paper;
        use std::fs::File;

        #[test]
        fn opens_valid_pdf() {
            let paper =
                Paper::open("testdata/001-trivial/minimal-document.pdf").expect("PDF should open");

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
                    matches!(&entry.status_flag, super::XRefEntryType::Free),
                    is_free
                );
            }
        }
    }
}
