use dynomite_derive::Attributes;

#[derive(Attributes)]
#[dynomite(tag = "kind")]
enum Foo {
    Bar(Bar),
    #[dynomite(rename = "Bar")]
    Baz(Bar),
    Bruh(Bar),
}

#[derive(Attributes)]
struct Bar {}

fn main() {}
