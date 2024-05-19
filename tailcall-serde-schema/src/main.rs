mod de;
mod schema;

use schema::Schema;

fn main() {
    let schema = Schema::String;
    let input = r#""Hello World!""#;
    let value = schema.deserialize(input);

    print!("{:?}", value);
}
