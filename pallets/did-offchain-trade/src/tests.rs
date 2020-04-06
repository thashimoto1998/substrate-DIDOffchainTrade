#![cfg(test)]

use super::*;
use sp_runtime::{
	testing::{Header},
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use sp_std::marker::PhantomData;
use frame_support::{
	assert_ok, assert_noop, impl_outer_origin, 
	impl_outer_event, parameter_types, weights::Weight,
};
use frame_system::{self,};
use sp_core::{sr25519, Pair, H256};
use mock::{
	Test, Origin, System, OffchainTrade, DID, new_test_ext,
};

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
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let players_vec = [alice_public.clone(), bob_public.clone()].to_vec();

		let identity = account_key("Identity");
		assert_ok!(
			DID::register_identity(
				Origin::signed(alice_public.clone()),
				identity.clone(),
			)
		);

		let condition_account = account_key("Condition");
		let nonce = 2;
		let players_vec = [alice_public.clone(), bob_public.clone()].to_vec();

		assert_ok!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				2,
				identity.clone(),
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
		assert_eq!(OffchainTrade::did_list(2), Some(identity.clone()));
		assert_eq!(OffchainTrade::key_of_did(identity.clone()), Some(2));
		assert_eq!(OffchainTrade::is_finalized(&condition_account), false);

		let risa_pair = account_pair("Risa");
		let risa_public = risa_pair.public();
		let invalid_players_vec = [alice_public.clone(), bob_public.clone(), risa_public.clone()].to_vec();
		assert_noop!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				invalid_players_vec,
				nonce,
				identity.clone(),
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

		let identity = account_key("Identity");

		assert_ok!(
			DID::register_identity(
				Origin::signed(alice_public.clone()),
				identity.clone(),
			)
		);

		let condition_account = account_key("Condition");
		let nonce = 2;
	
		assert_ok!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				2,
				identity.clone(),
				condition_account.clone()
			)
		);

		let app_state_1 = AppState {
			nonce: 2,
			seq_num: 1,
			state: [0, 2].to_vec(),
		};

		let mut encoded_1 = app_state_1.nonce.encode();
		encoded_1.extend(app_state_1.seq_num.encode());
		encoded_1.extend(app_state_1.state.encode());

		let alice_sig_1 = alice_pair.sign(&encoded_1);
		let bob_sig_1 = bob_pair.sign(&encoded_1);
		let sigs_vec_1 = [alice_sig_1.clone(), bob_sig_1.clone()].to_vec();

		let state_proof_1 = StateProof {
			app_state: app_state_1,
			sigs: sigs_vec_1,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_1
			)
		);

		assert_eq!(OffchainTrade::is_finalized(&condition_account), true);
		assert_eq!(OffchainTrade::get_outcome(&condition_account), true);
		assert_eq!(OffchainTrade::check_permissions(
			identity.clone(), bob_public.clone()), 
			true
		);

		let app_state_2 = AppState {
			nonce: 2,
			seq_num: 2,
			state: [0, 1].to_vec()
		};

		let mut encoded_2 = app_state_2.nonce.encode();
		encoded_2.extend(app_state_2.seq_num.encode());
		encoded_2.extend(app_state_2.state.encode());

		let alice_sig_2 = alice_pair.sign(&encoded_2);
		let bob_sig_2 = bob_pair.sign(&encoded_2);
		let sigs_vec_2 = [alice_sig_2.clone(), bob_sig_2.clone()].to_vec();

		let state_proof_2 = StateProof {
			app_state: app_state_2,
			sigs: sigs_vec_2,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_2
			)
		);

		/**
		expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::SetIdle(
					condition_account.clone(),
					System::block_number()
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));
		*/
		assert_eq!(OffchainTrade::is_finalized(&condition_account), false);
		assert_eq!(OffchainTrade::get_outcome(&condition_account), false);
	

		let app_state_3 = AppState {
			nonce: 2,
			seq_num: 3,
			state: [0, 0].to_vec()
		};

		let mut encoded_3 = app_state_3.nonce.encode();
		encoded_3.extend(app_state_3.seq_num.encode());
		encoded_3.extend(app_state_3.state.encode());

		let alice_sig_3 = alice_pair.sign(&encoded_3);
		let bob_sig_3 = bob_pair.sign(&encoded_3);
		let sigs_vec_3 = [alice_sig_3.clone(), bob_sig_3.clone()].to_vec();

		let state_proof_3 = StateProof {
			app_state: app_state_3,
			sigs: sigs_vec_3,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_3
			)
		);

		/**
		expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::SwapPosition(
					condition_account.clone(),
					System::block_number()
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));
		*/
		assert_eq!(OffchainTrade::is_finalized(&condition_account), false);
		assert_eq!(OffchainTrade::get_outcome(&condition_account), false);
		assert_eq!(OffchainTrade::test_get_owner
			(condition_account.clone()), bob_public.clone());
	
	
		let app_state_4 = AppState {
			nonce: 2,
			seq_num: 4,
			state: [0, 1, 1].to_vec()
		};

		let mut encoded_4 = app_state_4.nonce.encode();
		encoded_4.extend(app_state_4.seq_num.encode());
		encoded_4.extend(app_state_4.state.encode());

		let alice_sig_4 = alice_pair.sign(&encoded_4);
		let bob_sig_4 = bob_pair.sign(&encoded_4);
		let sigs_vec_4 = [alice_sig_4.clone(), bob_sig_4.clone()].to_vec();

		let state_proof_4 = StateProof {
			app_state: app_state_4,
			sigs: sigs_vec_4,
		};
		
		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_4
			),
			Error::<Test>::InvalidStateLength
		);

		
		let app_state_5 = AppState {
			nonce: 2,
			seq_num: 4,
			state: [1, 1].to_vec()
		};

		let mut encoded_5 = app_state_5.nonce.encode();
		encoded_5.extend(app_state_5.seq_num.encode());
		encoded_5.extend(app_state_5.state.encode());

		let alice_sig_5 = alice_pair.sign(&encoded_5);
		let bob_sig_5 = bob_pair.sign(&encoded_5);
		let sigs_vec_5 = [alice_sig_5.clone(), bob_sig_5.clone()].to_vec();

		let state_proof_5 = StateProof {
			app_state: app_state_5,
			sigs: sigs_vec_5,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_5
			),
			Error::<Test>::InvalidState
		);

		let app_state_6 = AppState {
			nonce: 3,
			seq_num: 4,
			state: [0, 1].to_vec()
		};

		let mut encoded_6 = app_state_6.nonce.encode();
		encoded_6.extend(app_state_6.seq_num.encode());
		encoded_6.extend(app_state_6.state.encode());

		let alice_sig_6 = alice_pair.sign(&encoded_6);
		let bob_sig_6 = bob_pair.sign(&encoded_6);
		let sigs_vec_6 = [alice_sig_6.clone(), bob_sig_6.clone()].to_vec();

		let state_proof_6 = StateProof {
			app_state: app_state_6,
			sigs: sigs_vec_6,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_6
			),
			Error::<Test>::InvalidNonce
		);		

		let app_state_7 = AppState {
			nonce: 2,
			seq_num: 3,
			state: [0, 1].to_vec()
		};

		let mut encoded_7 = app_state_7.nonce.encode();
		encoded_7.extend(app_state_7.seq_num.encode());
		encoded_7.extend(app_state_7.state.encode());

		let alice_sig_7 = alice_pair.sign(&encoded_7);
		let bob_sig_7 = bob_pair.sign(&encoded_7);
		let sigs_vec_7 = [alice_sig_7.clone(), bob_sig_7.clone()].to_vec();

		let state_proof_7 = StateProof {
			app_state: app_state_7,
			sigs: sigs_vec_7,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_7
			),
			Error::<Test>::InvalidSeqNum
		);

		let app_state_8 = AppState {
			nonce: 2,
			seq_num: 4,
			state: [0, 3].to_vec()
		};

		let mut encoded_8 = app_state_8.nonce.encode();
		encoded_8.extend(app_state_8.seq_num.encode());
		encoded_8.extend(app_state_8.state.encode());

		let alice_sig_8 = alice_pair.sign(&encoded_8);
		let bob_sig_8 = bob_pair.sign(&encoded_8);
		let sigs_vec_8 = [alice_sig_8.clone(), bob_sig_8.clone()].to_vec();

		let state_proof_8 = StateProof {
			app_state: app_state_8,
			sigs: sigs_vec_8,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_8
			),
			Error::<Test>::InvalidDIDState
		);
		
		let app_state_9 = AppState {
			nonce: 2,
			seq_num: 4,
			state: [0, 0].to_vec(),
		};
		
		let mut encoded_9 = app_state_9.nonce.encode();
		encoded_9.extend(app_state_9.seq_num.encode());
		encoded_9.extend(app_state_9.state.encode());
		
		let risa_pair = account_pair("Risa");
		let risa_public = risa_pair.public();

		let alice_sig_9 = alice_pair.sign(&encoded_9);
		let risa_sig = risa_pair.sign(&encoded_9);
		let sigs_vec_9 = [alice_sig_9.clone(), risa_sig.clone()].to_vec();

		let state_proof_9 = StateProof {
			app_state: app_state_9,
			sigs: sigs_vec_9,
		};
		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_9
			),
			Error::<Test>::InvalidSignature
		);

	});
}

