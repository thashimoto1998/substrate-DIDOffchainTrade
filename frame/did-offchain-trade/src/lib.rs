#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use pallet_did::{BooleanOwner};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, 
	dispatch::DispatchResult, ensure, 
	storage::{StorageMap}
};
use sp_runtime::traits::{IdentifyAccount, Member, Verify};
use sp_std::{prelude::*, vec::Vec};
use frame_system::{self as system, ensure_signed};
use sp_core::{RuntimeDebug};

#[cfg(test)]
mod tests;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct AccessCondition<AccountId> {
	pub nonce: u32,
	pub players: Vec<AccountId>,
	pub seq_num: u32,
	pub status: AppStatus,
	pub outcome: bool,
	pub owner: AccountId,
	pub grantee: AccountId,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Encode, Decode, RuntimeDebug)]
pub enum AppStatus {
	IDLE,
	FINALIZED,
}

type AccessConditionOf<T> = AccessCondition<<T as system::Trait>::AccountId>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct AppState {
	pub nonce: u32,
	pub seq_num: u32,
	pub state: Vec<u32>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct StateProof<Signature> {
	pub app_state: AppState,
	pub sigs: Vec<Signature>,
}


/// The pallet's configuration trait.
pub trait Trait: frame_system::Trait  {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Public: IdentifyAccount<AccountId = Self::AccountId>;
	type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
	type BooleanOwner: BooleanOwner<Self::AccountId>;
}

pub trait SingleSessionBooleanOutcome<AccountId> {
	fn is_finalized(condition_address: &AccountId) -> bool;
	fn get_outcome(condition_address: &AccountId) -> bool;
}

pub trait PaymentChannel<AccountId> {
	fn check_permissions(identity: AccountId, grantee: AccountId) -> bool;
}

decl_storage! {
	trait Store for Module<T: Trait> as DIDOffchainTrade {
		pub ConditionKey get(fn condition_key): u32;
		pub AccessConditionAddressList get(fn condition_address): 
			map hasher(blake2_256) u32 => Option<T::AccountId>;
		pub KeyOfCondition get(fn key_of_condition):
			map hasher(blake2_256) T::AccountId => Option<u32>;
		pub AccessConditionList get(fn condition_list): 
			map hasher(blake2_256) T::AccountId => Option<AccessConditionOf<T>>;
		
		pub DIDKey get(fn did_key): u32;
		pub DIDList get(fn did_list): 
			map hasher(blake2_256) u32 => Option<T::AccountId>;
		pub KeyOfDID get(fn key_of_did): 
			map hasher(blake2_256) T::AccountId => Option<u32>;
		
		pub DocumentPermissionsStates get(fn permission):
			map hasher(blake2_256) T::AccountId => Option<T::AccountId>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		pub fn create_access_condition(
			origin,
			players: Vec<T::AccountId>, 
			nonce: u32,
			did: T::AccountId,
			condition_address: T::AccountId
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(players.len() == 2, Error::<T>::InvalidPlayerLength);
			let is_player1: bool = T::BooleanOwner::boolean_owner(&did, &players[0]);
			let is_player2: bool = T::BooleanOwner::boolean_owner(&did, &players[1]);
			ensure!(is_player1 == true || is_player2 == true, Error::<T>::NotOwner);

			let condition_key = Self::condition_key();
			<AccessConditionAddressList<T>>::insert(condition_key, &condition_address);
			<ConditionKey>::mutate(|key| *key += 1);
			<KeyOfCondition<T>>::insert(&condition_address, condition_key);

			// TODO: Refactoring and default <DIDKey> is 2.
			let mut did_key = Self::did_key();
			if did_key == 0 {
				did_key = 2;
				<DIDKey>::mutate(|key| *key = 3);
			} else {
				<DIDKey>::mutate(|key| *key += 1);
			}
			<DIDList<T>>::insert(did_key, &did);
			<KeyOfDID<T>>::insert(&did, did_key);

			if is_player1 == true {
				Self::set_access_condition(condition_address, nonce, players[0].clone(), players[1].clone(), condition_key, did_key)?;
			} else {
				Self::set_access_condition(condition_address, nonce, players[1].clone(), players[0].clone(), condition_key, did_key)?;
			}

			Ok(())
		}

		pub fn intend_settle(
			origin, 
			transaction: StateProof<T::Signature>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(transaction.app_state.state.len() == 2, Error::<T>::InvalidState);
			let condition_address = match Self::condition_address(transaction.app_state.state[0]) {
				Some(_address) => _address,
				None => return Err(Error::<T>::InvalidState.into())
			};
			
			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};
			
			let players: Vec<T::AccountId> = vec![access_condition.players[0].clone(), access_condition.players[1].clone()];
			ensure!(&who == &players[0] || &who == &players[1], Error::<T>::InvalidSender);
			
			let mut encoded = transaction.app_state.nonce.encode();
			encoded.extend(transaction.app_state.seq_num.encode());
			encoded.extend(transaction.app_state.state.encode());

			Self::valid_signers(transaction.sigs, &encoded, players)?;

			ensure!(access_condition.nonce == transaction.app_state.nonce, Error::<T>::InvalidNonce);
			ensure!(access_condition.seq_num < transaction.app_state.seq_num, Error::<T>::InvalidSeqNum);

			if transaction.app_state.state[1] == 0 {
				let players: Vec<T::AccountId> = vec![access_condition.players[0].clone(), access_condition.players[1].clone()];
				let new_access_condition = AccessConditionOf::<T> {
					nonce: access_condition.nonce,
					players: players,
					seq_num: transaction.app_state.seq_num,
					status: AppStatus::IDLE,
					outcome: false,
					owner: access_condition.grantee.clone(),
					grantee: access_condition.owner.clone(),
				};
				
				<AccessConditionList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				Self::deposit_event(
					RawEvent::SwapPosition(
						condition_address,
						<frame_system::Module<T>>::block_number(),
					)
				);
			} else if transaction.app_state.state[1] == 1 {
				let players: Vec<T::AccountId> = vec![access_condition.players[0].clone(), access_condition.players[1].clone()];
				let new_access_condition = AccessConditionOf::<T> {
					nonce: access_condition.nonce,
					players: players,
					seq_num: transaction.app_state.seq_num,
					status: AppStatus::IDLE,
					outcome: false,
					owner: access_condition.owner.clone(),
					grantee: access_condition.grantee.clone(),
				};

				<AccessConditionList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				
				Self::deposit_event(
					RawEvent::SetIdle(
						condition_address,
						<frame_system::Module<T>>::block_number(),
					)
				);
			} else {
				let did = match Self::did_list(transaction.app_state.state[1]) {
					Some(_did) => _did,
					None => return Err(Error::<T>::InvalidState.into())
				};

				let new_access_condition = AccessConditionOf::<T> {
					nonce: access_condition.nonce,
					players: access_condition.players.clone(),
					seq_num: transaction.app_state.seq_num,
					status: AppStatus::FINALIZED,
					outcome: true,
					owner: access_condition.owner.clone(),
					grantee: access_condition.grantee.clone(),
				};

				<AccessConditionList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				<DocumentPermissionsStates<T>>::insert(&did, &access_condition.grantee);
			
				Self::deposit_event(
					RawEvent::IntendSettle(
						condition_address,
						<frame_system::Module<T>>::block_number(),
					)
				);
			}
			Ok(())
		}

		pub fn get_status(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			
			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};
			
			let status = access_condition.status;
			
			if status == AppStatus::IDLE {
				Self::deposit_event(
					RawEvent::IdleStatus(
						condition_address, 
						<frame_system::Module<T>>::block_number(),
				));
			} else {
				Self::deposit_event(
					RawEvent::FinalizedStatus(
						condition_address,
						<frame_system::Module<T>>::block_number(),
					)
				);
			}
			
			Ok(())
		}

		pub fn get_seq_num(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			
			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};

			let seq = access_condition.seq_num;
			
			Self::deposit_event(
				RawEvent::SeqNum(
					seq,
					<frame_system::Module<T>>::block_number(),
				)
			);
			
			Ok(())
		}

		pub fn get_owner(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};
			
			let owner = access_condition.owner;
			
			Self::deposit_event(
				RawEvent::Owner(
					owner,
					<frame_system::Module<T>>::block_number(),
				)
			);
			
			Ok(())
		}

		pub fn get_grantee(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			
			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};

			let grantee = access_condition.grantee;
			
			Self::deposit_event(
				RawEvent::Grantee(
					grantee,
					<frame_system::Module<T>>::block_number(),
				)
			);
			
			Ok(())
		}

		pub fn set_new_did(origin, did: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			let is_owner: bool = T::BooleanOwner::boolean_owner(&did, &who);
			ensure!(is_owner == true, Error::<T>::NotOwner);

			let mut did_key = Self::did_key();
			if did_key == 0 {
				did_key = 2;
				<DIDKey>::mutate(|key| *key = 3);
			} else {
				<DIDKey>::mutate(|key| *key += 1);
			}
			<DIDList<T>>::insert(did_key, &did);
			<KeyOfDID<T>>::insert(&did, did_key);

			Self::deposit_event(
				RawEvent::NewDID(
					did,
					did_key
				)
			);
			Ok(())
		}

		pub fn get_did_key(origin, did: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let key = match Self::key_of_did(&did) {
				Some(_key) => _key,
				None => return Err(Error::<T>::NotExist.into())
			};

			Self::deposit_event(
				RawEvent::DIDKey(
					key
				)
			);

			Ok(())
		}

		pub fn get_did(origin, key: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let _did = match Self::did_list(key) {
				Some(_did) => _did,
				None => return Err(Error::<T>::NotExist.into())
			};
			
			Self::deposit_event(
				RawEvent::DID(
					_did
				)
			);

			Ok(())
		}

		pub fn access_condition_address_key(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let key = match Self::key_of_condition(&condition_address) {
				Some(_key) => _key,
				None => return Err(Error::<T>::NotExist.into())
			};

			Self::deposit_event(
				RawEvent::ConditionAddressKey(
					key
				)
			);

			Ok(())
		}

		pub fn access_condition_address(origin, condition_key: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let condition_address = match Self::condition_address(condition_key) {
				Some(_address) => _address,
				None => return Err(Error::<T>::NotExist.into())
			};

			Self::deposit_event(
				RawEvent::ConditionAddress(
					condition_address
				)
			);

			Ok(())
		}

		pub fn get_access_condition(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			
			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condition) => _condition,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};

			Self::deposit_event(
				RawEvent::AccessCondition(
					access_condition.nonce,
					access_condition.players,
					access_condition.seq_num,
					access_condition.owner,
					access_condition.grantee
				)
			);

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T>
	where
	<T as frame_system::Trait>::AccountId,
	<T as frame_system::Trait>::BlockNumber,
	{
		AccessConditionCreated(AccountId, AccountId, AccountId, u32, u32),
		SwapPosition(AccountId, BlockNumber),
		SetIdle(AccountId, BlockNumber),
		IntendSettle(AccountId, BlockNumber),
		IdleStatus(AccountId, BlockNumber),
		FinalizedStatus(AccountId, BlockNumber),
		SeqNum(u32, BlockNumber),
		Owner(AccountId, BlockNumber),
		Grantee(AccountId, BlockNumber),
		NewDID(AccountId, u32),
		DIDKey(u32),
		DID(AccountId),
		ConditionAddressKey(u32),
		ConditionAddress(AccountId),
		AccessCondition(u32, Vec<AccountId>, u32, AccountId, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NotOwner,
		InvalidPlayerLength,
		InvalidSender,
		InvalidState,
		InvalidNonce,
		InvalidSeqNum,
		InvalidSignature,
		InvalidConditionAddress,
		NotExist,
		ExistAddress,
	}
}

