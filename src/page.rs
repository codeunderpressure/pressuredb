use crate::error::DbError;
use std::cmp::Ordering;

pub const PAGE_SIZE: usize = 4096;
pub const HEADER_SIZE: usize = 4;
pub const TYPE_NODE: u16 = 1;
pub const TYPE_LEAF: u16 = 2;

#[derive(Clone, Copy)]
pub struct Page {
    pub data: [u8; PAGE_SIZE],
}

impl Page {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [0u8; PAGE_SIZE],
        }
    }

    #[must_use]
    pub fn btype(&self) -> u16 {
        u16::from_le_bytes([self.data[0], self.data[1]])
    }

    #[must_use]
    pub fn nkeys(&self) -> u16 {
        u16::from_le_bytes([self.data[2], self.data[3]])
    }

    pub fn set_header(&mut self, btype: u16, nkeys: u16) {
        self.data[0..2].copy_from_slice(&btype.to_le_bytes());
        self.data[2..4].copy_from_slice(&nkeys.to_le_bytes());
    }

    #[must_use]
    pub fn get_ptr(&self, idx: u16) -> u64 {
        let pos = HEADER_SIZE + 8 * usize::from(idx);
        u64::from_le_bytes(self.data[pos..pos + 8].try_into().unwrap_or([0; 8]))
    }

    pub fn set_ptr(&mut self, idx: u16, ptr: u64) {
        let pos = HEADER_SIZE + 8 * usize::from(idx);
        self.data[pos..pos + 8].copy_from_slice(&ptr.to_le_bytes());
    }

    fn offset_pos(&self, idx: u16) -> usize {
        let n = usize::from(self.nkeys());
        HEADER_SIZE + 8 * n + 2 * usize::from(idx.saturating_sub(1))
    }

    #[must_use]
    pub fn get_offset(&self, idx: u16) -> u16 {
        if idx == 0 {
            return 0;
        }
        let pos = self.offset_pos(idx);
        u16::from_le_bytes([self.data[pos], self.data[pos + 1]])
    }

    pub fn set_offset(&mut self, idx: u16, offset: u16) {
        let pos = self.offset_pos(idx);
        self.data[pos..pos + 2].copy_from_slice(&offset.to_le_bytes());
    }

    #[must_use]
    pub fn kv_pos(&self, idx: u16) -> usize {
        let n = usize::from(self.nkeys());
        HEADER_SIZE + 8 * n + 2 * n + usize::from(self.get_offset(idx))
    }

    #[must_use]
    pub fn get_key(&self, idx: u16) -> &[u8] {
        let pos = self.kv_pos(idx);
        let klen = usize::from(u16::from_le_bytes([self.data[pos], self.data[pos + 1]]));
        &self.data[pos + 4..pos + 4 + klen]
    }

    #[must_use]
    pub fn get_val(&self, idx: u16) -> &[u8] {
        let pos = self.kv_pos(idx);
        let klen = usize::from(u16::from_le_bytes([self.data[pos], self.data[pos + 1]]));
        let vlen = usize::from(u16::from_le_bytes([self.data[pos + 2], self.data[pos + 3]]));
        &self.data[pos + 4 + klen..pos + 4 + klen + vlen]
    }

    #[must_use]
    pub fn nbytes(&self) -> usize {
        self.kv_pos(self.nkeys())
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn node_lookup_le(page: &Page, key: &[u8]) -> u16 {
    let nkeys = page.nkeys();
    if nkeys <= 1 {
        return 0;
    }
    let mut low = 1;
    let mut high = nkeys - 1;
    let mut found = 0;
    while low <= high {
        let mid = u16::midpoint(low, high);
        let cmp = page.get_key(mid).cmp(key);
        if cmp == Ordering::Less || cmp == Ordering::Equal {
            found = mid;
            low = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        }
    }
    found
}

pub fn node_append_range(dst: &mut Page, src: &Page, dst_new: u16, src_old: u16, n: u16) {
    if n == 0 {
        return;
    }
    for i in 0..n {
        dst.set_ptr(dst_new + i, src.get_ptr(src_old + i));
    }

    let dst_begin = dst.get_offset(dst_new);
    let src_begin = src.get_offset(src_old);
    for i in 1..=n {
        let offset = dst_begin + src.get_offset(src_old + i) - src_begin;
        dst.set_offset(dst_new + i, offset);
    }

    let begin = src.kv_pos(src_old);
    let end = src.kv_pos(src_old + n);
    let dst_pos = dst.kv_pos(dst_new);
    dst.data[dst_pos..dst_pos + (end - begin)].copy_from_slice(&src.data[begin..end]);
}

pub(crate) fn node_append_kv(
    dst: &mut Page,
    idx: u16,
    ptr: u64,
    key: &[u8],
    val: &[u8],
) -> Result<(), DbError> {
    dst.set_ptr(idx, ptr);
    let pos = dst.kv_pos(idx);
    let klen = u16::try_from(key.len()).map_err(|_| DbError::KeyTooLarge)?;
    let vlen = u16::try_from(val.len()).map_err(|_| DbError::ValTooLarge)?;

    dst.data[pos..pos + 2].copy_from_slice(&klen.to_le_bytes());
    dst.data[pos + 2..pos + 4].copy_from_slice(&vlen.to_le_bytes());
    dst.data[pos + 4..pos + 4 + key.len()].copy_from_slice(key);
    dst.data[pos + 4 + key.len()..pos + 4 + key.len() + val.len()].copy_from_slice(val);

    let next_offset = dst.get_offset(idx) + 4 + klen + vlen;
    dst.set_offset(idx + 1, next_offset);
    Ok(())
}

pub(crate) fn leaf_insert(
    dst: &mut Page,
    src: &Page,
    idx: u16,
    key: &[u8],
    val: &[u8],
) -> Result<(), DbError> {
    dst.set_header(TYPE_LEAF, src.nkeys() + 1);
    node_append_range(dst, src, 0, 0, idx);
    node_append_kv(dst, idx, 0, key, val)?;
    node_append_range(dst, src, idx + 1, idx, src.nkeys() - idx);
    Ok(())
}

pub(crate) fn leaf_update(
    dst: &mut Page,
    src: &Page,
    idx: u16,
    key: &[u8],
    val: &[u8],
) -> Result<(), DbError> {
    dst.set_header(TYPE_LEAF, src.nkeys());
    node_append_range(dst, src, 0, 0, idx);
    node_append_kv(dst, idx, 0, key, val)?;
    node_append_range(dst, src, idx + 1, idx + 1, src.nkeys() - idx - 1);
    Ok(())
}