#[test]
fn test_set_new_did() {
	new_test_ext().execute_with(|| {
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();

		let identity = account_key("Identity");

		assert_ok!(
			DID::register_identity(
				Origin::signed(alice_public.clone()),
				identity.clone(),
			)
		);

		assert_ok!(
			OffchainTrade::set_new_did(
				Origin::signed(alice_public.clone()),
				identity.clone()
			)
		);

		/**
		let expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::NewDID(
					identity.clone(),
					2
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));
		*/
		assert_eq!(OffchainTrade::did_key(), 3);
		assert_eq!(OffchainTrade::did_list(2), Some(identity.clone()));
		assert_eq!(OffchainTrade::key_of_did(identity.clone()), Some(2));
	});
}

#[test]
fn test_another_did_trade(){
	new_test_ext().execute_with(|| {
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let players_vec = [alice_public.clone(), bob_public.clone()].to_vec();
		
		let identity_1 = account_key("Identity1");
		assert_ok!(
			DID::register_identity(
				Origin::signed(alice_public.clone()),
				identity_1.clone(),
			)
		);

		let identity_2 = account_key("Identity2");
		assert_ok!(
			DID::register_identity(
				Origin::signed(alice_public.clone()),
				identity_2.clone(),
			)
		);
		
		let condition_account = account_key("Condition");

		assert_ok!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				2,
				identity_1.clone(),
				condition_account.clone()
			)
		);

		let app_state_1 = AppState {
			nonce: 2,
			seq_num: 1,
			state: [0, 2].to_vec(),
		};

		let mut encoded_1 = app_state_1.nonce.encode();
		encoded_1.extend(app_state_1.seq_num.encode());
		encoded_1.extend(app_state_1.state.encode());

		let alice_sig_1 = alice_pair.sign(&encoded_1);
		let bob_sig_1 = bob_pair.sign(&encoded_1);
		let sigs_vec_1 = [alice_sig_1.clone(), bob_sig_1.clone()].to_vec();

		let state_proof_1 = StateProof {
			app_state: app_state_1,
			sigs: sigs_vec_1,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_1
			)
		);

		assert_ok!(
			OffchainTrade::set_new_did(
				Origin::signed(alice_public.clone()),
				identity_2.clone()
			)
		);
		assert_eq!(OffchainTrade::key_of_did(identity_2.clone()), Some(3));

		let app_state_2 = AppState {
			nonce: 2,
			seq_num: 2,
			state: [0, 3].to_vec(),
		};

		let mut encoded_2 = app_state_2.nonce.encode();
		encoded_2.extend(app_state_2.seq_num.encode());
		encoded_2.extend(app_state_2.state.encode());

		let alice_sig_2 = alice_pair.sign(&encoded_2);
		let bob_sig_2 = bob_pair.sign(&encoded_2);
		let sigs_vec_2 = [alice_sig_2.clone(), bob_sig_2.clone()].to_vec();

		let state_proof_2 = StateProof {
			app_state: app_state_2,
			sigs: sigs_vec_2,
		};

		assert_noop!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_2
			),
			Error::<Test>::NotIdleStatus
		);
		
		let app_state_3 = AppState {
			nonce: 2,
			seq_num: 2,
			state: [0, 1].to_vec(),
		};

		let mut encoded_3 = app_state_3.nonce.encode();
		encoded_3.extend(app_state_3.seq_num.encode());
		encoded_3.extend(app_state_3.state.encode());

		let alice_sig_3 = alice_pair.sign(&encoded_3);
		let bob_sig_3 = bob_pair.sign(&encoded_3);
		let sigs_vec_3 = [alice_sig_3.clone(), bob_sig_3.clone()].to_vec();

		let state_proof_3 = StateProof {
			app_state: app_state_3,
			sigs: sigs_vec_3,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_3
			)
		);
		assert_eq!(OffchainTrade::is_finalized(&condition_account), false);

		let app_state_4 = AppState {
			nonce: 2,
			seq_num: 3,
			state: [0, 3].to_vec(),
		};

		let mut encoded_4 = app_state_4.nonce.encode();
		encoded_4.extend(app_state_4.seq_num.encode());
		encoded_4.extend(app_state_4.state.encode());

		let alice_sig_4 = alice_pair.sign(&encoded_4);
		let bob_sig_4 = bob_pair.sign(&encoded_4);
		let sigs_vec_4 = [alice_sig_4.clone(), bob_sig_4.clone()].to_vec();

		let state_proof_4 = StateProof {
			app_state: app_state_4,
			sigs: sigs_vec_4,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_4
			)
		);
		assert_eq!((OffchainTrade::is_finalized(&condition_account)), true);
		assert_eq!((OffchainTrade::get_outcome(&condition_account)), true);
		assert_eq!(
			(OffchainTrade::check_permissions(identity_2.clone(), bob_public.clone())), 
			true
		);
	});
}