impl<T: Trait> Module<T> {
	pub fn valid_signers(
		signatures: Vec<T::Signature>,
		msg: &[u8],
		signers: Vec<T::AccountId>,
	) -> DispatchResult {
		let signature1 = &signatures[0];
		let signature2 = &signatures[1];
		if signature1.verify(msg, &signers[0]) && signature2.verify(msg, &signers[1]) {
			Ok(())
		} else if signature1.verify(msg, &signers[1]) && signature2.verify(msg, &signers[0]) {
			Ok(())
		} else {
			Err(Error::<T>::InvalidSignature.into())
		}
	}

	fn set_access_condition(
		condition_address: T::AccountId, 
		nonce: u32,
		owner: T::AccountId,
		grantee: T::AccountId,
		condition_key: u32,
		did_key: u32,
	) -> DispatchResult {
		let players: Vec<T::AccountId> = vec![owner.clone(), grantee.clone()];
		
		let access_condition = AccessConditionOf::<T> {
			nonce: nonce,
			players: players,
			seq_num: 0,
			status: AppStatus::IDLE,
			outcome: false,
			owner: owner.clone(),
			grantee: grantee.clone(),
		};
		
		<AccessConditionList<T>>::insert(&condition_address, &access_condition);

		Self::deposit_event(
			RawEvent::AccessConditionCreated(
				condition_address,
				owner,
				grantee,
				condition_key,
				did_key,
			)
		);
		Ok(())
	}

