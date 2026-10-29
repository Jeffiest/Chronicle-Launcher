//! Just enough ISO 9660 to read a file in a disc image's root: SYSTEM.CNF, and the index the release is told by.
//! PlayStation 2 DVDs use 2048-byte sectors, and these files are in the root.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

const SECTOR: u64 = 2048;

pub struct Iso {
    file: File,
    root: (u64, u64),
}

fn bad(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, what.to_string())
}

fn le32(bytes: &[u8], at: usize) -> u64 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]) as u64
}

impl Iso {
    pub fn open(path: &Path) -> io::Result<Iso> {
        let mut file = File::open(path)?;
        let mut pvd = [0u8; SECTOR as usize];
        file.seek(SeekFrom::Start(16 * SECTOR))?;
        file.read_exact(&mut pvd)
            .map_err(|_| bad("too small to be a disc image"))?;
        if pvd[0] != 1 || &pvd[1..6] != b"CD001" {
            return Err(bad("not an ISO 9660 disc image"));
        }
        // The root directory's record is at 156: its extent at +2, its size at +10.
        let root = (le32(&pvd, 156 + 2), le32(&pvd, 156 + 10));
        Ok(Iso { file, root })
    }

    fn read(&mut self, sector: u64, size: u64) -> io::Result<Vec<u8>> {
        let mut bytes = vec![0u8; size as usize];
        self.file.seek(SeekFrom::Start(sector * SECTOR))?;
        self.file.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    /// A file in the root directory, by name without its `;1`, ignoring case.
    pub fn root_file(&mut self, name: &str) -> io::Result<Option<Vec<u8>>> {
        let (sector, size) = self.root;
        if size > 1 << 20 {
            return Err(bad("root directory is implausibly large"));
        }
        let dir = self.read(sector, size)?;
        let mut at = 0usize;
        while at < dir.len() {
            let length = dir[at] as usize;
            if length == 0 {
                // Records never cross a sector; the rest of this one is padding.
                at = (at / SECTOR as usize + 1) * SECTOR as usize;
                continue;
            }
            if at + length > dir.len() || length < 34 {
                return Err(bad("damaged directory record"));
            }
            let record = &dir[at..at + length];
            let name_length = record[32] as usize;
            let raw = &record[33..(33 + name_length).min(length)];
            let found = String::from_utf8_lossy(raw);
            let found = found.split(';').next().unwrap_or("");
            if record[25] & 2 == 0 && found.eq_ignore_ascii_case(name) {
                let (extent, file_size) = (le32(record, 2), le32(record, 10));
                if file_size > 64 << 20 {
                    return Err(bad("file is implausibly large"));
                }
                return self.read(extent, file_size).map(Some);
            }
            at += length;
        }
        Ok(None)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A disc image holding `files` in its root, for tests.
    pub fn image(files: &[(&str, &[u8])]) -> Vec<u8> {
        let sectors = |n: usize| n.div_ceil(SECTOR as usize).max(1);
        let mut next = 19usize;
        let mut placed = Vec::new();
        for (name, data) in files {
            placed.push((*name, *data, next));
            next += sectors(data.len());
        }
        let mut disc = vec![0u8; next * SECTOR as usize];
        let record = |extent: usize, size: usize, flags: u8, name: &[u8]| {
            let length = (33 + name.len()).next_multiple_of(2);
            let mut r = vec![0u8; length];
            r[0] = length as u8;
            r[2..6].copy_from_slice(&(extent as u32).to_le_bytes());
            r[10..14].copy_from_slice(&(size as u32).to_le_bytes());
            r[25] = flags;
            r[32] = name.len() as u8;
            r[33..33 + name.len()].copy_from_slice(name);
            r
        };
        let mut dir = record(18, 2048, 2, &[0]);
        dir.extend(record(18, 2048, 2, &[1]));
        for (name, data, extent) in &placed {
            dir.extend(record(*extent, data.len(), 0, format!("{name};1").as_bytes()));
            let start = extent * SECTOR as usize;
            disc[start..start + data.len()].copy_from_slice(data);
        }
        let pvd = 16 * SECTOR as usize;
        disc[pvd] = 1;
        disc[pvd + 1..pvd + 6].copy_from_slice(b"CD001");
        let root = record(18, 2048, 2, &[0]);
        disc[pvd + 156..pvd + 156 + root.len()].copy_from_slice(&root);
        disc[18 * SECTOR as usize..18 * SECTOR as usize + dir.len()].copy_from_slice(&dir);
        disc
    }

    fn write(bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, bytes).unwrap();
        file
    }

    #[test]
    fn reads_a_root_file_by_name() {
        let file = write(&image(&[
            ("SYSTEM.CNF", b"BOOT2 = cdrom0:\\SCUS_971.11;1\r\n"),
            ("DATA.HD2", &[7; 3000]),
        ]));
        let mut iso = Iso::open(file.path()).unwrap();
        assert_eq!(
            iso.root_file("system.cnf").unwrap().unwrap(),
            b"BOOT2 = cdrom0:\\SCUS_971.11;1\r\n"
        );
        assert_eq!(iso.root_file("DATA.HD2").unwrap().unwrap(), vec![7; 3000]);
        assert_eq!(iso.root_file("DATA.DAT").unwrap(), None);
    }

    #[test]
    fn something_else_is_not_a_disc() {
        let file = write(&[0u8; 40000]);
        assert!(Iso::open(file.path()).is_err());
        let file = write(b"tiny");
        assert!(Iso::open(file.path()).is_err());
    }
}
