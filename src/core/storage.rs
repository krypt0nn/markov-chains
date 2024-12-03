use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom, BufReader, BufWriter};
use std::marker::PhantomData;
use std::iter::FusedIterator;

use crate::prelude::*;

#[derive(Debug)]
pub struct StorageReader<Entity> {
    file: BufReader<File>,
    compression: Compression,
    block: Vec<u8>,
    block_ptr: usize,
    _entity: PhantomData<Entity>
}

impl<Entity: Serialize> Iterator for StorageReader<Entity> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.block.len();

        if self.block_ptr >= n {
            let mut len = [0; 2];

            self.file.read_exact(&mut len).ok()?;

            let mut block = vec![0; u16::from_be_bytes(len) as usize];

            self.file.read_exact(&mut block).ok()?;

            self.block = self.compression.decompress(&block).ok()?;
            self.block_ptr = 0;
        }

        let len = u16::from_be_bytes([
            self.block[self.block_ptr],
            self.block[self.block_ptr + 1]
        ]) as usize;

        let entity = &self.block[self.block_ptr + 2..self.block_ptr + 2 + len];

        self.block_ptr += len + 2;

        Entity::from_bytes(entity)
    }
}

impl<Entity: Serialize> FusedIterator for StorageReader<Entity> {}

#[derive(Debug)]
/// Generic file storage with entities compression.
pub struct Storage<Entity: Serialize> {
    path: PathBuf,
    file: BufWriter<File>,
    compression: Compression,
    block_size: u16,
    buf: Vec<u8>,
    _entity: PhantomData<Entity>
}

impl<Entity: Serialize> Storage<Entity> {
    const DATA_OFFSET: u64 = 3;

    /// Open existing store or create a new one.
    pub fn open(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path: PathBuf = path.into();

        if !path.exists() {
            // Subject of discussions.
            Ok(Self::create(path, Compression::default(), 1024)?)
        }

        else {
            let mut file = File::options()
                .read(true)
                .write(true)
                .open(&path)?;

            // Read compression variant.
            let mut compression = [0; 1];

            file.read_exact(&mut compression)?;

            let compression = match compression[0] {
                1 => Compression::Lz4,
                2 => Compression::Zstd,
                3 => Compression::Brotli,

                _ => anyhow::bail!("Unsupported compression variant")
            };

            // Read used compression's minimal block size.
            let mut block_size = [0; 2];

            file.read_exact(&mut block_size)?;

            let block_size = u16::from_be_bytes(block_size);

            if block_size == 0 {
                anyhow::bail!("Block size must be larger than 0");
            }

            Ok(Self {
                path,
                file: BufWriter::new(file),
                compression,
                block_size,
                buf: Vec::with_capacity(block_size as usize),
                _entity: PhantomData
            })
        }
    }

    /// Create new store, replace existing one.
    pub fn create(path: impl Into<PathBuf>, compression: Compression, block_size: u16) -> anyhow::Result<Self> {
        if block_size == 0 {
            anyhow::bail!("Block size must be larger than 0");
        }

        let path: PathBuf = path.into();

        let mut file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        // Used compression algorithm.
        // Reserve byte 0 for no compression.
        match compression {
            Compression::Lz4    => file.write_all(&[1])?,
            Compression::Zstd   => file.write_all(&[2])?,
            Compression::Brotli => file.write_all(&[3])?
        }

        // Write minimal block size.
        file.write_all(&block_size.to_be_bytes())?;

        Ok(Self {
            path,
            file: BufWriter::new(file),
            compression,
            block_size,
            buf: Vec::with_capacity(block_size as usize),
            _entity: PhantomData
        })
    }

    /// Insert entity to the store.
    pub fn insert(&mut self, entity: impl Into<Entity>) -> anyhow::Result<()> {
        let entity = entity.into().to_bytes();

        if entity.len() <= u16::MAX as usize {
            anyhow::bail!("Entities lengths must be under 65536 bytes (64 KB)");
        }

        let n = u16::try_from(self.buf.len())?;
        let m = u16::try_from(entity.len())?;

        fn write(file: &mut BufWriter<File>, data: &[u8], compression: &Compression) -> anyhow::Result<()> {
            let data = compression.compress(data)?;

            // Safe cast because we're making sure it's below u16.
            let n = data.len() as u16;

            file.write_all(&n.to_be_bytes())?;
            file.write_all(&data)?;

            Ok(())
        }

        match n.checked_add(m + 2) {
            Some(len) if len > self.block_size => {
                // Do not write empty buffers.
                if n > 0 {
                    write(&mut self.file, &self.buf, &self.compression)?;
                }

                self.buf.clear();
            }

            None => {
                // Do not write empty buffers.
                // Though this must be impossible in this case.
                if n > 0 {
                    write(&mut self.file, &self.buf, &self.compression)?;
                }

                self.buf.clear();
            }

            _ => ()
        }

        self.buf.extend_from_slice(&m.to_be_bytes());
        self.buf.extend_from_slice(&entity);

        Ok(())
    }

    /// Write all the entries stored in the RAM buffer on disk.
    pub fn flush(&mut self) -> anyhow::Result<()> {
        // Do not write empty buffer.
        if !self.buf.is_empty() {
            let data = self.compression.compress(&self.buf)?;

            // Safe cast because we're making sure it's below u16.
            let n = data.len() as u16;

            self.file.write_all(&n.to_be_bytes())?;
            self.file.write_all(&data)?;

            self.buf.clear();
        }

        self.file.flush()?;

        Ok(())
    }

    /// Open entities reader.
    pub fn read(&mut self) -> anyhow::Result<StorageReader<Entity>> {
        self.flush()?;

        let mut file = BufReader::new(File::open(&self.path)?);

        file.seek(SeekFrom::Start(Self::DATA_OFFSET))?;

        Ok(StorageReader {
            file,
            compression: self.compression,
            block: Vec::with_capacity(self.block_size as usize),
            block_ptr: 0,
            _entity: PhantomData
        })
    }
}

impl<Entry: Serialize> Drop for Storage<Entry> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[test]
fn test() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join("storage-test");

    for compression in [Compression::Lz4, Compression::Zstd, Compression::Brotli] {
        for block_size in [8, 16, 32, 64, 128] {
            let mut storage = Storage::<String>::create(&path, compression, block_size)?;

            storage.insert("Test entry 1")?;
            storage.insert("Test entry 2")?;
            storage.insert("Test entry 3")?;
            storage.insert("Test entry 4")?;
            storage.insert("Test entry 5")?;

            storage.flush()?;

            let mut reader = storage.read()?;

            assert_eq!(reader.next(), Some(String::from("Test entry 1")), "Failed for compression {compression:?} and block size {block_size}");
            assert_eq!(reader.next(), Some(String::from("Test entry 2")), "Failed for compression {compression:?} and block size {block_size}");
            assert_eq!(reader.next(), Some(String::from("Test entry 3")), "Failed for compression {compression:?} and block size {block_size}");
            assert_eq!(reader.next(), Some(String::from("Test entry 4")), "Failed for compression {compression:?} and block size {block_size}");
            assert_eq!(reader.next(), Some(String::from("Test entry 5")), "Failed for compression {compression:?} and block size {block_size}");
            assert_eq!(reader.next(), None,                               "Failed for compression {compression:?} and block size {block_size}");

            std::fs::remove_file(&path)?;
        }
    }

    Ok(())
}