	// To use test
	pub fn test_get_owner(
		condition_address: T::AccountId
	) -> T::AccountId {
		let access_condition = match Self::condition_list(&condition_address) {
			Some(_address) => _address,
			None => return condition_address
		};

		return access_condition.owner;
	}
}

impl<T: Trait> SingleSessionBooleanOutcome<T::AccountId> for Module<T> {
    fn is_finalized(condition_address: &T::AccountId) -> bool {
		let access_condition = match Self::condition_list(condition_address) {
			Some(_condition) => _condition,
			None => return false
		};

		let status = access_condition.status;

		if status == AppStatus::FINALIZED {
			return true;
		} else {
			return false;
		}
	}

	fn get_outcome(condition_address: &T::AccountId) -> bool {
		let access_condition = match Self::condition_list(condition_address) {
			Some(_condition) => _condition,
			None => return false
		};

		let outcome = access_condition.outcome;

		if outcome == true {
			return true;
		} else {
			return false;
		}
	}
}

impl<T: Trait> PaymentChannel<T::AccountId> for Module<T> {
	fn check_permissions(identity: T::AccountId, grantee: T::AccountId) -> bool {
		let _grantee = match Self::permission(identity) {
			Some(_grantee) => _grantee,
			None => return false
		};

		if grantee == _grantee {
			return true;
		} else {
			return false;
		}
	}
}