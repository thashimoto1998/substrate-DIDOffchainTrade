//! # DID Offchain Trade Pallet
//!
//! The DID Offchain Trade pallet allows trading data access control rights at offchain.
//!
//! ## Overview
//! 
//! The DID Offchain Trade pallet provides functionality for data access control rights trading.
//!
//! * Create Access Condition
//! * Update on-chain condition by co-singed state proof.
//! * Set New DID
//!
//! ### Terminology
//!
//! * **DID** A Decentralized Identifiers/Identity compliant with the DID standard.
//!		The DID is an AccountId with associated attributes/properties.
//! * **Access Condition** Access Condition allows managing and resolving payment logic.
//! * **DocumentPermissionsState** DocumentPermissionsState manage who has data access control rights.
//! 
//! ### Goals
//! The DID Offchain Trade system in designed to make the following possible:
//!
//! * Users control their data. 
//! * Manage data access control rights transparently. 
//! * Trading data access control rights without trusted third party.
//!
//! ### Dispatchable Functions
//!
//! * `create_access_condition` - Create a new Access Condition from channel peer.
//! * `intend_settle` - Update Access Condition and DocumentPermissionsState by co-signed state proof from channel peer.
//! * `set_new_did` - Set a new did to DID List.
//! * `get_access_condition` - Get field of Access Condition.
//!
//!	### Dispatchable Functions
//!
//! * `is_finalized` - Returns a boolen value. `True` if the AppStatus is FINALIZED. AppStatus is field of AccessCondition.
//! * `get_outcome` - Returns a boolean value. `True` if the outcome which is field of AccessCondition is true. 
//! * `check_permissions` - Returns a boolean value. `True` if the grantee has data access control rights.
//! * `get_nonce` - Get the nonce which is field of AccessCondition.
//! * `get_seq_num` - Get the sequence number which is field of AccessCondition.
//! * `get_status` - Get the AppStatus which is field of AccessCondition. AppStatus is IDLE or FINALIZED.
//! * `get_owner` - Get the owner which is field of AccessCondition.
//! * `get_grantee` - Get the grantee which is field of AccessCondition.
//! * `get_did_key` - Get the key of did.
//! * `access_condition_address_key` - Get the key of AccessCondition.
//!
//! ## Dependencies
//!
//! This pallet depends on the DID pallet and secret store module.
//!
//! *

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use pallet_did;
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, 
	dispatch::DispatchResult, ensure, 
	storage::{StorageMap, StorageDoubleMap},
};
use sp_runtime::traits::{IdentifyAccount, Member, Verify};
use sp_std::{prelude::*, vec::Vec};
use frame_system::{self as system, ensure_signed};
use sp_core::{RuntimeDebug};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod mock;

/// Access Condition 
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct AccessCondition<AccountId> {
	pub nonce: i32,
	pub players: Vec<AccountId>,
	pub seq_num: i32,
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
	pub nonce: i32,
	pub seq_num: i32,
	pub state: Vec<i32>,
}

/// Co-signed state proof
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct StateProof<Signature> {
	pub app_state: AppState,
	pub sigs: Vec<Signature>,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_did::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Public: IdentifyAccount<AccountId = Self::AccountId>;
	type Signature: Verify<Signer = <Self as Trait>::Public> + Member + Decode + Encode;
}

