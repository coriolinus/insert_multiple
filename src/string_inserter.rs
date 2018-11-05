use inserter::Inserter;
use std::io::{self, Cursor};
use std::string::FromUtf8Error;

type Insertions<'i> = Vec<(usize, &'i str)>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    Utf8Error(FromUtf8Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

/// inserter keeps track of origin, target writer, and all points of insertion
pub struct StringInserter<'o, 'i> {
    origin: &'o str,
    insertions: Insertions<'i>,
}

impl<'o, 'i> StringInserter<'o, 'i> {
    /// create a new inserter with the specified origin document and target
    pub fn new(origin: &'o str) -> StringInserter<'o, 'i> {
        StringInserter {
            origin: origin,
            insertions: Insertions::new(),
        }
    }

    /// insert the source document into the output document at the given origin index
    pub fn insert(mut self, position: usize, source: &'i str) -> Self {
        self.insertions.push((position, source));
        self
    }

    /// execute this inserter, consuming it
    pub fn execute(self) -> Result<String, Error> {
        // this just delegates to Inserter, of course
        let mut buffer: Vec<u8> = Vec::with_capacity(
            self.origin.len() + self.insertions.iter().map(|(_, i)| i.len()).sum::<usize>(),
        );
        {
            let mut inserter = Inserter::new(self.origin.as_bytes(), Cursor::new(&mut buffer));
            for (position, item) in self.insertions.iter() {
                inserter = inserter.insert(*position, item.as_bytes());
            }
            if let Err(err) = inserter.execute() {
                return Err(err.into());
            }
        }

        String::from_utf8(buffer).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_one_at_beginning() {
        let origin = "fghij";
        let insertion = "abcde";

        let out = StringInserter::new(origin)
            .insert(0, insertion)
            .execute()
            .unwrap();

        assert_eq!("abcdefghij", &out);
    }

    #[test]
    fn insert_one_at_end() {
        let origin = "abcde";
        let insertion = "fghij";

        let out = StringInserter::new(origin)
            .insert(origin.len(), insertion)
            .execute()
            .unwrap();

        assert_eq!("abcdefghij", &out);
    }

    #[test]
    fn insert_one_past_end() {
        let origin = "abcde";
        let insertion = "klmno";

        let out = StringInserter::new(origin)
            .insert(origin.len() * 2, insertion)
            .execute()
            .unwrap();

        assert_eq!("abcdeklmno", &out);
    }

    #[test]
    fn interleave() {
        let origin = "alpha bravo delta hotel";
        let out = StringInserter::new(origin)
            .insert(12, "charlie ")
            .insert(18, "echo fox golf ")
            .execute()
            .unwrap();

        assert_eq!("alpha bravo charlie delta echo fox golf hotel", &out);
    }
}
