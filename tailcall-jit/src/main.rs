struct Field {
    name: String,
    selection: Selection,
}

struct Selection {
    fields: Vec<Field>,
}

fn main() {
    println!("Hello, world!");
}
