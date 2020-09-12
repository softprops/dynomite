use dynomite_derive::Attributes;

#[derive(Attributes)]
pub enum MyEnum {
    Foo(Foo),
}

#[derive(Attributes)]
pub struct Foo {
    s: String,
}

fn main() {}
