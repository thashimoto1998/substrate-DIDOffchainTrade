#![cfg(test)]

use super::*;
use frame_support::{assert_ok, assert_noop};
use frame_system;
use sp_core::{sr25519, Pair};
use mock::{
	Test, Origin, System, DIDTrade, DID, new_test_ext, TestEvent,
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
		
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let signers_vec = [alice_public.clone(), bob_public.clone()].to_vec();

		let identity = account_key("Identity");
		let condition_address = account_key("Condition");

		let state = State {
			condition_address: condition_address,
			op: 2,
			did: Some(identity),
		};

		let app_state = AppState {
			nonce: nonce,
			seq_num: seq_num,
			state: state,
		};

		let mut encoded = app_state.nonce.encode();
		encoded.extend(app_state.seq_num.encode());
		encoded.extend(app_state.state.condition_address.encode());
		encoded.extend(app_state.state.op.encode());
		encoded.extend(app_state.state.did.unwrap().encode());

		let alice_sig = alice_pair.sign(&encoded);
		let bob_sig = bob_pair.sign(&encoded);
		let sig_vec = [alice_sig.clone(), bob_sig.clone()].to_vec();

		assert_ok!(DIDTrade::valid_signers(
			sig_vec,
			&encoded,
			signers_vec
		));

		let risa_pair = account_pair("Risa");
		let risa_public = risa_pair.public();
		let invalid_signers_vec = [alice_public.clone(), risa_public.clone()].to_vec();
		let invalid_sig_vec = [alice_sig.clone(), bob_sig.clone()].to_vec();

		assert_noop!(
			DIDTrade::valid_signers(
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

		let condition_account_1 = account_key("Condition");
		let nonce = 2;
		let players_vec_1 = [alice_public.clone(), bob_public.clone()].to_vec();
		assert_ok!(
			DIDTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				2,
				identity.clone(),
				condition_account_1.clone()
			)
		);
		let expected_event = TestEvent::pallet_did_offchain_trade(
			RawEvent::AccessConditionCreated(
				condition_account_1.clone(),
				alice_public.clone(),
				bob_public.clone(),
			)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));
		assert_eq!(DIDTrade::is_finalized(&condition_account_1), false);
		assert_eq!(DIDTrade::get_outcome(&condition_account_1), false);


		let risa_pair = account_pair("Risa");
		let risa_public = risa_pair.public();
		let invalid_players_vec = [alice_public.clone(), bob_public.clone(), risa_public.clone()].to_vec();
		assert_noop!(
			DIDTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				invalid_players_vec,
				nonce,
				identity.clone(),
				condition_account_1.clone()
			),
			Error::<Test>::InvalidPlayerLength
		);

		let invalid_identity = account_key("Identity_2");
		assert_noop!(
			DIDTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec_1,
				nonce,
				invalid_identity.clone(),
				condition_account_1.clone()
			),
			Error::<Test>::NotExist
		);


		let identity_2 = account_key("Identity2");
		assert_ok!(
			DID::register_identity(
				Origin::signed(risa_public.clone()),
				identity_2.clone(),
			)
		);

		let invalid_players_vec_2 = [alice_public.clone(), bob_public.clone()].to_vec();
		assert_noop!(
			DIDTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				invalid_players_vec_2,
				nonce,
				identity_2.clone(),
				condition_account_1.clone()
			),
			Error::<Test>::NotOwner
		);


		let invalid_condition_address = condition_account_1.clone();
		let players_vec_2 = [alice_public.clone(), risa_public.clone()].to_vec();
		assert_noop!(
			DIDTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec_2,
				nonce,
				identity_2.clone(),
				invalid_condition_address.clone()
			),
			Error::<Test>::ExistAddress
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
			DIDTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				nonce,
				identity.clone(),
				condition_account.clone()
			)
		);

		let state_1 = State {
			condition_address: condition_account,
			op: 2,
			did: Some(identity),
		};

		let app_state_1 = AppState {
			nonce: nonce,
			seq_num: 1,
			state: state_1,
		};

		let mut encoded_1 = app_state_1.nonce.encode();
		encoded_1.extend(app_state_1.seq_num.encode());
		encoded_1.extend(app_state_1.state.condition_address.clone().encode());
		encoded_1.extend(app_state_1.state.op.encode());
		encoded_1.extend(app_state_1.state.did.unwrap().clone().encode());

		let alice_sig_1 = alice_pair.sign(&encoded_1);
		let bob_sig_1 = bob_pair.sign(&encoded_1);
		let sig_vec_1 = [alice_sig_1.clone(), bob_sig_1.clone()].to_vec();

		
		let state_proof_1 = StateProof {
			app_state: app_state_1,
			sigs: sig_vec_1
		};

		assert_ok!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_1
			)
		);

		let mut expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::IntendSettle(
					condition_account.clone(),
					System::block_number(),
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));

		assert_eq!(DIDTrade::is_finalized(&condition_account), true);
		assert_eq!(DIDTrade::get_outcome(&condition_account), true);
		assert_eq!(DIDTrade::check_permissions(
			identity.clone(), bob_public.clone()), 
			true
		);

		let invalid_condition_account = account_key("Invalid");

		let state_2 = State {
			condition_address: invalid_condition_account,
			op: 2,
			did: Some(identity),
		};

		let app_state_2 = AppState {
			nonce: nonce,
			seq_num: 1,
			state: state_2,
		};

		let mut encoded_2 = app_state_2.nonce.encode();
		encoded_2.extend(app_state_2.seq_num.encode());
		encoded_2.extend(app_state_2.state.condition_address.clone().encode());
		encoded_2.extend(app_state_2.state.op.encode());
		encoded_2.extend(app_state_2.state.did.unwrap().clone().encode());

		let alice_sig_2 = alice_pair.sign(&encoded_2);
		let bob_sig_2 = bob_pair.sign(&encoded_2);
		let sig_vec_2 = [alice_sig_2.clone(), bob_sig_2.clone()].to_vec();

		
		let state_proof_2 = StateProof {
			app_state: app_state_2,
			sigs: sig_vec_2
		};

		assert_noop!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_2
			),
			Error::<Test>::InvalidConditionAddress
		);


		let invalid_nonce = 3;
		let state_3 = State {
			condition_address: condition_account,
			op: 2,
			did: Some(identity),
		};

		let app_state_3 = AppState {
			nonce: invalid_nonce,
			seq_num: 3,
			state: state_3,
		};

		let mut encoded_3 = app_state_3.nonce.encode();
		encoded_3.extend(app_state_3.seq_num.encode());
		encoded_3.extend(app_state_3.state.condition_address.clone().encode());
		encoded_3.extend(app_state_3.state.op.encode());
		encoded_3.extend(app_state_3.state.did.unwrap().clone().encode());

		let alice_sig_3 = alice_pair.sign(&encoded_3);
		let bob_sig_3 = bob_pair.sign(&encoded_3);
		let sig_vec_3 = [alice_sig_3.clone(), bob_sig_3.clone()].to_vec();

		
		let state_proof_3 = StateProof {
			app_state: app_state_3,
			sigs: sig_vec_3
		};

		assert_noop!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_3
			),
			Error::<Test>::InvalidNonce
		);
		

		let invalid_seq_num = 0;
		let state_4 = State {
			condition_address: condition_account,
			op: 2,
			did: Some(identity),
		};

		let app_state_4 = AppState {
			nonce: nonce,
			seq_num: invalid_seq_num,
			state: state_4,
		};

		let mut encoded_4 = app_state_4.nonce.encode();
		encoded_4.extend(app_state_4.seq_num.encode());
		encoded_4.extend(app_state_4.state.condition_address.clone().encode());
		encoded_4.extend(app_state_4.state.op.encode());
		encoded_4.extend(app_state_4.state.did.unwrap().clone().encode());

		let alice_sig_4 = alice_pair.sign(&encoded_3);
		let bob_sig_4 = bob_pair.sign(&encoded_3);
		let sig_vec_4 = [alice_sig_4.clone(), bob_sig_4.clone()].to_vec();

		
		let state_proof_4 = StateProof {
			app_state: app_state_4,
			sigs: sig_vec_4
		};

		assert_noop!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_4
			),
			Error::<Test>::InvalidSeqNum
		);
	});
}

