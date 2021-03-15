use common::tantivy::schema::*;
use common::tracing::info;

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
