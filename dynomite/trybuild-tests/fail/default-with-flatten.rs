use dynomite_derive::Attributes;

#[derive(Attributes)]
struct Foo {
    #[dynomite(default, flatten)]
    flat: Flattened
}

struct Flattened {
    a: u32,
}


fn main() {}
