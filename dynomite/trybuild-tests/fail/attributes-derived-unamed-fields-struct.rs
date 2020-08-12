use dynomite_derive::Item;

#[derive(Item)]
struct Foo {
    #[dynomite(partition_key)]
    key1: String,
    #[dynomite(typo)]
    key2: String
}

fn main() {}