#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use pallet_did::{self as did};
use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, 
	dispatch::DispatchResult, ensure, 
	storage::{StorageMap, StorageDoubleMap},
};
use sp_runtime::{MultiSignature};
use sp_runtime::traits::{Hash, IdentifyAccount, Member, Verify};
use sp_std::{prelude::*, vec::Vec};
use system::{self as system, ensure_signed};
use sp_core::{RuntimeDebug};
use sp_core::crypto::KeyTypeId;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"offchain-trade");
pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, KEY_TYPE);
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct AccessCondition<AccountId> {
	pub nonce: u32,
	pub players: Vec<AccountId>,
	pub seqNum: u32,
	pub status: AppStatus,
	pub owner: AccountId,
	pub grantee: AccountId,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
pub enum AppStatus {
	IDLE,
	FINALIZED,
}

type AccessConditionOf<T> = AccessCondition<<T as system::Trait>::AccountId>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct AppState {
	pub nonce: u32,
	pub seqNum: u32,
	pub state: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StateProof<Signature> {
	pub appState: AppState,
	pub sigs: Vec<Signature>,
}


/// The pallet's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait + did::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	//type Public: IdentifyAccount<AccountId = Self::AccountId>;
	//type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
}

decl_storage! {
	trait Store for Module<T: Trait> as DIDOffchainTrade {
		pub AccessConditionAddressKey get(fn key): u32;
		pub AccessConditionAddressList get(fn key_of): 
			map hasher(blake2_256) u32 => Option<T::AccountId>;
		pub AccessConditionList get(condition_list): 
			map hasher(blake2_256) T::AccountId => Option<AccessConditionOf<T>>;
		pub DIDKey get(fn did_key): u32;
		pub DIDList get(fn did_list): 
			map hasher(blake2_256) u32 => Option<T::AccountId>;
		pub KeyOfDID get (fn key_of_did): 
			map hasher(blake2_256) T::AccountId => Option<u32>;
		pub DocumentPermissionsState get(fn permission):
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

			ensure!(players.len() == 2, Error::<T>::InvalidPlayerLength);

			let isPlayer1: bool = <did::Module<T>>::boolean_owner(&did, &players[0]);
			let isPlayer2: bool = <did::Module<T>>::boolean_owner(&did, &players[1]);
			ensure!(isPlayer1 == true || isPlayer2 == true, Error::<T>::NotOwner);

			// Create new Address of Access Condition
			let condition_address = Self::create_condition_address();
		
			// TODO: Refactoring and default <DIDKey> is 2.
			let mut didKey: u32 = |_didKey| {
				let mut _didKey = Self::did_key();
				if _didKey == 0 || _didKey == 1 {
					_didKey = 2;
					_didKey
				} else {
					_didKey
				}
			};
			<DIDList<T>>::insert(didKey, &did);
			<KeyOfDID<T>>::insert(&did, didKey);
			<DIDKey>::mutate(|key| *key += 1);

			if isPlayer1 == true {
				Self::set_access_condition(condition_address, nonce, players[0], players[1], did)?;
			} else {
				Self::set_access_condition(condition_address, nonce, players[1], players[0], did)?;
			}

			Ok(())
		}

		pub fn intendSettle(
			origin, 
			condition_address: T::AccountId,
			transaction: StateProof<T::Signature>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};
			
			let players: Vec<T::AccountId> = access_condition.players;
			ensure!(&who == &players[0] || &who == &players[1], Error::<T>::InvalidSender);
			
			let mut encoded = transaction.appState.nonce.encode();
			encoded.extend(transaction.appState.seqNum.encode());
			encoded.extend(transaction.appState.state.encode());

			Self::valid_signers(transaction.sigs, encoded, players)?;

			ensure!(access_condition.nonce == transaction.appState.nonce, Error::<T>::InvalidNonce);
			ensure!(access_condition.seqNum < transaction.appState.seqNum, Error::<T>::InvalidSeqNum);

			if transaction.appState.state == 0 {
				let new_access_condition = AccessConditionOf::<T> {
					nonce: access_condition.nonce,
					players: access_condition.players.clone(),
					seqNum: transaction.seqNum,
					status: AppStatus::IDLE,
					owner: access_condition.grantee.clone(),
					grantee: access_condition.owner.clone(),
				};
				
				<AccessConditionList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				
				Self::deposit_event(
					RawEvent::SwapPosition(
						condition_address,
						<system::Module<T>>::block_number(),
					)
				);
			} else if transaction.appState.state == 1 {
				let new_access_condition = AccessConditionOf::<T> {
					nonce: access_condition.nonce,
					players: access_condition.players.clone(),
					seqNum: transaction.seqNum,
					status: AppStatus::IDLE,
					owner: access_condition.owner.clone(),
					grantee: access_condition.grantee.clone(),
				};

				<AccessConditionList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				
				Self::deposit_event(
					RawEvent::SetIdle(
						condition_address,
						<system::Module<T>>::block_number(),
					)
				)
			} else {
				let did = match Self::did_list(transaction.appStete.state) {
					Some(_did) => _did,
					None => return Err(Error::<T>::InvalidState.into())
				};

				let new_access_condition = AccessConditionOf::<T> {
					nonce: access_condition.nonce,
					players: access_condition.players.clone(),
					seqNum: transaction.seqNum,
					status: AppStatus::FINALIZED,
					owner: access_condition.owner.clone(),
					grantee: access_condition.grantee.clone(),
				};

				<AccessConditionAddressList<T>>::mutate(&condition_address, |new| *new = Some(new_access_condition.clone()));
				<DocumentPermissionsState<T>>::mutate((&did, &access_condition.grantee), true);
				<FinalizedOf<T>>::mutate(&condition_address, true);
				<OutcomeOf<T>>::mutate(&condition_address, true);
				
				Self::deposit_event(
					RawEvent::IntendSettle(
						condition_address,
						<system::Module<T>>::block_number(),
					)
				);
			}
			Ok(())
		}

		pub fn getStatus(origin, condition_address: T::AccountId) -> DispatchResult {
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
						<system::Module<T>>::block_number(),
				));
			} else {
				Self::deposit_event(
					RawEvent::FinalizedStatus(
						condition_address,
						<system::Module<T>>::block_number(),
					)
				);
			}
			
			Ok(())
		}

		pub fn getSeqNum(origin, condition_address: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			
			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};

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

			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};
			
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
			
			let access_condition = match Self::condition_list(&condition_address) {
				Some(_condtion) => _condtion,
				None => return Err(Error::<T>::InvalidConditionAddress.into())
			};

			let grantee = access_condition.grantee;
			
			Self::deposit_event(
				RawEvent::Grantee(
					grantee,
					<system::Module<T>>::block_number(),
				)
			);
			
			Ok(())
		}

		
		//pub fn isFinalized() {}

		//pub fn getOutcome() {}

		//pub fn checkPermissions() {}

		pub fn setNewDID(origin, did: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			let isOwner: bool = <did::Module<T>>::boolean_owner(&did, &who);
			ensure!(isOwner == true, Error::<T>::NotOwner);

			let mut didKey: u32 = |_didKey| {
				let mut _didKey = Self::did_key();
				if _didKey == 0 || _didKey == 1 {
					_didKey = 2;
					_didKey
				} else {
					_didKey
				}
			};
			<DIDList<T>>::insert(didKey, &did);
			<KeyOfDID<T>>::insert(&did, didKey);
			<DIDKey>::mutate(|key| *key += 1);

			Self::deposit_event(
				RawEvent::NewDID(
					did,
					didKey
				)
			);
			Ok(())
		}

		pub fn getDIDKey(origin, did: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

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

		pub fn getDID(origin, key: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let did = match Self::did_list(key) {
				Some(_did) => _did,
				None => return Err(Error::<T>::NotExist.into())
			};
			
			Self::deposit_event(
				RawEvent::DID(
					did
				)
			);

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T>
	where
	<T as system::Trait>::AccountId,
	<T as system::Trait>::BlockNumber,
	{
		AccessConditionCreated(AccountId, AccountId, AccountId, u32),
		SwapPosition(AccountId, BlockNumber),
		SetIdle(AccountId, BlockNumber),
		IntendSettle(AccountId, BlockNumber),
		IdleStatus(AccountId, BlockNumber),
		FinalizedStatus(AccountId, BlockNumber),
		SeqNum(u32, BlockNumber),
		Owner(AccountId, BlockNumber),
		Grantee(AccountId, BlockNumber),
		BooleanOutcome(bool),
		AccessPermission(bool),
		NewDID(AccountId, u32),
		DID(AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NotOwner,
		InvalidPlayerLength,
		InvalidSender,
		InvalidNonce,
		InvalidSeqNum,
		InvalidSignature,
		InvalidConditionAddress,
		NotExist,
	}
}

impl<T: Trait> Module<T> {
	pub fn valid_signers(
		signatures: Vec<T::Signature>,
		msg: &[u8],
		signers: Vec<T::AccountId>,
	) -> DispatchResult {
		if &signatures[0].verify(msg, &signers[0]) && &signatures[1].verify(msg, &signers[1]) {
			Ok(())
		} else if &signatures[1].verify(msg, &signers[1]) && &signatures[0].verify(msg, &signers[0]) {
			Ok(())
		} else {
			Err(Error::<T>::InvalidSignature.into())
		}
	}

	pub fn account_pair(s: &str) -> sr25519::Pair {
		sr25519::Pair::from_string(&format!("//{}", s), None)
			.expect("static values are valid; qed")
	}
	pub fn account_key(s: &str) -> sr25519::Public {
		sr25519::Pair::from_string(&format!("//{}", s), None)
			.expect("static values are valid; qed")
			.pubic()
	}

	pub fn create_condition_address() -> sr25519::Public {
		let _key: u32 = <AccessConditionAddressKey>::get();
		let key_string: String = _key.to_string();
		let access_condition_string: String = "AccessCondition" + key_string;
		let access_condition_pair: sr25519::Pair = Self::account_pair(access_condition_string);
		let access_condition_public: sr25519::Public = Self::account_key(access_condition_pair);
		access_condition_public
	}

	pub fn set_access_condition(
		condition_address: T::AccountId, 
		nonce: u32,
		owner: T::AccountId,
		grantee: T::AccountId,
		did: T::AccountId,
	) -> DispatchResult {
		let players: Vec<T::AccountId> = vec![owner.clone(), grantee.clone()];
		
		let access_condition = AccessConditionOf::<T> {
			nonce: nonce,
			players: players,
			seqNum: 0,
			status: AppStatus::IDLE,
			owner: owner.clone(),
			grantee: grantee.clone(),
		};
		
		let _key: u32 = <AccessConditionAddressKey>::get();
		<AccessConditionAddressList<T>>::insert(_key, &condition_address);
		<AccessConditionList<T>>::insert(&condition_address, &access_condition);
		<KeyOfDID<T>>::insert(_key, &did);
		<DocumentPermissionsState<T>>::insert(&did, &grantee, false);
		<FinalizedOf<T>>::insert(&condition_address, false);
		<OutcomeOf<T>>::insert(&condition_address, false);

		Ok(())
	}
}