#[test]
fn test_another_did_trade() {
	new_test_ext().execute_with(|| {
		let alice_pair = account_pair("Alice");
		let alice_public = alice_pair.public();
		let bob_pair = account_pair("Bob");
		let bob_public = bob_pair.public();
		let players_vec = [alice_public.clone(), bob_public.clone()].to_vec();
		let nonce = 2;

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
			DIDTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				2,
				identity_1.clone(),
				condition_account.clone()
			)
		);

		let state_1 = State {
			condition_address: condition_account,
			op: 2,
			did: Some(identity_1.clone()),
		};

		let app_state_1 = AppState {
			nonce: nonce,
			seq_num: 1,
			state: state_1,
		};

		let mut encoded_1 = app_state_1.nonce.encode();
		encoded_1.extend(app_state_1.seq_num.encode());
		encoded_1.extend(app_state_1.state.condition_address.clone().encode());
		encoded_1.extend(app_state_1.state.op.encode());
		encoded_1.extend(app_state_1.state.did.unwrap().clone().encode());

		let alice_sig_1 = alice_pair.sign(&encoded_1);
		let bob_sig_1 = bob_pair.sign(&encoded_1);
		let sig_vec_1 = [alice_sig_1.clone(), bob_sig_1.clone()].to_vec();

		
		let state_proof_1 = StateProof {
			app_state: app_state_1,
			sigs: sig_vec_1
		};

		assert_ok!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_1
			)
		);

		let mut expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::IntendSettle(
					condition_account.clone(),
					System::block_number(),
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));

		assert_eq!(DIDTrade::is_finalized(&condition_account), true);
		assert_eq!(DIDTrade::get_outcome(&condition_account), true);
		assert_eq!(DIDTrade::check_permissions(
			identity_1.clone(), bob_public.clone()), 
			true
		);

		let state_2 = State {
			condition_address: condition_account,
			op: 1,
			did: None,
		};

		let app_state_2 = AppState {
			nonce: nonce,
			seq_num: 2,
			state: state_2,
		};

		let mut encoded_2 = app_state_2.nonce.encode();
		encoded_2.extend(app_state_2.seq_num.encode());
		encoded_2.extend(app_state_2.state.condition_address.clone().encode());
		encoded_2.extend(app_state_2.state.op.encode());

		let alice_sig_2 = alice_pair.sign(&encoded_2);
		let bob_sig_2 = bob_pair.sign(&encoded_2);
		let sig_vec_2 = [alice_sig_2.clone(), bob_sig_2.clone()].to_vec();

		
		let state_proof_2 = StateProof {
			app_state: app_state_2,
			sigs: sig_vec_2
		};

		assert_ok!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_2
			)
		);

		expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::SetIdle(
					condition_account.clone(),
					System::block_number()
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));
		
		assert_eq!(DIDTrade::is_finalized(&condition_account), false);
		assert_eq!(DIDTrade::get_outcome(&condition_account), false);

		let state_3 = State {
			condition_address: condition_account,
			op: 2,
			did: Some(identity_2.clone()),
		};

		let app_state_3 = AppState {
			nonce: nonce,
			seq_num: 3,
			state: state_3,
		};

		let mut encoded_3 = app_state_3.nonce.encode();
		encoded_3.extend(app_state_3.seq_num.encode());
		encoded_3.extend(app_state_3.state.condition_address.clone().encode());
		encoded_3.extend(app_state_3.state.op.encode());
		encoded_3.extend(app_state_3.state.did.unwrap().clone().encode());

		let alice_sig_3 = alice_pair.sign(&encoded_3);
		let bob_sig_3 = bob_pair.sign(&encoded_3);
		let sig_vec_3 = [alice_sig_3.clone(), bob_sig_3.clone()].to_vec();

		
		let state_proof_3 = StateProof {
			app_state: app_state_3,
			sigs: sig_vec_3
		};

		assert_ok!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_3
			)
		);

		let mut expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::IntendSettle(
					condition_account.clone(),
					System::block_number(),
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));

		assert_eq!(DIDTrade::is_finalized(&condition_account), true);
		assert_eq!(DIDTrade::get_outcome(&condition_account), true);
		assert_eq!(DIDTrade::check_permissions(
			identity_2.clone(), bob_public.clone()), 
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
		let nonce = 2;

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
			DIDTrade::create_access_condition(
				Origin::signed(alice_public.clone()),
				players_vec,
				2,
				identity_1.clone(),
				condition_account.clone()
			)
		);

		let state_1 = State {
			condition_address: condition_account,
			op: 2,
			did: Some(identity_1.clone()),
		};

		let app_state_1 = AppState {
			nonce: nonce,
			seq_num: 1,
			state: state_1,
		};

		let mut encoded_1 = app_state_1.nonce.encode();
		encoded_1.extend(app_state_1.seq_num.encode());
		encoded_1.extend(app_state_1.state.condition_address.clone().encode());
		encoded_1.extend(app_state_1.state.op.encode());
		encoded_1.extend(app_state_1.state.did.unwrap().clone().encode());

		let alice_sig_1 = alice_pair.sign(&encoded_1);
		let bob_sig_1 = bob_pair.sign(&encoded_1);
		let sig_vec_1 = [alice_sig_1.clone(), bob_sig_1.clone()].to_vec();

		
		let state_proof_1 = StateProof {
			app_state: app_state_1,
			sigs: sig_vec_1
		};

		assert_ok!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_1
			)
		);

		let state_2 = State {
			condition_address: condition_account,
			op: 0,
			did: None,
		};

		let app_state_2 = AppState {
			nonce: nonce,
			seq_num: 2,
			state: state_2,
		};

		let mut encoded_2 = app_state_2.nonce.encode();
		encoded_2.extend(app_state_2.seq_num.encode());
		encoded_2.extend(app_state_2.state.condition_address.clone().encode());
		encoded_2.extend(app_state_2.state.op.encode());

		let alice_sig_2 = alice_pair.sign(&encoded_2);
		let bob_sig_2 = bob_pair.sign(&encoded_2);
		let sig_vec_2 = [alice_sig_2.clone(), bob_sig_2.clone()].to_vec();

		
		let state_proof_2 = StateProof {
			app_state: app_state_2,
			sigs: sig_vec_2
		};

		assert_ok!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_2
			)
		);

		let mut expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::SwapPosition(
					condition_account.clone(),
					System::block_number()
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));
		
		assert_eq!(DIDTrade::is_finalized(&condition_account), false);
		assert_eq!(DIDTrade::get_outcome(&condition_account), false);

		let state_3 = State {
			condition_address: condition_account,
			op: 2,
			did: Some(identity_2.clone()),
		};

		let app_state_3 = AppState {
			nonce: nonce,
			seq_num: 3,
			state: state_3,
		};

		let mut encoded_3 = app_state_3.nonce.encode();
		encoded_3.extend(app_state_3.seq_num.encode());
		encoded_3.extend(app_state_3.state.condition_address.clone().encode());
		encoded_3.extend(app_state_3.state.op.encode());
		encoded_3.extend(app_state_3.state.did.unwrap().clone().encode());

		let alice_sig_3 = alice_pair.sign(&encoded_3);
		let bob_sig_3 = bob_pair.sign(&encoded_3);
		let sig_vec_3 = [alice_sig_3.clone(), bob_sig_3.clone()].to_vec();

		
		let state_proof_3 = StateProof {
			app_state: app_state_3,
			sigs: sig_vec_3
		};

		assert_ok!(
			DIDTrade::intend_settle(
				Origin::signed(alice_public.clone()),
				state_proof_3
			)
		);

		let mut expected_event = TestEvent::pallet_did_offchain_trade(
				RawEvent::IntendSettle(
					condition_account.clone(),
					System::block_number(),
				)
		);
		assert!(System::events().iter().any(|a| a.event == expected_event));

		assert_eq!(DIDTrade::is_finalized(&condition_account), true);
		assert_eq!(DIDTrade::get_outcome(&condition_account), true);
		assert_eq!(DIDTrade::check_permissions(
			identity_2.clone(), alice_public.clone()), 
			true
		);
	});
}