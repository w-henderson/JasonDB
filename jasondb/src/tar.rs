use std::io::{Read, Seek, SeekFrom};

pub struct ReadableArchive<T>
where
    T: Read + Seek,
{
    source: T,
    offset: u64,
}

pub struct WritableArchive {
    entries: Vec<WriteEntry>,
}

pub struct ReadEntry {
    pub name: String,
    pub pointer: u64,
    pub length: u64,
}

pub struct WriteEntry {
    name: String,
    data: Vec<u8>,
}

impl<T> ReadableArchive<T>
where
    T: Read + Seek,
{
    pub fn new(source: T) -> Self {
        Self { source, offset: 0 }
    }
}

impl WritableArchive {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, name: impl AsRef<str>, data: Vec<u8>) {
        if name.as_ref().len() > 100 {
            panic!("Name too long");
        }

        self.entries.push(WriteEntry {
            name: name.as_ref().to_string(),
            data,
        });
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for entry in &self.entries {
            let mut entry_bytes: [u8; 512] = [0; 512];

            // Write the name
            entry_bytes[..entry.name.len()].copy_from_slice(entry.name.as_bytes());

            // Write the size
            let size_bytes = format!("{:01$o}", entry.data.len(), 11);
            entry_bytes[124..135].copy_from_slice(size_bytes.as_bytes());

            // Write an empty time
            entry_bytes[136..147].copy_from_slice(&[0x30; 11]);

            // Write the magic string
            entry_bytes[257..264].copy_from_slice(b"ustar  ");

            // Write the checksum
            // During calculation checksum should be considered to be spaces, so add 256 to the total
            let checksum = entry_bytes.iter().fold(0, |acc, &x| acc + x as u64) + 256;
            let checksum_bytes = format!("{:01$o}", checksum, 7);
            entry_bytes[148..155].copy_from_slice(checksum_bytes.as_bytes());

            // Copy to the result
            result.extend_from_slice(&entry_bytes);
            result.extend_from_slice(&entry.data);

            // Pad to 512 bytes
            if entry.data.len() % 512 != 0 {
                result.extend_from_slice(&vec![0; 512 - (entry.data.len() % 512)]);
            }
        }

        // Add two empty blocks
        result.extend_from_slice(&[0; 1024]);

        result
    }
}

impl<T> Iterator for ReadableArchive<T>
where
    T: Read + Seek,
{
    type Item = ReadEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // Go to the specified offset
        self.source.seek(SeekFrom::Start(self.offset)).ok()?;

        // Read the file header
        let mut buf: [u8; 512] = [0; 512];
        self.source.read_exact(&mut buf).ok()?;

        // Extract key information
        let nul = buf[0..100].iter().position(|&b| b == 0).unwrap_or(100);
        let name = String::from_utf8(buf[0..nul].to_vec()).ok()?;
        let length = u64::from_str_radix(std::str::from_utf8(&buf[124..136]).ok()?, 8).ok()?;
        let pointer = self.offset + 512;

        // Update the offset
        self.offset += 512 + ((length + 512) / 512) * 512;

        // Return the entry
        Some(Self::Item {
            name,
            pointer,
            length,
        })
    }
}
