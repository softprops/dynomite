use dynomite::{Attributes};

#[derive(Attributes)]
struct Test1 {
    #[dynomite(skip_serializing_if = "true")]
    field: u32,
}

#[derive(Attributes)]
struct Test2 {
    #[dynomite(skip_serializing_if = "2 + 2")]
    field: u32,
}

#[derive(Attributes)]
struct Test3 {
    #[dynomite(skip_serializing_if = "|| true")]
    field: u32,
}

#[derive(Attributes)]
struct Test4 {
    #[dynomite(skip_serializing_if = "invalid_fn")]
    field: u32,
}

fn invalid_fn() -> bool {
    true
}

#[derive(Attributes)]
struct Test5 {
    #[dynomite(skip_serializing_if = "module::invalid_fn_in_module")]
    field: u32,
}

mod module {
    pub(super) fn invalid_fn_in_module() {}
}

fn main() {}
