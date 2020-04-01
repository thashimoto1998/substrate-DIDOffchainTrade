use super::*;
use sp_runtime::{
	testing::{Header},
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use sp_std::marker::PhantomData;
use frame_support::{
	assert_err, assert_ok, assert_noop, impl_outer_origin, 
	parameter_types, StorageMap, weights::Weight,
};
use frame_system::{self, EventRecord, Phase};
use sp_core::{sr25519, Pair, H256};
use pallet_balances;


impl_outer_origin! {
	pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl system::Trait for Test {
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
}



/// define mock did trait
pub trait MockDIDTrait: system::Trait  {}
decl_storage! {
	trait Store for MockDIDModule<T: MockDIDTrait > as MockDID {
		pub OwnerOf get(owner_of): map hasher(blake2_256) <T as frame_system::Trait>::AccountId => Option<<T as frame_system::Trait>::AccountId>;
	}
}
pub struct MockDIDModule<T: MockDIDTrait>(PhantomData<T>);
impl<T: MockDIDTrait> BooleanOwner<<T as frame_system::Trait>::AccountId> for MockDIDModule<T> {
	fn boolean_owner(identity: &<T as frame_system::Trait>::AccountId, actual_owner: &<T as frame_system::Trait>::AccountId) -> bool {
		return true;
	}
}

impl MockDIDTrait for Test {}

impl Trait for Test {
	type Event = ();
	type Public = sr25519::Public;
	type Signature = sr25519::Signature;
	type BooleanOwner = MockDIDModule<Test>;
}

type System = frame_system::Module<Test>;
type OffchainTrade = Module<Test>;

fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

pub fn account_pair(s: &str) -> sr25519::Pair {
	sr25519::Pair::from_string(&format!("//{}", s), None).expect("static values are valid: qed")
}

pub fn account_key(s: &str) -> sr25519::Public {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}

#[test]
fn validate_signature() {
	new_test_ext().execute_with(|| {
		let nonce: u32 = 1;
		let seq_num: u32 = 0;
		let state = [0, 0].to_vec();

		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let signers_vec = [alice_public.clone(), bob_public.clone()].to_vec();

		let mut encoded = nonce.encode();
		encoded.extend(seq_num.encode());
		encoded.extend(state.encode());

		let alice_sig = alice_pair.sign(&encoded);
		let bob_sig = bob_pair.sign(&encoded);
		let sig_vec = [alice_sig.clone(), bob_sig.clone()].to_vec();

		assert_ok!(OffchainTrade::valid_signers(
			sig_vec,
			&encoded,
			signers_vec
		));

		let risa_pair = account_pair("Risa");
		let risa_public = risa_pair.public();
		let invalid_signers_vec = [alice_public.clone(), risa_public.clone()].to_vec();
		let invalid_sig_vec = [alice_sig.clone(), bob_sig.clone()].to_vec();

		assert_noop!(
			OffchainTrade::valid_signers(
				invalid_sig_vec,
				&encoded,
				invalid_signers_vec
			),
			Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn test_create_access_condition() {
	new_test_ext().execute_with(|| {
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let players_vec = [alice_public.clone(), bob_public.clone()].to_vec();

		let condition_account = account_key("Condition");

		let did_account = account_key("DID");

		let nonce = 2;

		assert_ok!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				nonce,
				did_account.clone(),
				condition_account.clone()
			)
		);
		assert_eq!(OffchainTrade::condition_key(), 1);
		assert_eq!(
			OffchainTrade::key_of_condition(condition_account.clone()), Some(0)
		);
		assert_eq!(
			OffchainTrade::condition_address(0), Some(condition_account.clone())
		);
		assert_eq!(OffchainTrade::did_key(), 3);
		assert_eq!(OffchainTrade::did_list(2), Some(did_account.clone()));
		assert_eq!(OffchainTrade::key_of_did(did_account.clone()), Some(2));
		assert_eq!(OffchainTrade::is_finalized(&condition_account), false);

		let risa_pair = account_pair("Risa");
		let risa_public = risa_pair.public();
		let invalid_players_vec = [alice_public.clone(), bob_public.clone(), risa_public.clone()].to_vec();
		assert_noop!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				invalid_players_vec,
				nonce,
				did_account.clone(),
				condition_account.clone()
			),
			Error::<Test>::InvalidPlayerLength
		);
	});
}

#[test]
fn test_intend_settle() {
	new_test_ext().execute_with(|| {
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let players_vec = [alice_public.clone(), bob_public.clone()].to_vec();

		let condition_account = account_key("Condition");

		let did_account = account_key("DID");

		assert_ok!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				2,
				did_account.clone(),
				condition_account.clone()
			)
		);

		let app_state1 = AppState {
			nonce: 2,
			seq_num: 1,
			state: [0, 2].to_vec(),
		};

		let mut encoded1 = app_state1.nonce.encode();
		encoded1.extend(app_state1.seq_num.encode());
		encoded1.extend(app_state1.state.encode());

		let alice_sig1 = alice_pair.sign(&encoded1);
		let bob_sig1 = bob_pair.sign(&encoded1);
		let sigs_vec1 = [alice_sig1.clone(), bob_sig1.clone()].to_vec();

		let state_proof1 = StateProof {
			app_state: app_state1,
			sigs: sigs_vec1,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof1
			)
		);
		assert_eq!(
			OffchainTrade::permission(did_account.clone()), Some(bob_public.clone())
		);
		assert_eq!(OffchainTrade::is_finalized(&condition_account), true);
		assert_eq!(OffchainTrade::get_outcome(&condition_account), true);
		assert_eq!(OffchainTrade::check_permissions(
			did_account.clone(), bob_public.clone()), 
			true
		);

		let app_state2 = AppState {
			nonce: 2,
			seq_num: 2,
			state: [0, 1].to_vec()
		};

		let mut encoded2 = app_state2.nonce.encode();
		encoded2.extend(app_state2.seq_num.encode());
		encoded2.extend(app_state2.state.encode());

		let alice_sig2 = alice_pair.sign(&encoded2);
		let bob_sig2 = bob_pair.sign(&encoded2);
		let sigs_vec2 = [alice_sig2.clone(), bob_sig2.clone()].to_vec();

		let state_proof2 = StateProof {
			app_state: app_state2,
			sigs: sigs_vec2,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof2
			)
		);
		assert_eq!(OffchainTrade::is_finalized(&condition_account), false);
		assert_eq!(OffchainTrade::get_outcome(&condition_account), false);
	

		let app_state3 = AppState {
			nonce: 2,
			seq_num: 3,
			state: [0, 0].to_vec()
		};

		let mut encoded3 = app_state3.nonce.encode();
		encoded3.extend(app_state3.seq_num.encode());
		encoded3.extend(app_state3.state.encode());

		let alice_sig3 = alice_pair.sign(&encoded3);
		let bob_sig3 = bob_pair.sign(&encoded3);
		let sigs_vec3 = [alice_sig3.clone(), bob_sig3.clone()].to_vec();

		let state_proof3 = StateProof {
			app_state: app_state3,
			sigs: sigs_vec3,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof3
			)
		);
		assert_eq!(OffchainTrade::is_finalized(&condition_account), false);
		assert_eq!(OffchainTrade::get_outcome(&condition_account), false);
		assert_eq!(OffchainTrade::test_get_owner
			(condition_account.clone()), bob_public.clone());
	
	
		let app_state4 = AppState {
			nonce: 2,
			seq_num: 4,
			state: [0, 1, 1].to_vec()
		};

		let mut encoded4 = app_state4.nonce.encode();
		encoded4.extend(app_state4.seq_num.encode());
		encoded4.extend(app_state4.state.encode());

		let alice_sig4 = alice_pair.sign(&encoded4);
		let bob_sig4 = bob_pair.sign(&encoded4);
		let sigs_vec4 = [alice_sig4.clone(), bob_sig4.clone()].to_vec();

		let state_proof4 = StateProof {
			app_state: app_state4,
			sigs: sigs_vec4,
		};
		
		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof4
			),
			Error::<Test>::InvalidStateLength
		);

		
		let app_state5 = AppState {
			nonce: 2,
			seq_num: 4,
			state: [1, 1].to_vec()
		};

		let mut encoded5 = app_state5.nonce.encode();
		encoded5.extend(app_state5.seq_num.encode());
		encoded5.extend(app_state5.state.encode());

		let alice_sig5 = alice_pair.sign(&encoded5);
		let bob_sig5 = bob_pair.sign(&encoded5);
		let sigs_vec5 = [alice_sig5.clone(), bob_sig5.clone()].to_vec();

		let state_proof5 = StateProof {
			app_state: app_state5,
			sigs: sigs_vec5,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof5
			),
			Error::<Test>::InvalidState
		);

		let app_state6 = AppState {
			nonce: 3,
			seq_num: 4,
			state: [0, 1].to_vec()
		};

		let mut encoded6 = app_state6.nonce.encode();
		encoded6.extend(app_state6.seq_num.encode());
		encoded6.extend(app_state6.state.encode());

		let alice_sig6 = alice_pair.sign(&encoded6);
		let bob_sig6 = bob_pair.sign(&encoded6);
		let sigs_vec6 = [alice_sig6.clone(), bob_sig6.clone()].to_vec();

		let state_proof6 = StateProof {
			app_state: app_state6,
			sigs: sigs_vec6,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof6
			),
			Error::<Test>::InvalidNonce
		);		

		let app_state7 = AppState {
			nonce: 2,
			seq_num: 3,
			state: [0, 1].to_vec()
		};

		let mut encoded7 = app_state7.nonce.encode();
		encoded7.extend(app_state7.seq_num.encode());
		encoded7.extend(app_state7.state.encode());

		let alice_sig7 = alice_pair.sign(&encoded7);
		let bob_sig7 = bob_pair.sign(&encoded7);
		let sigs_vec7 = [alice_sig7.clone(), bob_sig7.clone()].to_vec();

		let state_proof7 = StateProof {
			app_state: app_state7,
			sigs: sigs_vec7,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof7
			),
			Error::<Test>::InvalidSeqNum
		);

		let app_state8 = AppState {
			nonce: 2,
			seq_num: 4,
			state: [0, 3].to_vec()
		};

		let mut encoded8 = app_state8.nonce.encode();
		encoded8.extend(app_state8.seq_num.encode());
		encoded8.extend(app_state8.state.encode());

		let alice_sig8 = alice_pair.sign(&encoded8);
		let bob_sig8 = bob_pair.sign(&encoded8);
		let sigs_vec8 = [alice_sig8.clone(), bob_sig8.clone()].to_vec();

		let state_proof8 = StateProof {
			app_state: app_state8,
			sigs: sigs_vec8,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof8
			),
			Error::<Test>::InvalidDIDState
		);
	});
}

#[test]
fn test_set_new_did() {
	new_test_ext().execute_with(|| {
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();

		let did_account = account_key("DID");

		assert_ok!(
			OffchainTrade::set_new_did(
				Origin::signed(alice_public.clone()),
				did_account.clone()
			)
		);
		assert_eq!(OffchainTrade::did_key(), 3);
		assert_eq!(OffchainTrade::did_list(2), Some(did_account.clone()));
		assert_eq!(OffchainTrade::key_of_did(did_account.clone()), Some(2));
	});
}