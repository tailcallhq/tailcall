mod de;
mod ignore;
mod schema;

use schema::Schema;

fn main() {
    let schema = Schema::String;
    let input = r#""Hello World!""#;
    let value = schema.deserialize(input);
    serde_json::from_str::<serde_json::Value>(r#""Hello World!""#)
        .expect("Failed to deserialize JSON string");

    print!("{:?}", value);
}
