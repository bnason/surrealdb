use crate::sql::fmt::Fmt;
use crate::sql::idiom::Idiom;
use crate::sql::Value;
use crate::sql::Part;
use revision::revisioned;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::{cmp, fmt};

#[revisioned(revision = 1)]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[non_exhaustive]
pub enum Ordering {
	Random,
	Order(OrderList),
}

impl fmt::Display for Ordering {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Ordering::Random => write!(f, "ORDER BY RAND()"),
			Ordering::Order(list) => writeln!(f, "ORDER BY {list}"),
		}
	}
}

#[revisioned(revision = 1)]
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[non_exhaustive]
pub struct OrderList(pub Vec<Order>);

impl Deref for OrderList {
	type Target = Vec<Order>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl fmt::Display for OrderList {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", Fmt::comma_separated(&self.0))
	}
}

impl OrderList {
	pub(crate) fn compare(&self, a: &Value, b: &Value) -> cmp::Ordering {
		for order in &self.0 {
			let parts = order.get_parts();
			
			// Reverse the ordering if DESC
			let o = match order.direction {
				Value::Bool(true) => a.compare(b, &parts, order.collate, order.numeric),
				Value::Bool(false) => b.compare(a, &parts, order.collate, order.numeric),
				Value::Param(_) => a.compare(b, &parts, order.collate, order.numeric), // Default to ASC for parameters
				_ => a.compare(b, &parts, order.collate, order.numeric), // Default to ASC
			};
			//
			match o {
				Some(cmp::Ordering::Greater) => return cmp::Ordering::Greater,
				Some(cmp::Ordering::Equal) => continue,
				Some(cmp::Ordering::Less) => return cmp::Ordering::Less,
				None => continue,
			}
		}
		cmp::Ordering::Equal
	}
}

#[revisioned(revision = 1)]
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[non_exhaustive]
pub struct Order {
	/// The value to order by
	pub value: Value,
	pub collate: bool,
	pub numeric: bool,
	/// The direction to order by (true for ASC, false for DESC)
	/// Can be a parameter or a boolean
	pub direction: Value,
}

impl fmt::Display for Order {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.value)?;
		if self.collate {
			write!(f, " COLLATE")?;
		}
		if self.numeric {
			write!(f, " NUMERIC")?;
		}
		match &self.direction {
			Value::Bool(false) => write!(f, " DESC")?,
			Value::Param(p) => write!(f, " ${}", p.0)?,
			_ => {} // ASC is default, no need to write it
		}
		Ok(())
	}
}

impl Order {
	fn get_parts(&self) -> Vec<Part> {
		if let Value::Idiom(idiom) = &self.value {
			idiom.0.clone()
		} else {
			vec![Part::Start(self.value.clone())]
		}
	}
}

#[revisioned(revision = 1)]
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[non_exhaustive]
pub struct OldOrders(pub Vec<OldOrder>);

#[revisioned(revision = 1)]
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[non_exhaustive]
pub struct OldOrder {
	pub order: Idiom,
	pub random: bool,
	pub collate: bool,
	pub numeric: bool,
	/// true if the direction is ascending
	pub direction: bool,
}
