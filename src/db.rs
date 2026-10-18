use crate::disk::DiskManager;
use crate::error::DbError;
use crate::tree::{tree_get, tree_insert};
use std::path::Path;

pub struct Db {
    disk: DiskManager,
}

impl Db {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DbError> {
        let disk = DiskManager::open(path)?;
        Ok(Self { disk })
    }

    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        let root = self.disk.root_page;
        tree_get(&mut self.disk, root, key)
    }

    pub fn set(&mut self, key: &[u8], val: &[u8]) -> Result<(), DbError> {
        let root = self.disk.root_page;
        let new_root = tree_insert(&mut self.disk, root, key, val)?;
        // Flush new tree pages to disk before updating the root pointer
        self.disk.sync()?;
        // Atomically update master page 0 with the new root pointer
        self.disk.update_master(new_root)?;
        Ok(())
    }

    #[must_use]
    pub fn page_count(&self) -> u64 {
        self.disk.page_used
    }

    #[must_use]
    pub fn root_page(&self) -> u64 {
        self.disk.root_page
    }
}
