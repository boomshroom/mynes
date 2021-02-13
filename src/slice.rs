use std::ops::Index;
use std::convert::TryFrom;

pub struct Slice<const SIZE: usize>(usize);

impl<T, const SIZE: usize> Index<Slice<SIZE>> for [T] {
	type Output = [T; SIZE];
	fn index(&self, idx: Slice<SIZE>) -> &[T; SIZE] {
		<&[T; SIZE]>::try_from(&self[idx.0..][..SIZE]).unwrap()
	}
}