#[macro_use]
extern crate criterion;

// fn bench_indexing_pdf_file(c: &mut Criterion) {
//     let rt = Runtime::new().unwrap();
//     let test_file_path = Path::new("../../../test_files/Cats.pdf");
//     let ftp = rt.block_on(new_file_to_process(test_file_path));

//     c.bench_function("indexing_pdf_file", |b| {
//         b.iter(|| {
//             let indexed_document = PdfIndexer
//                 .index_file(&ftp)
//                 .unwrap();
//         });
//     });
// }

// criterion_group!(
//     benches,
//     bench_indexing_pdf_file,
// );

// criterion_main!(benches);

fn main() {}