#[test]
fn test_another_did_trade_and_swap_owner_grantee() {
	new_test_ext().execute_with(|| {
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let players_vec = [alice_public.clone(), bob_public.clone()].to_vec();

		let identity_1 = account_key("Identity1");
		assert_ok!(
			DID::register_identity(
				Origin::signed(alice_public.clone()),
				identity_1.clone(),
			)
		);

		let identity_2 = account_key("Identity2");
		assert_ok!(
			DID::register_identity(
				Origin::signed(bob_public.clone()),
				identity_2.clone(),
			)
		);
		
		let condition_account = account_key("Condition");

		assert_ok!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				2,
				identity_1.clone(),
				condition_account.clone()
			)
		);

		let app_state_1 = AppState {
			nonce: 2,
			seq_num: 1,
			state: [0, 2].to_vec(),
		};

		let mut encoded_1 = app_state_1.nonce.encode();
		encoded_1.extend(app_state_1.seq_num.encode());
		encoded_1.extend(app_state_1.state.encode());

		let alice_sig_1 = alice_pair.sign(&encoded_1);
		let bob_sig_1 = bob_pair.sign(&encoded_1);
		let sigs_vec_1 = [alice_sig_1.clone(), bob_sig_1.clone()].to_vec();

		let state_proof_1 = StateProof {
			app_state: app_state_1,
			sigs: sigs_vec_1,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_1
			)
		);

		assert_ok!(
			OffchainTrade::set_new_did(
				Origin::signed(bob_public.clone()),
				identity_2.clone()
			)
		);
		assert_eq!(OffchainTrade::key_of_did(identity_2.clone()), Some(3));

		let app_state_2 = AppState {
			nonce: 2,
			seq_num: 2,
			state: [0, 0].to_vec(),
		};

		let mut encoded_2 = app_state_2.nonce.encode();
		encoded_2.extend(app_state_2.seq_num.encode());
		encoded_2.extend(app_state_2.state.encode());

		let alice_sig_2 = alice_pair.sign(&encoded_2);
		let bob_sig_2 = bob_pair.sign(&encoded_2);
		let sigs_vec_2 = [alice_sig_2.clone(), bob_sig_2.clone()].to_vec();

		let state_proof_2 = StateProof {
			app_state: app_state_2,
			sigs: sigs_vec_2,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_2
			)
		);
		assert_eq!(OffchainTrade::is_finalized(&condition_account), false);
		assert_eq!(OffchainTrade::test_get_owner(condition_account.clone()), bob_public.clone());

		let app_state_3 = AppState {
			nonce: 2,
			seq_num: 3,
			state: [0, 3].to_vec(),
		};

		let mut encoded_3 = app_state_3.nonce.encode();
		encoded_3.extend(app_state_3.seq_num.encode());
		encoded_3.extend(app_state_3.state.encode());

		let alice_sig_3 = alice_pair.sign(&encoded_3);
		let bob_sig_3 = bob_pair.sign(&encoded_3);
		let sigs_vec_3 = [alice_sig_3.clone(), bob_sig_3.clone()].to_vec();

		let state_proof_3 = StateProof {
			app_state: app_state_3,
			sigs: sigs_vec_3,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_3
			)
		);
		assert_eq!((OffchainTrade::is_finalized(&condition_account)), true);
		assert_eq!((OffchainTrade::get_outcome(&condition_account)), true);
		assert_eq!(
			(OffchainTrade::check_permissions(identity_2.clone(), alice_public.clone())), 
			true
		);
	});
}

