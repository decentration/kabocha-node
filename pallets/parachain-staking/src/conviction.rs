//! The conviction datatype.

use crate::types::Follows;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Bounded, CheckedDiv, CheckedMul, Zero},
	RuntimeDebug,
};
use sp_std::{prelude::*, result::Result};

/// A value denoting the strength of conviction of a vote.
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub enum Conviction {
	/// 0.1x stake, unlocked.
	None,
	/// 1x stake, locked for an enactment period following a successful vote.
	Locked1x,
	/// 2x stake, locked for 2x enactment periods following a successful vote.
	Locked2x,
	/// 3x stake, locked for 4x...
	Locked3x,
	/// 4x stake, locked for 8x...
	Locked4x,
	/// 5x stake, locked for 16x...
	Locked5x,
	/// 6x stake, locked for 32x...
	Locked6x,
}

impl Default for Conviction {
	fn default() -> Self {
		Conviction::None
	}
}

impl From<Conviction> for u8 {
	fn from(c: Conviction) -> u8 {
		match c {
			Conviction::None => 0,
			Conviction::Locked1x => 1,
			Conviction::Locked2x => 2,
			Conviction::Locked3x => 3,
			Conviction::Locked4x => 4,
			Conviction::Locked5x => 5,
			Conviction::Locked6x => 6,
		}
	}
}

impl TryFrom<u8> for Conviction {
	type Error = ();
	fn try_from(i: u8) -> Result<Conviction, ()> {
		Ok(match i {
			0 => Conviction::None,
			1 => Conviction::Locked1x,
			2 => Conviction::Locked2x,
			3 => Conviction::Locked3x,
			4 => Conviction::Locked4x,
			5 => Conviction::Locked5x,
			6 => Conviction::Locked6x,
			_ => return Err(()),
		})
	}
}

impl Conviction {
	/// The amount of time (in number of periods) that our conviction implies a successful voter's
	/// balance should be locked for.
	pub fn lock_periods(self) -> u32 {
		match self {
			Conviction::None => 0,
			Conviction::Locked1x => 1,
			Conviction::Locked2x => 2,
			Conviction::Locked3x => 4,
			Conviction::Locked4x => 8,
			Conviction::Locked5x => 16,
			Conviction::Locked6x => 32,
		}
	}

	/// The stake of a voter of the given `balance` with our conviction.
	pub fn stake<B: From<u8> + Zero + Copy + CheckedMul + CheckedDiv + Bounded>(
		self,
		capital: B,
	) -> Follows<B> {
		let stake = match self {
			Conviction::None => capital.checked_div(&10u8.into()).unwrap_or_else(Zero::zero),
			x => capital.checked_mul(&u8::from(x).into()).unwrap_or_else(B::max_value),
		};
		Follows { stake, capital }
	}
}

impl Bounded for Conviction {
	fn min_value() -> Self {
		Conviction::None
	}
	fn max_value() -> Self {
		Conviction::Locked6x
	}
}