use dynomite_derive::Attributes;

#[derive(Attributes)]
struct Foo {
    #[dynomite(rename)]
    val: u32
}

fn main() {}
