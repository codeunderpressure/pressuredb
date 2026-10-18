mod db;
mod disk;
mod error;
mod page;
mod tree;

use db::Db;
use error::DbError;
use std::fs;
use std::path::Path;

fn main() -> Result<(), DbError> {
    let db_path = Path::new("test.db");
    if db_path.exists() {
        let _ = fs::remove_file(db_path);
    }

    println!("==================================================");
    println!(" PressureDB: 4KB Page Persistent B-Tree Storage");
    println!(" Built from scratch in Rust");
    println!("==================================================");

    let mut db = Db::open(db_path)?;
    println!("[1] Database initialized at {}", db_path.display());
    println!("    Initial pages used: {}", db.page_count());

    println!("\n[2] Testing single key write...");
    db.set(b"channel", b"Code Under Pressure")?;
    println!("    Wrote key: 'channel' -> 'Code Under Pressure'");
    println!("    Total 4KB pages on disk: {}", db.page_count());
    println!("    B-Tree root page ID: {}", db.root_page());

    if let Some(val) = db.get(b"channel")? {
        println!("    Successfully read key 'channel': {}", String::from_utf8_lossy(&val));
    }

    println!("\n[3] Testing multi-key insertion...");
    db.set(b"language", b"Rust")?;
    println!("    Wrote key: 'language' -> 'Rust'");

    db.set(b"storage", b"4096-byte pages")?;
    println!("    Wrote key: 'storage' -> '4096-byte pages'");

    println!("\n[4] Querying keys back from disk...");
    if let Some(val) = db.get(b"channel")? {
        println!("    Found 'channel': {}", String::from_utf8_lossy(&val));
    }
    if let Some(val) = db.get(b"language")? {
        println!("    Found 'language': {}", String::from_utf8_lossy(&val));
    }
    if let Some(val) = db.get(b"storage")? {
        println!("    Found 'storage': {}", String::from_utf8_lossy(&val));
    }

    println!("\nDatabase run complete!");
    Ok(())
}
