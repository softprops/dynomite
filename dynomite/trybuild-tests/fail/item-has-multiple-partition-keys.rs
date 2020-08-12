use dynomite_derive::Item;

#[derive(Item)]
struct Foo {
    #[dynomite(partition_key)]
    key1: String,
    #[dynomite(partition_key)]
    key2: String
}

fn main() {}