use crate::error::DbError;
use crate::page::{Page, PAGE_SIZE};
use crate::tree::PageStore;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub const DB_SIG: &[u8; 16] = b"PressureDB01\0\0\0\0";

pub struct DiskManager {
    fp: File,
    pub root_page: u64,
    pub page_used: u64,
}

impl DiskManager {
    pub(crate) fn open<P: AsRef<Path>>(path: P) -> Result<Self, DbError> {
        let mut fp = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let len = fp.seek(SeekFrom::End(0))?;
        if len == 0 {
            let mut master = [0u8; PAGE_SIZE];
            master[0..16].copy_from_slice(DB_SIG);
            master[16..24].copy_from_slice(&0u64.to_le_bytes());
            master[24..32].copy_from_slice(&1u64.to_le_bytes());

            fp.seek(SeekFrom::Start(0))?;
            fp.write_all(&master)?;
            fp.sync_all()?;

            Ok(Self {
                fp,
                root_page: 0,
                page_used: 1,
            })
        } else {
            fp.seek(SeekFrom::Start(0))?;
            let mut master = [0u8; PAGE_SIZE];
            fp.read_exact(&mut master)?;

            if &master[0..16] != DB_SIG {
                return Err(DbError::CorruptedPage);
            }

            let root_page = u64::from_le_bytes(
                master[16..24]
                    .try_into()
                    .map_err(|_| DbError::CorruptedPage)?,
            );
            let page_used = u64::from_le_bytes(
                master[24..32]
                    .try_into()
                    .map_err(|_| DbError::CorruptedPage)?,
            );

            Ok(Self {
                fp,
                root_page,
                page_used,
            })
        }
    }

    pub(crate) fn read_page(&mut self, page_id: u64) -> Result<Page, DbError> {
        let pos = page_id * (PAGE_SIZE as u64);
        self.fp.seek(SeekFrom::Start(pos))?;
        let mut page = Page::new();
        self.fp.read_exact(&mut page.data)?;
        Ok(page)
    }

    pub(crate) fn write_page(&mut self, page_id: u64, page: &Page) -> Result<(), DbError> {
        let pos = page_id * (PAGE_SIZE as u64);
        self.fp.seek(SeekFrom::Start(pos))?;
        self.fp.write_all(&page.data)?;
        Ok(())
    }

    pub(crate) fn sync(&mut self) -> Result<(), DbError> {
        self.fp.sync_all()?;
        Ok(())
    }

    pub(crate) fn update_master(&mut self, root_page: u64) -> Result<(), DbError> {
        self.root_page = root_page;
        let mut master = [0u8; PAGE_SIZE];
        master[0..16].copy_from_slice(DB_SIG);
        master[16..24].copy_from_slice(&root_page.to_le_bytes());
        master[24..32].copy_from_slice(&self.page_used.to_le_bytes());

        self.fp.seek(SeekFrom::Start(0))?;
        self.fp.write_all(&master)?;
        self.sync()?;
        Ok(())
    }
}

impl PageStore for DiskManager {
    fn get_page(&mut self, page_id: u64) -> Result<Page, DbError> {
        self.read_page(page_id)
    }

    fn alloc_page(&mut self, page: &Page) -> Result<u64, DbError> {
        let page_id = self.page_used;
        self.page_used += 1;
        self.write_page(page_id, page)?;
        Ok(page_id)
    }
}