#[test]
fn test_did_trade_with_two_grantee() {
	new_test_ext().execute_with(|| {
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let players_vec_1 = [alice_public.clone(), bob_public.clone()].to_vec();

		let condition_account_1 = account_key("Condition1");

		let identity = account_key("Identity");
		assert_ok!(
			DID::register_identity(
				Origin::signed(alice_public.clone()),
				identity.clone(),
			)
		);

		assert_ok!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec_1,
				2,
				identity.clone(),
				condition_account_1.clone()
			)
		);

		let app_state_1 = AppState {
			nonce: 2,
			seq_num: 1,
			state: [0, 2].to_vec(),
		};

		let mut encoded_1 = app_state_1.nonce.encode();
		encoded_1.extend(app_state_1.seq_num.encode());
		encoded_1.extend(app_state_1.state.encode());

		let alice_sig_1 = alice_pair.sign(&encoded_1);
		let bob_sig_1 = bob_pair.sign(&encoded_1);
		let sigs_vec_1 = [alice_sig_1.clone(), bob_sig_1.clone()].to_vec();

		let state_proof_1 = StateProof {
			app_state: app_state_1,
			sigs: sigs_vec_1,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_1
			)
		);


		let risa_pair = account_pair("Risa");
		let risa_public = risa_pair.public();
		let players_vec_2 = [alice_public.clone(), risa_public.clone()].to_vec();

		let condition_account_2 = account_key("Condition2");

		assert_ok!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec_2,
				3,
				identity.clone(),
				condition_account_2.clone()
			)
		);

		let app_state_2 = AppState {
			nonce: 3,
			seq_num: 1,
			state: [1, 2].to_vec(),
		};

		let mut encoded_2 = app_state_2.nonce.encode();
		encoded_2.extend(app_state_2.seq_num.encode());
		encoded_2.extend(app_state_2.state.encode());

		let alice_sig_2 = alice_pair.sign(&encoded_2);
		let risa_sig = risa_pair.sign(&encoded_2);
		let sigs_vec_2 = [alice_sig_2.clone(), risa_sig.clone()].to_vec();

		let state_proof_2 = StateProof {
			app_state: app_state_2,
			sigs: sigs_vec_2,
		};

		assert_ok!(
			OffchainTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_2
			)
		);

		assert_eq!((OffchainTrade::is_finalized(&condition_account_2)), true);
		assert_eq!((OffchainTrade::get_outcome(&condition_account_2)), true);
		assert_eq!(
			(OffchainTrade::check_permissions(identity.clone(), risa_public.clone())), 
			true
		);

		let miwa_pair = account_pair("Miwa");
		let miwa_public = miwa_pair.public();
		let players_vec_3 = [alice_public.clone(), miwa_public.clone()].to_vec();

		assert_noop!(
			OffchainTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec_3,
				4,
				identity.clone(),
				condition_account_1.clone()
			),
			Error::<Test>::ExistAddress
		);
	});
}
