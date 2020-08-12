use dynomite_derive::Item;

fn main() {

  fail();

}

#[derive(Item)]
fn fail() {
  println!("This should fail");
}
