#![cfg_attr(not(feature = "std"), no_std)]

//! # Direct Delegation
//! 
//! A pallet that aims to deliver a minimal viable working product of a system that lets people stake their funds to:
//! - Become a collator;
//! - Delegate to a collator;
//! - Join a delegator pool
//! 
//! //! ## Overview
//!
//! This pallet provides functions to:
//! 
//! - Join a collator set as a candidate;
//! - Leave (requesting to leave) a collator set, removing oneself as a candidate;
//! - Delegating directly to a collator candidate.
//! - Withdraw unstaked after a wait period
//! - Join a delegator pool (ToDo)
//! 
//!
//! ### Terminology
//!
//! - **Candidate:** A user which locks up tokens to be included into the set of
//!   authorities which author blocks and receive rewards for doing so.
//!
//! - **Collator:** A candidate that was chosen to collate this round.
//!
//! - **Delegator:** A user which locks up tokens for collators they trust. When
//!   their collator authors a block, the corresponding delegators also receive
//!   rewards.
//!
//! - **Total Stake:** A collator’s own stake + the sum of delegated stake to
//!   this collator.
//!
//! - **Total collator stake:** The sum of tokens locked for staking from all
//!   collator candidates.
//!
//! - **Total delegator stake:** The sum of tokens locked for staking from all
//!   delegators.
//!
//! - **To Stake:** Lock tokens for staking.
//!
//! - **To Unstake:** Unlock tokens from staking.
//!
//! - **Round (= Session):** A fixed number of blocks in which the set of
//!   collators does not change. We set the length of a session to the length of
//!   a staking round, thus both words are interchangeable in the context of
//!   this pallet.
//!
//! - **Lock:** A freeze on a specified amount of an account's free balance
//!   until a specified block number. Multiple locks always operate over the
//!   same funds, so they "overlay" rather than "stack"
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//! 
//! - `set_inflation` - Change the inflation configuration. Requires sudo.
//! 
//! - `set_max_selected_candidates` - Change the number of collator candidates
//!   which can be selected to be in the set of block authors. Requires sudo.
//! 
//! - `set_blocks_per_round` - Change the number of blocks of a round. Shorter
//!   rounds enable more frequent changes of the selected candidates, earlier
//!   unlockal from unstaking and earlier collator leaving. Requires sudo.
//! 
//! - `join_candidates` - Join the set of collator candidates by staking at
//!   least `MinCandidateStake` and at most `MaxCollatorCandidateStake`.
//! 
//! - `init_leave_candidates` - Request to leave the set of collators. Unstaking
//!   and storage clean-up is delayed until executing the exit at least
//!   ExitQueueDelay rounds later.
//! 
//! - `candidate_stake_more` - Increase your own stake as a collator candidate
//!   by the provided amount up to `MaxCollatorCandidateStake`.
//! 
//! - `candidate_stake_less` - Decrease your own stake as a collator candidate
//!   by the provided amount down to `MinCandidateStake`.
//! 
//! - `join_delegators` - Join the set of delegators by delegating to a collator
//!   candidate.
//! 
//! - `delegate_another_candidate` - Delegate to another collator candidate by
//!   staking for them.
//! 
//! - `leave_delegators` - Leave the set of delegators and revoke all
//!   delegations. Since delegators do not have to run a node and cannot be
//!   selected to become block authors, this exit is not delayed like it is for
//!   collator candidates.
//! 
//! - `revoke_delegation` - Revoke a single delegation to a collator candidate.
//! - `delegator_stake_more` - Increase your own stake as a delegator and the
//!   delegated collator candidate's total stake.
//! 
//! - `delegator_stake_less` - Decrease your own stake as a delegator and the
//!   delegated collator candidate's total stake by the provided amount down to
//!   `MinDelegatorStake`.
//! 
//! - `unlock_unstaked` - Attempt to unlock previously unstaked balance from any
//!   account. Succeeds if at least one unstake call happened at least
//!   `StakeDuration` blocks ago.
//!
//! ## Genesis config
//!
//! Thisg pallet depends on the [`GenesisConfig`].
//!
//! ## Assumptions+
//!
//! - At the start of session s(i), the set of session ids for session s(i+1)
//!   are chosen. These equal the set of selected candidates. Thus, we cannot
//!   allow collators to leave at least until the start of session s(i+2).
//!
//!


/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(().into())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(().into())
				},
			}
		}
	}
}
