use dynomite_derive::Item;

#[derive(Item)]
struct Foo {
    #[dynomite(partition_key)]
    key: String,
    #[dynomite(typo)]
    fail: String
}

fn main() {}