decl_storage! {
	trait Store for Module<T: Trait> as DIDDIDTrade {
		/// Key of Access Condition used to identify address of Access Condition.
		pub ConditionKey get(fn condition_key): i32;
		/// Mapping key of Access Condition to address of Access Condition.
		pub AccessConditionAddressList get(fn condition_address): 
			map hasher(twox_64_concat) i32 => Option<T::AccountId>;
		/// Mapping address of Access Condition to key of Access Condition.
		pub KeyOfConditionAddress get(fn key_of_condition):
			map hasher(twox_64_concat) T::AccountId => Option<i32>;
		/// The set of address of Access Condition and Access Condition. 
		pub AccessConditionList get(fn condition_list): 
			map hasher(twox_64_concat) T::AccountId => Option<AccessConditionOf<T>>;
		
		/// Key of DID use dto identify DID.
		pub DIDKey get(fn did_key): i32;
		/// Mapping key of DID to DID.
		pub DIDList get(fn did_list): 
			map hasher(twox_64_concat) i32 => Option<T::AccountId>;
		/// Mapping DID to key of DID.
		pub KeyOfDID get(fn key_of_did): 
			map hasher(twox_64_concat) T::AccountId => Option<i32>;
		
		/// First account is DID and second account is grantee.
		/// If grantee has data access control right, DocumentPermissionsStates is 1.
		pub DocumentPermissionsStates get(fn permission):
			double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) T::AccountId => u8;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Create Access Condition.
		pub fn create_access_condition(
			origin,
			players: Vec<T::AccountId>, 
			nonce: i32,
			did: T::AccountId,
			condition_address: T::AccountId
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			/// Checks if number of channel peer is 2.
			ensure!(players.len() == 2, Error::<T>::InvalidPlayerLength);
			
			let owner = match <pallet_did::Module<T>>::owner_of(&did) {
				Some(_owner) => _owner,
				None => return Err(Error::<T>::NotExist.into())
			};
			/// Checks if channel peer is owner of did.
			ensure!(owner == players[0] || owner == players[1], Error::<T>::NotOwner);

			/// Add address of Access Condition and update key of Access Condition.
			ensure!(KeyOfConditionAddress::<T>::contains_key(&condition_address) == false, Error::<T>::ExistAddress);
			let condition_key = Self::set_condition_address(&condition_address);

			/// Add DID and update key of DID.
			let did_key = match Self::set_did(&did) {
				Some(_key) => _key,
				None => return Err(Error::<T>::NotExist.into())
			};

			if owner == players[0] {
				/// Add Access Condition.
				Self::set_access_condition(condition_address, nonce, players[0].clone(), players[1].clone(), condition_key, did_key)?;
			} else {
				/// Add Access Condition.
				Self::set_access_condition(condition_address, nonce, players[1].clone(), players[0].clone(), condition_key, did_key)?;
			}

			Ok(())
		}

		/// Update Access Condition and DocumentPermissionsState by co-signed state proof from channel peer.
		pub fn intend_settle(
			origin, 
			transaction: StateProof<<T as Trait>::Signature>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(transaction.app_state.state.len() == 2, Error::<T>::InvalidStateLength);
			
			/// Get address of Access Condition. state[0] is key of condition address.
			let condition_address = match Self::condition_address(transaction.app_state.state[0]) {
				Some(_address) => _address,
				None => return Err(Error::<T>::InvalidState.into())
			};
			
			/// Get Access Condition.
			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};
			
			let players: Vec<T::AccountId> = vec![access_condition.players[0].clone(), access_condition.players[1].clone()];
			ensure!(&who == &players[0] || &who == &players[1], Error::<T>::InvalidSender);
			
			let mut encoded = transaction.app_state.nonce.encode();
			encoded.extend(transaction.app_state.seq_num.encode());
			encoded.extend(transaction.app_state.state.encode());
			/// Checks if a state proof is signed by channel peer.
			Self::valid_signers(transaction.sigs, &encoded, players)?;

			/// Checks if a nonce is valid.
			ensure!(access_condition.nonce == transaction.app_state.nonce, Error::<T>::InvalidNonce);
			/// Checks if a sequence number is higher than previous one.
			ensure!(access_condition.seq_num < transaction.app_state.seq_num, Error::<T>::InvalidSeqNum);

			if transaction.app_state.state[1] == -1 {
			/// If state[1] is -1, AppStatus update from FINALED to IDLE and replace owner and grantee.
				
				/// Checks if AppStatus is FINALIZED.
				ensure!(access_condition.status == AppStatus::FINALIZED, Error::<T>::NotFinalizedStatus);
				
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
				
				/// Update Access Condition.
				<AccessConditionList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				Self::deposit_event(
					RawEvent::SwapPosition(
						condition_address,
						<frame_system::Module<T>>::block_number(),
					)
				);
			} else if transaction.app_state.state[1] == -2 {
			/// If state[1] is -2, AppStatus update from FINALIZED to IDLE.
				
				/// Checks if AppStatus is IDLE.
				ensure!(access_condition.status == AppStatus::FINALIZED, Error::<T>::NotFinalizedStatus);
				
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

				/// Update Access Condition.
				<AccessConditionList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				
				Self::deposit_event(
					RawEvent::SetIdle(
						condition_address,
						<frame_system::Module<T>>::block_number(),
					)
				);
			} else {
			/// If state[1] is key of DID, grantee is granted data access control rights, 
			/// AppStatus update from IDLE to FINALIZED and outcome update true.
			
				let did = match Self::did_list(transaction.app_state.state[1]) {
					Some(_did) => _did,
					None => return Err(Error::<T>::InvalidDIDState.into())
				};

				/// Checks if AppStatus is IDLE.
				ensure!(access_condition.status == AppStatus::IDLE, Error::<T>::NotIdleStatus);

				let new_access_condition = AccessConditionOf::<T> {
					nonce: access_condition.nonce,
					players: access_condition.players.clone(),
					seq_num: transaction.app_state.seq_num,
					status: AppStatus::FINALIZED,
					outcome: true,
					owner: access_condition.owner.clone(),
					grantee: access_condition.grantee.clone(),
				};

				/// Update Access condition.
				<AccessConditionList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				/// Add DocumentPermissionState.
				<DocumentPermissionsStates<T>>::insert(&did, &access_condition.grantee, 1);
			
				Self::deposit_event(
					RawEvent::IntendSettle(
						condition_address,
						<frame_system::Module<T>>::block_number(),
					)
				);
			}
			
			Ok(())
		}

		/// Set new DID.
		pub fn set_new_did(origin, did: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			let owner = match <pallet_did::Module<T>>::owner_of(&did) {
				Some(_owner) => _owner,
				None => return Err(Error::<T>::NotExist.into())
			};
			/// Checks if caller is owner of did.
			ensure!(who == owner, Error::<T>::NotOwner);

			/// Add DID and update did key.
			let did_key = match Self::set_did(&did) {
				Some(_key) => _key,
				None => return Err(Error::<T>::NotExist.into())
			};

			Self::deposit_event(
				RawEvent::NewDID(
					did,
					did_key
				)
			);
			
			Ok(())
		}		
		
		/// Get Access Condition.
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
		AccessConditionCreated(AccountId, AccountId, AccountId, i32, i32),
		SwapPosition(AccountId, BlockNumber),
		SetIdle(AccountId, BlockNumber),
		IntendSettle(AccountId, BlockNumber),
		NewDID(AccountId, i32),
		DIDKey(i32),
		AccessCondition(i32, Vec<AccountId>, i32, AccountId, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NotOwner,
		InvalidPlayerLength,
		InvalidSender,
		InvalidState,
		InvalidStateLength,
		InvalidDIDState,
		InvalidNonce,
		InvalidSeqNum,
		InvalidSignature,
		InvalidConditionAddress,
		NotExist,
		ExistAddress,
		NotIdleStatus,
		NotFinalizedStatus,
	}
}

