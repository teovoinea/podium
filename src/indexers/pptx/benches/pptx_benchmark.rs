#[macro_use]
extern crate criterion;
use criterion::Criterion;
use criterion::*;
use criterion::{async_executor::AsyncExecutor, black_box};

use contracts::file_to_process::new_file_to_process;
use contracts::indexer::Indexer;
use pptx_indexer::pptx_indexer::PptxIndexer;
use std::path::Path;
use tokio::runtime::Runtime;

use common::tokio;

fn bench_indexing_pptx_file(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let test_file_path = Path::new("../../../test_files/Cats.pptx");
    let ftp = rt.block_on(new_file_to_process(test_file_path));

    c.bench_function("indexing_pptx_file", |b| {
        b.iter(|| {
            let indexed_document = PptxIndexer.index_file(&ftp).unwrap();
        });
    });
}

criterion_group!(benches, bench_indexing_pptx_file,);

criterion_main!(benches);
