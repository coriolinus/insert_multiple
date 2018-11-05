use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
};

/// size of the internal buffer used to copy data from readers to writers
pub const BUFFER_SIZE: usize = 1024;

type Insertions<'i> = BTreeMap<usize, Box<'i + Read>>;

/// inserter keeps track of origin reader, target writer, and all points of insertion
pub struct Inserter<'i, R, W> {
    origin: R,
    insertions: Insertions<'i>,
    target: W,
}

impl<'i, R, W> Inserter<'i, R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(origin: R, target: W) -> Inserter<'i, R, W> {
        Inserter {
            origin: origin,
            insertions: BTreeMap::new(),
            target: target,
        }
    }

    pub fn insert<I: 'i + Read>(mut self, position: usize, source: I) -> Self {
        self.insertions.insert(position, Box::new(source));
        self
    }

    pub fn execute(mut self) -> io::Result<()> {
        let mut input_index: usize = 0;
        let mut buffer = [0_u8; BUFFER_SIZE];
        for (&insert_idx, mut to_insert) in self.insertions.iter_mut() {
            // if we haven't yet reached this insertion index, copy bytes
            // from the origin until we have
            while input_index < insert_idx {
                let remaining_bytes = insert_idx - input_index;
                let mut source = self.origin.by_ref().take(remaining_bytes as u64);
                match source.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(bytes_read) => {
                        let written = &buffer[..bytes_read];
                        input_index += bytes_read;
                        self.target.write_all(written)?;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                        // try again
                    }
                    Err(e) => return Err(e),
                }
            }

            // now that we've reached the insertion index (or the origin has
            // run out of bytes), copy over the data at this insertion point
            // note that this doesn't affect the input index
            loop {
                match to_insert.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(bytes_read) => {
                        let written = &buffer[..bytes_read];
                        self.target.write_all(written)?;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                        // try again
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        // we've added all inserts
        // now finish copying over any remaining bytes from the origin
        loop {
            match self.origin.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    let written = &buffer[..bytes_read];
                    self.target.write_all(written)?;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                    // try again
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn insert_one_at_beginning() {
        let origin: Vec<u8> = (5..10).collect();
        let insertion: Vec<u8> = (0..5).collect();
        let mut dest = Vec::new();

        Inserter::new(origin.as_slice(), Cursor::new(&mut dest))
            .insert(0, insertion.as_slice())
            .execute()
            .expect("manipulating u8 lists should never fail");

        assert_eq!(&(0..10).collect::<Vec<_>>(), &dest);
    }

    #[test]
    fn insert_one_at_end() {
        let origin: Vec<u8> = (0..5).collect();
        let insertion: Vec<u8> = (5..10).collect();
        let mut dest = Vec::new();

        Inserter::new(origin.as_slice(), Cursor::new(&mut dest))
            .insert(origin.len(), insertion.as_slice())
            .execute()
            .expect("manipulating u8 lists should never fail");

        assert_eq!(&(0..10).collect::<Vec<_>>(), &dest);
    }

    #[test]
    fn insert_one_past_end() {
        let origin: Vec<u8> = (0..5).collect();
        let insertion: Vec<u8> = (10..15).collect();
        let mut dest = Vec::new();

        Inserter::new(origin.as_slice(), Cursor::new(&mut dest))
            .insert(10, insertion.as_slice())
            .execute()
            .expect("manipulating u8 lists should never fail");

        let expect: Vec<u8> = (0..5).chain(10..15).collect();

        assert_eq!(&expect, &dest);
    }

    #[test]
    fn interleave_single() {
        let origin: Vec<u8> = (0..10).filter(|i| i % 2 != 0).collect(); // odds
        let insertions: Vec<u8> = (0..10).filter(|i| i % 2 == 0).collect(); // evens
        let mut dest = Vec::new();

        {
            let mut inserter = Inserter::new(origin.as_slice(), Cursor::new(&mut dest));
            for i in 0..insertions.len() {
                inserter = inserter.insert(i, &insertions[i..i + 1]);
            }
            inserter
                .execute()
                .expect("manipulating u8 lists should never fail");
        }

        assert_eq!(&(0..10).collect::<Vec<u8>>(), &dest);
    }
}
