#![cfg(test)]

use std::cell::RefCell;

use crate::{Module, Trait};
use sp_runtime::Perbill;
use sp_runtime::testing::{Header, UintAuthorityId, TestXt};
use sp_runtime::traits::{IdentityLookup, BlakeTwo256, ConvertInto};
use frame_support::{impl_outer_origin, impl_outer_dispatch, parameter_types, weights::Weight};
use sp_core::{sr25519, Pair, H256};

use frame_system as system;
impl_outer_origin!{
	pub enum Origin for Test {}
}

/**
mod palllet_did_offchain_trade {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        pallet_did_offchain_trade<T>,
        frame_system<T>,
    }
}
*/

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Trait for Test {
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

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl pallet_did::Trait for Test {
    type Event = ();
    type Public = sr25519::Public;
	type Signature = sr25519::Signature;
}

impl Trait for Test {
    type Event = ();
    type Public = sr25519::Public;
    type Signature = sr25519::Signature;
}

pub type OffchainTrade = Module<Test>;
pub type System = frame_system::Module<Test>;
pub type DID = pallet_did::Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
