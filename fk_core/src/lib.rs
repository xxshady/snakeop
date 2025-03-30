pub fn def<T: Default>() -> T {
  Default::default()
}

#[derive(Clone, Copy, PartialEq)]
pub struct Entity(pub u64);
