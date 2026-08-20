pub mod parser;
#[cfg(test)]
pub mod tests;

pub mod paper {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

    use crate::paper::XRefEntryType::{Free, InUse};
    use crate::parser;
    use crate::parser::ParseError;

    pub struct Paper {
        _filename: String,
        reader: BufReader<File>,
        pub data: Option<PdfData>,
    }

    pub struct PdfData {
        pub version: String,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct ObjectRef {
        pub object_number: isize,
        pub generation_number: isize,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum PdfObject {
        ObjectRef(ObjectRef),
        StringLiteral(String),
        HexString(Vec<u8>),
        Array(Vec<PdfObject>),
        Integer(isize),
    }

    pub enum XRefEntryType {
        InUse,
        Free,
    }

    pub struct XRefTableEntry {
        pub offset: usize,
        pub generation_number: usize,
        pub status_flag: XRefEntryType,
    }
    type XRefTable = Vec<XRefTableEntry>;
    pub type Dictionary = HashMap<String, PdfObject>;

    impl Paper {
        pub fn open(filename: impl Into<String>) -> io::Result<Self> {
            // open file
            let filename = filename.into();
            let file = File::open(&filename)?;

            Ok(Self {
                _filename: filename,
                reader: BufReader::new(file),
                data: None,
            })
        }

        pub fn extract(&mut self) -> io::Result<()> {
            let version = self.read_header()?;

            let xref_offset = self.find_xref_start()?;

            let _ = self.read_xref_table(xref_offset)?;

            self.data = Some(PdfData { version: version });
            Ok(())
        }

        pub fn read_header(&mut self) -> io::Result<String> {
            // read first line
            let mut version_line = String::new();

            if self.reader.read_line(&mut version_line)? == 0 {
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

        pub fn find_xref_start(&mut self) -> io::Result<u64> {
            let file_len = self.reader.seek(SeekFrom::End(0))?;
            let window = file_len.min(64 * 1024);
            self.reader.seek(SeekFrom::End(-(window as i64)))?;

            let mut tail = vec![0; window as usize];
            self.reader.read_exact(&mut tail)?;

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

        pub fn read_xref_table(&mut self, start_xref: u64) -> io::Result<XRefTable> {
            self.reader.seek(SeekFrom::Start(start_xref))?;

            let mut xref_table = XRefTable::new();

            let mut line_buf = String::new();
            let _ = self.reader.skip_until(b'\n');
            if self.reader.read_line(&mut line_buf)? == 0 {
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
                if self.reader.read_line(&mut line_buf)? == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Unexpected end of file",
                    ));
                }
                // dbg!(&line_buf);
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

        pub fn read_trailer(&mut self) -> io::Result<Dictionary> {
            let mut lines = self.reader.by_ref().lines();

            let mut trailer_found = false;
            while let Some(line) = lines.next() {
                let line = line?;
                if line.trim() == "trailer" {
                    trailer_found = true;
                    break;
                }
            }
            if !trailer_found {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "No trailer found, must only call this function while pointer offset is before trailer keyword",
                ));
            }

            match parser::parse_dictionary(self.reader.by_ref()) {
                Err(ParseError::IOError(e)) => Err(e),
                Err(other) => Err(io::Error::new(io::ErrorKind::Other, other.to_string())),
                Ok(dict) => Ok(dict),
            }
        }
    }
}
