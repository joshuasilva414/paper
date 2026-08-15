pub mod paper {
    use regex::Regex;
    use std::fs::File;
    use std::io::{self, BufRead, BufReader};

    pub struct Paper {
        filename: String,
        file: BufReader<File>,
        pub version: String,
    }

    impl Paper {
        pub fn open(filename: impl Into<String>) -> io::Result<Self> {
            // open file
            let filename = filename.into();
            let file = File::open(&filename)?;
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
            // read second line (verifies pdf binary)

            Ok(Self {
                filename,
                file: reader,
                version,
            })
        }
    }
}
