#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, 
	dispatch::DispatchResult, ensure, StorageMap,
};
use sp_runtime::traits::{Hash, IdentifyAccount, Member, Verify};
use sp_std::{prelude::*m vec::Vec};
use system::ensure_signed;
use sp_core::RuntimeDebug; 

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct AccessCondition<AccountId> {
	pub nonce: u32,
	pub players: Vec<AccountId>,
	pub seqNum: u32,
	pub status: AppStatus,
	pub owner: AccountId,
	pub grantee: AccountId,
	pub did: AccountId,
	pub didList: Vec<AccountId>,
	pub documentPermissionsState: map (AccountId, AccountId) => bool;
	pub const key: u32,
}

pub enum AppStatus {
	IDLE,
	FINALIZED,
}

type AcessConditionOf<T> = AccessCondition<<T as system::Trait>::AccountId>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct AppState<AccountId> {
	pub nonce: u32,
	pub seqNum: u32,
	pub state: AccountId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StateProof<AccountId, Signature> {
	pub appState: AppState<AccountId>,
	pub sigs: Vec<Signature>,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Public: IdentifyAccount<AccountId = Self::AccountId>;
	type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
}


decl_storage! {
	trait Store for Module<T: Trait> as DIDOffchainTrade {
		pub AccessConditionOf get(appinfo_of): 
			map hasher(blake2_256) T::AccountId => AccessConditionOf<T>;
		pub keyOf get(key_of): map u32 => T::AccountId;
		pub FinalizedOf: map hasher(blake2_256) T::AccountId => bool;
		pub OutcomeOf: map hasher(blake2_256) T::AccountId => bool;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		pub fn createAccessCondition() {}

		pub fn intendSettle() {}

		pub fn getStatus() {}

		pub fn getSeqNum() {}

		pub fn isFinalized() {}

		pub fn getOutcome() {}

		pub fn checkPermissions() {}

		pub fn setNewDID() {}
	}
}

decl_event!(
	pub enum Event<T>
	where
	<T as system::Trait>::AccountId,
	<T as system::Trait>::BlockNumber,
	{
		AccessConditionCreated(AccountId, AccountId, AccountId, u32),
		IntendSettle(u32, BlockNumber),
		IDLE_STATUS(AccountId, BlockNumber),
		FINALIZED_STATUS(AccountId, BlockNumber),
		BooleanOutcome(bool),
		AccessPermission(bool),
		NewDID(AccountId),
	}
)

decl_error! {
	pub enum Error for Module<T: Trait> {
		NotOwner,
		InvalidSeqNum,
		InvalidSignature,
	}
}

impl<T: Trait> Module<T> {
	pub fn verifySignature() {}
}