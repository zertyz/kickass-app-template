#![allow(dead_code)]
//! Contains code for handling the Cargo features used to compile this application.
//! Currently, we have:
//! * Global allocator options:
//!   - **std allocator**:                      `RUSTFLAGS="-C target-cpu=native" cargo build --release --features=""`
//!   - **mimalloc**:                           `RUSTFLAGS="-C target-cpu=native" cargo build --release --features="mimalloc_allocator"`
//!   - **jemallocator**:                       `RUSTFLAGS="-C target-cpu=native" cargo build --release --features="jemalloc_allocator"`
//!   - **tcmalloc** *(with system libs)*:      `RUSTFLAGS="-C target-cpu=native" cargo build --release --features="tcmalloc_allocator"`
//!   - **tcmalloc** *(with bundled libs)*:     `RUSTFLAGS="-C target-cpu=native" cargo build --release --features="tcmalloc_allocator_bundled"`
//! * Please consult `Cargo.toml` to see what are the default `features`
//!
//! # Performance benchmarks
//! Command used:\
//! ``
//! for allocator in tcmalloc_allocator tcmalloc_allocator_bundled jemallocator_allocator mimalloc_allocator std_allocator; do
//!   for parallel in On Off; do
//!     for i in 1 2 3 4 5; do
//!         RUSTFLAGS="-C target-cpu=native" cargo build --release --features="${allocator}";
//!         sudo sync;
//!         pkill -f -stop firefox; pkill -f -stop chrome; pkill -f -stop java;
//!         echo 3 | sudo tee /proc/sys/vm/drop_caches;
//!         ./target/release/ogre-datasets-converter --help; sleep 15;
//!         clear; sudo nice -n -20 /usr/sbin/time -v ./target/release/ogre-datasets-converter --raw-input-dir /tmp/datasets --rkyv-output-dir /tmp/datasets update -d Trades -p ${parallel} 2>&1 | tee /tmp/ogre-datasets-converter--parallel_${parallel}-${allocator}-`date +%s`.out;
//!         pkill -f -cont firefox; pkill -f -cont chrome; pkill -f -cont java;
//!     done;
//!   done;
//! done
//! ``
//!
//! ## Non swapping scenario:
//! ### Results for Parallel algorithm (best of 5):
//!
//! | Allocator |  Elapsed  | Max RSS | IO Page Faults | Minor Page Faults | Voluntary Context Switches | Involuntary CSes | % of CPU |
//! |-----------|-----------|---------|----------------|-------------------|----------------------------|------------------|----------|
//! | tcmalloc  |  7:42.19  | 13.97g  | 17             | 2,097,280         | 129,411                    | 131,653          | 661%     |
//! | mimalloc  |  8:09.13  | 13.55g  | 7557           | 4,525,509         | 140,768                    | 132,270          | 662%     |
//! | stdalloc  | 10:44.19  | 10.05g  | 12             | 3,069,236         | 3,244,717                  | 123,741          | 663%     |
//! | jemalloc  | 10:46.94  | 10.11g  | 10             | 3,089,371         | 2,974,079                  | 124,892          | 663%     |
//! | tc bundled| 10:54.81  | 10.05g  | 6              | 3,059,233         | 3,529,529                  | 126,424          | 664%     |
//!
//!
//! ### Results for Serial algorithm (best of 5):
//!
//! | Allocator |  Elapsed  | Max RSS | IO Page Faults | Minor Page Faults | Voluntary Context Switches | Involuntary CSes | % of CPU |
//! |-----------|-----------|---------|----------------|-------------------|----------------------------|------------------|----------|
//! | tcmalloc  | 14:40.18  | 13.47g  | 12             | 1,967,302         | 149,214                    | 2,761            | 95%      |
//! | mimalloc  | 15:32.42  | 12.34g  | 6              | 4,202,377         | 148,460                    | 2,597            | 95%      |
//! | jemalloc  | 18:43.34  |  9.63g  | 9              | 2,769,881         | 144,705                    | 3,104            | 95%      |
//! | stdalloc  | 19:08.92  |  9.63g  | 7              | 2,780,021         | 143,930                    | 3,163            | 95%      |
//!
//! ## Swapping scenario:
//!
//! The loading algorithm for "Trades" does well on swapping scenario as well, without major
//! delays and keeping CPU 97~99% busy-- this was tested on a 2GiB RAM virtualized box with 10G of
//! swapping on an SSD drive, producing a 7GiB RKYV dataset in 30 min (before algorithm
//! optimizations) and on 18 min after the optimizations.
//!
//! Interestingly, tc bundled performed the best on that machine... so testing the different allocators
//! might be part of the Continuous Integration scripts for this application.

// custom global allocator
#[global_allocator]
#[cfg(feature = "mimalloc_allocator")]
static MI_MALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(feature = "jemallocator_allocator")]
static JE_MALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
#[cfg(any(feature = "tcmalloc_allocator", feature = "tcmalloc_allocator_bundled"))]
static TC_MALLOC: tcmalloc::TCMalloc = tcmalloc::TCMalloc;

