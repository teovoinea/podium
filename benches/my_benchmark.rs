
#[macro_use]
extern crate criterion;
extern crate podium_lib;

use podium_lib::indexers::*;

use criterion::Criterion;
use criterion::black_box;

use std::path::Path;

fn bench_indexing_csv_file(c: &mut Criterion) {
    c.bench_function("indexing_csv_file", |b| {
        b.iter(|| {
            let bench_file_path = black_box(Path::new("./test_files/data.csv"));
            CsvIndexer.index_file(bench_file_path)
        });
    });
}

fn bench_indexing_exif_file(c: &mut Criterion) {
    c.bench_function("indexing_exif_file", |b| {
        b.iter(|| {
            let bench_file_path = black_box(Path::new("./test_files/IMG_2551.jpeg"));
            ExifIndexer.index_file(bench_file_path)
        });
    });
}

#[cfg(not(target_os = "windows"))]
fn bench_indexing_mobile_net_v2_file(c: &mut Criterion) {
    c.bench_function("indexing_mobile_net_v2_file", |b| {
        b.iter(|| {
            let bench_file_path = black_box(Path::new("./test_files/IMG_2551.jpeg"));
            MobileNetV2Indexer.index_file(bench_file_path)
        });
    });
}

fn bench_indexing_pdf_file(c: &mut Criterion) {
    c.bench_function("indexing_pdf_file", |b| {
        b.iter(|| {
            let bench_file_path = black_box(Path::new("./test_files/Cats.pdf"));
            PdfIndexer.index_file(bench_file_path)
        });
    });
}

fn bench_indexing_pptx_file(c: &mut Criterion) {
    c.bench_function("indexing_pptx_file", |b| {
        b.iter(|| {
            let bench_file_path = black_box(Path::new("./test_files/Cats.pptx"));
            PptxIndexer.index_file(bench_file_path)
        });
    });
}

fn bench_indexing_spreadsheet_file(c: &mut Criterion) {
    c.bench_function("indexing_spreadsheet_file", |b| {
        b.iter(|| {
            let bench_file_path = black_box(Path::new("./test_files/Cats.xlsx"));
            SpreadsheetIndexer.index_file(bench_file_path)
        });
    });
}

fn bench_indexing_text_file(c: &mut Criterion) {
    c.bench_function("indexing_text_file", |b| {
        b.iter(|| {
            let bench_file_path = black_box(Path::new("./test_files/file.txt"));
            TextIndexer.index_file(bench_file_path)
        });
    });
}

#[cfg(not(target_os = "windows"))]
criterion_group!(benches,
                bench_indexing_csv_file,
                bench_indexing_exif_file,
                bench_indexing_mobile_net_v2_file,
                bench_indexing_pdf_file,
                bench_indexing_exif_file,
                bench_indexing_pptx_file,
                bench_indexing_spreadsheet_file,
                bench_indexing_text_file,);

#[cfg(target_os = "windows")]
criterion_group!(benches,
                bench_indexing_csv_file,
                bench_indexing_exif_file,
                bench_indexing_pdf_file,
                bench_indexing_exif_file,
                bench_indexing_pptx_file,
                bench_indexing_spreadsheet_file,
                bench_indexing_text_file,);
                
criterion_main!(benches);