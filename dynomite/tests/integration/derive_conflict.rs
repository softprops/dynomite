use dynomite_derive::Item;

#[allow(dead_code)]
type Result = std::result::Result<u8, u8>;

#[derive(Item, Debug)]
pub struct S {
    #[dynomite(partition_key)]
    s: String,
}
