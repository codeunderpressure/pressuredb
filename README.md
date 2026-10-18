# PressureDB

Hey! I'm 17, and I got tired of building generic to-do apps and toy projects. I wanted to understand how real databases like SQLite and LMDB actually work under the hood, so I decided to build one from scratch in Rust.

This repository contains the code for my database building from scratch YouTube series on my YouTube channel **[Code Under Pressure](https://www.youtube.com/@CodeUnderPressure)**.

Everything here is written with zero external crates. No third-party dependencies, just raw bytes, and the Rust standard lib.

## Follow The Series

I am building this entire database live on YouTube, step by step:

* **YouTube Channel**: [Code Under Pressure](https://www.youtube.com/@CodeUnderPressure)
* **Series Playlist**: [Watch the Full Playlist](https://youtube.com/playlist?list=PLAAINjgS-3kQ&si=Si6hmZCU3LOtEg6r)
* **Episode 1**: Building the Core 4KB Page Layout & B-Tree from Scratch
* **Episode 2**: Adding a Free List to reclaim disk space (Coming next)

## Running It Locally

You just need Rust installed:

```bash
git clone https://github.com/codeunderpressure/pressuredb.git
cd pressuredb
cargo run
```

## A Quick Note on Pull Requests

Because this codebase directly follows the live YouTube videos episode by episode, **I will not be accepting pull requests** (otherwise the code on GitHub will not match what people see in the videos).

That said, if you find a bug, have an idea to make something cleaner, or want to see a specific feature in a future video, please **[open an Issue](https://github.com/codeunderpressure/pressuredb/issues)**!.

## Disclaimer

This project is made for learning and teaching database internals. It is meant to be fun and educational, not a production database to run your company on.
