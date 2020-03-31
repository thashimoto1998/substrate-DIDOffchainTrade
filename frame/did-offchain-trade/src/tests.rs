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
	//type MigrateAccount = ();
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
	let sig_vec = [alice_sig, bob_sig].to_vec();


	assert_ok!(OffchainTrade::valid_signers(
		sig_vec,
		&encoded,
		signers_vec
	));
}