impl<T: Trait> Module<T> {
	/// Checks if signature is valid.
	pub fn valid_signers(
		signatures: Vec<<T as Trait>::Signature>,
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

	/// Set Access Condition.
	fn set_access_condition(
		condition_address: T::AccountId, 
		nonce: i32,
		owner: T::AccountId,
		grantee: T::AccountId,
		condition_key: i32,
		did_key: i32,
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

	/// Set DID.
	fn set_did(did: &T::AccountId) -> Option<i32> {
		let did_key = Self::did_key();
		<DIDList<T>>::insert(did_key, did);
		<KeyOfDID<T>>::insert(did, did_key);
		<DIDKey>::mutate(|key| *key += 1);

		return Some(did_key);
	}

	/// Set Condition Address.
	fn set_condition_address(condition_address: &T::AccountId) ->i32 {
		let condition_key = Self::condition_key();
		<AccessConditionAddressList<T>>::insert(condition_key, &condition_address);
		<ConditionKey>::mutate(|key| *key += 1);
		<KeyOfConditionAddress<T>>::insert(&condition_address, condition_key);
		
		return condition_key;
	}

	/// Check if AppStatus is FINALIZED.
	pub fn is_finalized(condition_address: &T::AccountId) -> bool {
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

	/// Check if outcome is true.
	pub fn get_outcome(condition_address: &T::AccountId) -> bool {
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

	/// Check if grantee has data access control rights.
	pub fn check_permissions(identity: T::AccountId, grantee: T::AccountId) -> bool {
		if Self::permission(&identity, &grantee) == 1 {
			return true;
		} else {
			return false;
		}
	}

	/// Get nonce of Access Condition.
	pub fn get_nonce(condition_address: T::AccountId) -> i32 {
		let access_condition = match Self::condition_list(&condition_address) {
			Some(_condition) => _condition,
			None => return -1
		};

		return access_condition.nonce;
	}

	/// Get sequence number of Access Condition.
	pub fn get_seq_num(condition_address: T::AccountId) -> i32 {
		let access_condition = match Self::condition_list(&condition_address) {
			Some(_condition) => _condition,
			None => return -1
		};

		return access_condition.seq_num;
	}

	/// Get AppStatus of Access Condition.
	/// If possible, this function return AppStatus
	pub fn get_status(condition_address: T::AccountId) -> i32 {
		let access_condition = match Self::condition_list(&condition_address) {
			Some(_condition) => _condition,
			None => return -1
		};
		
		if access_condition.status == AppStatus::IDLE {
			return 0;
		} else {
			return 1;
		}
	}

	/// Get Owner of Access Condition.
	pub fn get_owner(condition_address: T::AccountId) -> T::AccountId {
		let access_condition = match Self::condition_list(&condition_address) {
			Some(_condition) => _condition,
			None => return condition_address
		};

		return access_condition.owner;
	}

	/// Get Grantee of Access Condition.
	pub fn get_grantee(condition_address: T::AccountId) -> T::AccountId {
		let access_condition = match Self::condition_list(&condition_address) {
			Some(_condition) => _condition,
			None => return condition_address
		};

		return access_condition.grantee;
	}

	/// Get key of did.
	pub fn get_did_key(did: T::AccountId) -> i32 {
		let key = match Self::key_of_did(&did) {
			Some(_key) => _key,
			None => return -1
		};

		return key;
	}

	/// Get key of access condition address.
	pub fn access_condition_address_key(condition_address: T::AccountId) -> i32 {
		let key = match Self::key_of_condition(&condition_address) {
			Some(_key) => _key,
			None => return -1
		};

		return key;
	}
}
