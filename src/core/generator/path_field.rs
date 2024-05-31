pub enum PathField {
    EnumType,
    MessageType,
    Service,
    Field,
    Method,
    EnumValue,
    NestedType,
}

impl PathField {
    pub fn value(&self) -> i32 {
        match self {
            PathField::EnumType => 5,
            PathField::MessageType => 4,
            PathField::Service => 6,
            PathField::Field => 2,
            PathField::Method => 2,
            PathField::EnumValue => 2,
            PathField::NestedType => 3,
        }
    }
}
