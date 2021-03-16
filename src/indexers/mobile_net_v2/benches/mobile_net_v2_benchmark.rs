#[macro_use]
extern crate criterion;
use criterion::Criterion;
use criterion::*;
use criterion::{async_executor::AsyncExecutor, black_box};

use contracts::file_to_process::new_file_to_process;
use contracts::indexer::Indexer;
use mobile_net_v2_indexer::mobile_net_v2_indexer::MobileNetV2Indexer;
use std::path::Path;
use tokio::runtime::Runtime;

use common::tokio;

fn bench_indexing_mobile_net_v2_file(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let test_file_path = Path::new("../../../test_files/IMG_2551.jpeg");
    let ftp = rt.block_on(new_file_to_process(test_file_path));

    c.bench_function("indexing_mobile_net_v2_file", |b| {
        b.iter(|| {
            let indexed_document = MobileNetV2Indexer.index_file(&ftp).unwrap();
        });
    });
}

criterion_group!(benches, bench_indexing_mobile_net_v2_file,);

criterion_main!(benches);
