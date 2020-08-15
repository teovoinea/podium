use tantivy::schema::*;
use tracing::info;

use blake2b_simd::blake2b;

pub fn destructure_schema(schema: &Schema) -> (Field, Field, Field, Field) {
    (
        schema.get_field("title").unwrap(),
        schema.get_field("hash").unwrap(),
        schema.get_field("location").unwrap(),
        schema.get_field("body").unwrap(),
    )
}

/// Builds the tantivy schema
pub fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);

    schema_builder.add_text_field("hash", STRING | STORED);

    schema_builder.add_facet_field("location");

    schema_builder.add_text_field("body", TEXT | STORED);

    schema_builder.build()
}

pub fn calculate_hash(input: &[u8]) -> blake2b_simd::Hash {
    let file_hash = blake2b(input);
    info!("Hash of file is: {:?}", file_hash);
    file_hash
}
