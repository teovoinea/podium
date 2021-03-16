#[macro_use]
extern crate criterion;
use criterion::async_executor::AsyncExecutor;
use criterion::Criterion;
use criterion::*;

use contracts::file_to_process::new_file_to_process;
use contracts::indexer::Indexer;
use spreadsheet_indexer::spreadsheet_indexer::SpreadsheetIndexer;
use std::path::Path;
use tokio::runtime::Runtime;

use common::tokio;

fn bench_indexing_spreadsheet_file(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let test_file_path = Path::new("../../../test_files/Cats.xlsx");
    let ftp = rt.block_on(new_file_to_process(test_file_path));

    c.bench_function("indexing_spreadsheet_file", |b| {
        b.iter(|| {
            let _indexed_document = SpreadsheetIndexer.index_file(&ftp).unwrap();
        });
    });
}

criterion_group!(benches, bench_indexing_spreadsheet_file,);

criterion_main!(benches);
