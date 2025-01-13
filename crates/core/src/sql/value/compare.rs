use crate::sql::part::Next;
use crate::sql::part::Part;
use crate::sql::value::Value;
use std::cmp::Ordering;

impl Value {
	pub(crate) fn compare(
		&self,
		other: &Self,
		path: &[Part],
		collate: bool,
		numeric: bool,
	) -> Option<Ordering> {
		println!("Value::compare: Comparing: {:?} and {:?}", self, other);
		println!("Value::compare: Path: {:?}", path);
		println!("Value::compare: Collate: {:?}", collate);
		println!("Value::compare: Numeric: {:?}", numeric);

		let res = match path.first() {
			// Get the current path part
			Some(p) => match (self, other) {
				// Current path part is an object
				(Value::Object(a), Value::Object(b)) => match p {
					Part::Field(f) => match (a.get(f.as_str()), b.get(f.as_str())) {
						(Some(a), Some(b)) => {
							println!("Value::compare: Comparing object some and some: {:?} and {:?}", a, b);
							a.compare(b, path.next(), collate, numeric)
						},
						(Some(_), None) => {
							println!("Value::compare: Comparing object some and none: {:?} and {:?}", a, b);
							Some(Ordering::Greater)
						},
						(None, Some(_)) => {
							println!("Value::compare: Comparing object none and some: {:?} and {:?}", a, b);
							Some(Ordering::Less)
						},
						(_, _) => {
							println!("Value::compare: Comparing object none and none: {:?} and {:?}", a, b);
							Some(Ordering::Equal)
						},
					},
					_ => {
						println!("Value::compare: Comparing object default fields: {:?} and {:?}", a, b);
						None
					},
				},
				// Current path part is an array
				(Value::Array(a), Value::Array(b)) => match p {
					Part::All => {
						println!("Value::compare: Comparing arrays: {:?} and {:?}", a, b);
						for (a, b) in a.iter().zip(b.iter()) {
							match a.compare(b, path.next(), collate, numeric) {
								Some(Ordering::Equal) => continue,
								None => continue,
								o => return o,
							}
						}
						println!("Value::compare: Comparing array lengths: {:?} and {:?}", a.len(), b.len());
						match (a.len(), b.len()) {
							(a, b) if a > b => Some(Ordering::Greater),
							(a, b) if a < b => Some(Ordering::Less),
							_ => Some(Ordering::Equal),
						}
					}
					Part::First => {
						println!("Value::compare: Comparing array first: {:?} and {:?}", a, b);
						match (a.first(), b.first()) {
							(Some(a), Some(b)) => a.compare(b, path.next(), collate, numeric),
							(Some(_), None) => Some(Ordering::Greater),
							(None, Some(_)) => Some(Ordering::Less),
							(_, _) => Some(Ordering::Equal),
						}
					}
					Part::Last => {
						println!("Value::compare: Comparing array last: {:?} and {:?}", a, b);
						match (a.last(), b.last()) {
							(Some(a), Some(b)) => a.compare(b, path.next(), collate, numeric),
							(Some(_), None) => Some(Ordering::Greater),
							(None, Some(_)) => Some(Ordering::Less),
							(_, _) => Some(Ordering::Equal),
						}
					}
					Part::Index(i) => {
						println!("Value::compare: Comparing array index: {:?} and {:?}", a, b);
						match (a.get(i.to_usize()), b.get(i.to_usize())) {
							(Some(a), Some(b)) => a.compare(b, path.next(), collate, numeric),
							(Some(_), None) => Some(Ordering::Greater),
							(None, Some(_)) => Some(Ordering::Less),
							(_, _) => Some(Ordering::Equal),
						}
					}
					_ => {
						println!("Value::compare: Comparing array default: {:?} and {:?}", a, b);
						for (a, b) in a.iter().zip(b.iter()) {
							match a.compare(b, path, collate, numeric) {
								Some(Ordering::Equal) => continue,
								None => continue,
								o => return o,
							}
						}
						println!("Value::compare: Comparing array lengths: {:?} and {:?}", a.len(), b.len());
						match (a.len(), b.len()) {
							(a, b) if a > b => Some(Ordering::Greater),
							(a, b) if a < b => Some(Ordering::Less),
							_ => Some(Ordering::Equal),
						}
					}
				},
				// Ignore everything else
				(a, b) => {
					println!("Value::compare: Comparing default: {:?} and {:?}", a, b);
					a.compare(b, path.next(), collate, numeric)
				},
			},
			// No more parts so get the value
			None => match (collate, numeric) {
				(true, true) => {
					println!("Value::compare: Comparing natural lexical: {:?} and {:?}", self, other);
					self.natural_lexical_cmp(other)
				},
				(true, false) => {
					println!("Value::compare: Comparing lexical: {:?} and {:?}", self, other);
					self.lexical_cmp(other)
				},
				(false, true) => {
					println!("Value::compare: Comparing natural: {:?} and {:?}", self, other);
					self.natural_cmp(other)
				},
				_ => {
					println!("Value::compare: Comparing partial: {:?} and {:?}", self, other);
					self.partial_cmp(other)
				},
			},
		};

		println!("Value::compare: Result: {:?}", res);
		res
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use crate::sql::idiom::Idiom;
	use crate::syn::Parse;

	#[test]
	fn compare_none() {
		let idi = Idiom::default();
		let one = Value::parse("{ test: { other: null, something: 456 } }");
		let two = Value::parse("{ test: { other: null, something: 123 } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Greater));
	}

	#[test]
	fn compare_basic() {
		let idi = Idiom::parse("test.something");
		let one = Value::parse("{ test: { other: null, something: 456 } }");
		let two = Value::parse("{ test: { other: null, something: 123 } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Greater));
	}

	#[test]
	fn compare_basic_missing_left() {
		let idi = Idiom::parse("test.something");
		let one = Value::parse("{ test: { other: null } }");
		let two = Value::parse("{ test: { other: null, something: 123 } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Less));
	}

	#[test]
	fn compare_basic_missing_right() {
		let idi = Idiom::parse("test.something");
		let one = Value::parse("{ test: { other: null, something: 456 } }");
		let two = Value::parse("{ test: { other: null } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Greater));
	}

	#[test]
	fn compare_array() {
		let idi = Idiom::parse("test.something.*");
		let one = Value::parse("{ test: { other: null, something: [4, 5, 6] } }");
		let two = Value::parse("{ test: { other: null, something: [1, 2, 3] } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Greater));
	}

	#[test]
	fn compare_array_longer_left() {
		let idi = Idiom::parse("test.something.*");
		let one = Value::parse("{ test: { other: null, something: [1, 2, 3, 4, 5, 6] } }");
		let two = Value::parse("{ test: { other: null, something: [1, 2, 3] } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Greater));
	}

	#[test]
	fn compare_array_longer_right() {
		let idi = Idiom::parse("test.something.*");
		let one = Value::parse("{ test: { other: null, something: [1, 2, 3] } }");
		let two = Value::parse("{ test: { other: null, something: [1, 2, 3, 4, 5, 6] } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Less));
	}

	#[test]
	fn compare_array_missing_left() {
		let idi = Idiom::parse("test.something.*");
		let one = Value::parse("{ test: { other: null, something: null } }");
		let two = Value::parse("{ test: { other: null, something: [1, 2, 3] } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Less));
	}

	#[test]
	fn compare_array_missing_right() {
		let idi = Idiom::parse("test.something.*");
		let one = Value::parse("{ test: { other: null, something: [4, 5, 6] } }");
		let two = Value::parse("{ test: { other: null, something: null } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Greater));
	}

	#[test]
	fn compare_array_missing_value_left() {
		let idi = Idiom::parse("test.something.*");
		let one = Value::parse("{ test: { other: null, something: [1, null, 3] } }");
		let two = Value::parse("{ test: { other: null, something: [1, 2, 3] } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Less));
	}

	#[test]
	fn compare_array_missing_value_right() {
		let idi = Idiom::parse("test.something.*");
		let one = Value::parse("{ test: { other: null, something: [1, 2, 3] } }");
		let two = Value::parse("{ test: { other: null, something: [1, null, 3] } }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Greater));
	}

	#[test]
	fn compare_last() {
		let idi = Idiom::parse("test[$]");
		let one = Value::parse("{ test: [1,5] }");
		let two = Value::parse("{ test: [2,4] }");
		let res = one.compare(&two, &idi, false, false);
		assert_eq!(res, Some(Ordering::Greater))
	}
}
