#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use did::{DIDOwner};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, 
	dispatch::DispatchResult, ensure, 
	storage::{StorageMap, StorageDoubleMap},
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
	pub key: u32,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
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
	type DIDOwner: DIDOwner<AccountId = Self::AccountId>;
}


decl_storage! {
	trait Store for Module<T: Trait> as DIDOffchainTrade {
		pub Key get(key): u32;
		pub AccessConditionList get(condition_list): 
			map hasher(blake2_256) T::AccountId => AccessConditionOf<T>;
		pub KeyOf get(key_of): map u32 => T::AccountId;
		pub DocumentPermissionsState get(permission):
			double_map hasher(blake2_256) T::AccountId, hasher(blake2_256) T::AccountId => bool;
		pub FinalizedOf: map hasher(blake2_256) T::AccountId => bool;
		pub OutcomeOf: map hasher(blake2_256) T::AccountId => bool;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		pub fn createAccessCondition(
			origin,
			players: Vec<T::AccountId>, 
			nonce: u32,
			did: T::AccountId,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			// Create new Address of Access Condition
			let key: u32 = <Key>::get();
			let key_string: String = key.to_string();
			let access_condition_string: String = "AccessCondition" + key_string;
			let access_condition_pair: sr25519::Pair = Self::account_pair(access_condition_string);
			let access_condition_public: sr25519::Public = Self::account_key(access_condition_pair);

			let isPlayer1: bool = T::DIDOwner::is_did_owner(&did, &players[0]);
			let isPlayer2: bool = T::DIDOwner::is_did_owner(&did, &players[1]);
			ensure!(isPlayer1 == true || isPlayer2 == true, Error::<T>::NotOwner);
			
			if (isPlayer1 == true) {
				let _didList: Vec<T::AccountId> = vec![did];
				let access_condition = AccessConditionOf<T> {
					nonce: nonce,
					players: players.clone(),
					seqNum: 0,
					status: AppStatus::IDLE,
					owner: players[0].clone(),
					grantee: players[1].clone(),
					did: did,
					didList: _didList,
					key: key,
				};
				<AccessConditionList<T>>::insert(access_condition_public.clone(), access_condition);
				<KeyOf<T>>::insert(key, access_condition_public.clone());
				<DocumentPermissionsState<T>>::insert(&did, players[1].clone(), false);
				<FinalizedOf<T>>::insert(access_condition_public.clone(), false);
				<OutcomeOf<T>>::insert(access_condition_public.clone(), false);
				let _key = key + 1;
				<Key>::put(_key);
				Self::deposit_event(RawEvent::AccessConditionCreated(
					access_condition_public,
					players[0],
					players[1],
					key,
				));
				Ok(())
			} else {
				let _didList: Vec<T::AccountId> = vec![did];
				let access_condition = AccessConditionOf<T> {
					nonce: nonce,
					players: players.clone(),
					seqNum: 0,
					status: AppStatus::IDLE,
					owner: players[1],
					grantee: players[0],
					did: did,
					didList: _didList,
					key: key,
				};
				<AccessConditionList<T>>::insert(access_condition_public.clone(), access_condition);
				<KeyOf<T>>::insert(key, access_condition_public.clone());
				<DocumentPermissionsState<T>>::insert(&did, players[0].clone(), false);
				<FinalizedOf<T>>::insert(access_condition_public.clone(), false);
				<OutcomeOf<T>>::insert(access_condition_public.clone(), false);
				let _key = key + 1;
				<Key>::put(_key);
				Self::deposit_event(RawEvent::AccessConditionCreated(
					access_condition_public,
					players[1],
					players[0],
					key
				));
			}

		}

		pub fn intendSettle() {}

		pub fn getStatus(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let access_condition = Self::condition_list(&condition_address);
			let status = access_condition.status;
			
			if (status == AppStatus::IDLE) {
				Self::deposit_event(
					RawEvent::IDLE_STATUS(
						condition_address, 
						<system::Module<T>>::block_number(),
				));
			} else {
				Self::deposit_event(
					RawEvent::FINALIZED_STATUS(
						condition_address,
						<system::Module<T>>::block_number(),
					)
				);
			}
			Ok(())
		}

		pub fn getSeqNum(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let access_condition = Self::condition_list(&condition_address);
			let seq = access_condition.seqNum;
			Self::deposit_event(
				RawEvent::SeqNum(
					seq,
					<system::Module<T>>::block_number(),
				)
			);
			Ok(())
		}

		pub fn getOwner(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let access_condition = Self::condition_list(&condition_address);
			let owner = access_condition.owner;
			Self::deposit_event(
				RawEvent::Owner(
					owner,
					<system::Module<T>>::block_number(),
				)
			);
			Ok(())
		}

		pub fn getGrantee(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let access_condition = Self::condition_list(&condition_address);
			let grantee = access_condition.grantee;
			Self::deposit_event(
				RawEvent::Grantee(
					grantee,
					<system::Module<T>>::block_number(),
				)
			);
			Ok(())
		}

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
		SeqNum(u32, BlockNumber),
		Owner(AccountId, BlockNumber),
		Grantee(AccountId, BlockNumber),
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

	pub fn account_pair(s: &str) -> sr25519::Pair {
		sr25519::Pair::from_string(&format!("//{}", s), None)
			.expect("static values are valid; qed")
	}
	pub fn account_key(s: &str) -> sr25519::Public {
		sr25519::Pair::from_string(&format!("//{}", s), None)
			.expect("static values are valid; qed")
			.pubic()
	}
}