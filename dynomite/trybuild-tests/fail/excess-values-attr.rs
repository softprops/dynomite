use dynomite_derive::Item;

#[derive(Item)]
pub struct S {
    #[dynomite(partition_key = "bar")]
    s: String,
}

fn main() {}
