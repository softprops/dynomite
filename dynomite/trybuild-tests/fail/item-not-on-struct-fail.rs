fn main() {

  fail();

}

#[derive(dynomite_derive::Item)]
fn fail() {
  println!("This should fail");
}
