use crate::error::DbError;
use crate::page::{
    leaf_insert, leaf_update, node_append_kv, node_append_range, node_lookup_le, Page, PAGE_SIZE,
    TYPE_LEAF, TYPE_NODE,
};

pub(crate) trait PageStore {
    fn get_page(&mut self, page_id: u64) -> Result<Page, DbError>;
    fn alloc_page(&mut self, page: &Page) -> Result<u64, DbError>;
}

pub(crate) fn node_split2(left: &mut Page, right: &mut Page, old: &Page) {
    let nkeys = old.nkeys();
    let mid = nkeys / 2;

    left.set_header(old.btype(), mid);
    node_append_range(left, old, 0, 0, mid);

    let right_keys = nkeys - mid;
    right.set_header(old.btype(), right_keys);
    node_append_range(right, old, 0, mid, right_keys);
}

pub(crate) fn tree_get<S: PageStore>(
    store: &mut S,
    page_id: u64,
    key: &[u8],
) -> Result<Option<Vec<u8>>, DbError> {
    if page_id == 0 {
        return Ok(None);
    }
    let page = store.get_page(page_id)?;
    let nkeys = page.nkeys();
    if nkeys == 0 {
        return Ok(None);
    }

    let idx = node_lookup_le(&page, key);
    match page.btype() {
        TYPE_LEAF => {
            if page.get_key(idx) == key {
                Ok(Some(page.get_val(idx).to_vec()))
            } else {
                Ok(None)
            }
        }
        TYPE_NODE => {
            let child_id = page.get_ptr(idx);
            tree_get(store, child_id, key)
        }
        _ => Err(DbError::CorruptedPage),
    }
}

pub(crate) fn tree_insert<S: PageStore>(
    store: &mut S,
    root_id: u64,
    key: &[u8],
    val: &[u8],
) -> Result<u64, DbError> {
    if root_id == 0 {
        let mut root = Page::new();
        root.set_header(TYPE_LEAF, 1);
        node_append_kv(&mut root, 0, 0, key, val)?;
        return store.alloc_page(&root);
    }

    let page = store.get_page(root_id)?;
    let mut updated = Page::new();

    let idx = node_lookup_le(&page, key);
    match page.btype() {
        TYPE_LEAF => {
            if page.nkeys() == 0 {
                leaf_insert(&mut updated, &page, 0, key, val)?;
            } else if page.get_key(idx) == key {
                leaf_update(&mut updated, &page, idx, key, val)?;
            } else {
                let insert_idx = if key < page.get_key(0) { 0 } else { idx + 1 };
                leaf_insert(&mut updated, &page, insert_idx, key, val)?;
            }
        }
        TYPE_NODE => {
            let child_id = page.get_ptr(idx);
            let new_child = tree_insert(store, child_id, key, val)?;
            updated.set_header(TYPE_NODE, page.nkeys());
            node_append_range(&mut updated, &page, 0, 0, page.nkeys());
            updated.set_ptr(idx, new_child);
        }
        _ => return Err(DbError::CorruptedPage),
    }

    if updated.nbytes() > PAGE_SIZE {
        let mut left = Page::new();
        let mut right = Page::new();
        node_split2(&mut left, &mut right, &updated);

        let left_id = store.alloc_page(&left)?;
        let right_id = store.alloc_page(&right)?;

        let mut new_root = Page::new();
        new_root.set_header(TYPE_NODE, 2);
        node_append_kv(&mut new_root, 0, left_id, left.get_key(0), &[])?;
        node_append_kv(&mut new_root, 1, right_id, right.get_key(0), &[])?;
        store.alloc_page(&new_root)
    } else {
        store.alloc_page(&updated)
    }
}
