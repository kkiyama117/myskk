mod key_event;
use itertools::Itertools;
use key_event::StringExt;

fn main() {
  let input = "a i C-S-r";
  println!("{:?}", input.as_key_events().collect_vec());

  let input = "(shift)";
  println!("{:?}", input.as_key_events().collect_vec());

  let input = "foo bar-a b";
  println!("{:?}", input.as_key_events().collect_vec());
}
