use crate::{mock::*, ElectionInfo, Error, Event, Votes};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::Hash;

pub type HashType = <Test as frame_system::Config>::Hash;
pub type Hashing = <Test as frame_system::Config>::Hashing;
pub type AccountId = <Test as frame_system::Config>::AccountId;

#[test]
fn raise_join_request_as_candidate_successfully() {
	new_test_ext().execute_with(|| {
		assert_ok!(TemplateModule::raise_join_request_as_candidate(RuntimeOrigin::signed(1)));
	});
}

#[test]
fn raise_join_request_as_candidate_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(TemplateModule::raise_join_request_as_candidate(RuntimeOrigin::signed(1)));
		assert_noop!(
			TemplateModule::raise_join_request_as_candidate(RuntimeOrigin::signed(1)),
			Error::<Test>::UserAlreadyPresent
		);
	});
}

#[test]
fn start_election_successfully() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));
		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};

		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info
		));
	});
}

#[test]
fn start_duplicate_election_fail() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));
		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};

		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info.clone()
		));
		assert_noop!(
			TemplateModule::start_election(RuntimeOrigin::root(), election_hash, election_info),
			Error::<Test>::ElectionAlreadyStarted
		);
	});
}

#[test]
fn cast_vote_successfully() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));
		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};
		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info
		));

		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(1),
			ALICE,
			election_hash,
			Votes::Yes
		));
	});
}

#[test]
fn cast_vote_on_invalid_election_hash_fail() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));
		let temp_election_hash = HashType::from(Hashing::hash_of(&41));

		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};
		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info
		));

		assert_noop!(
			TemplateModule::cast_vote(
				RuntimeOrigin::signed(1),
				ALICE,
				temp_election_hash,
				Votes::Yes
			),
			Error::<Test>::InvalidElectionHash
		);
	});
}

#[test]
fn cast_duplicate_vote_fail() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));

		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};
		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info
		));

		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(1),
			ALICE,
			election_hash,
			Votes::Yes
		));
		assert_noop!(
			TemplateModule::cast_vote(RuntimeOrigin::signed(1), ALICE, election_hash, Votes::Yes),
			Error::<Test>::DuplicateVoteNotAllowed
		);
	});
}

#[test]
fn cast_vote_on_inactive_election_fail() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));

		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};
		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info
		));

		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(1),
			ALICE,
			election_hash,
			Votes::Yes
		));
		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(2),
			CHARLIE,
			election_hash,
			Votes::Yes
		));
		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(3),
			CHARLIE,
			election_hash,
			Votes::Yes
		));

		assert_ok!(TemplateModule::stop_election(RuntimeOrigin::root(), election_hash));
		assert_noop!(
			TemplateModule::cast_vote(RuntimeOrigin::signed(4), ALICE, election_hash, Votes::Yes),
			Error::<Test>::InactiveElection
		);
	});
}

#[test]
fn stop_election_successfully() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));

		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};
		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info
		));

		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(1),
			ALICE,
			election_hash,
			Votes::Yes
		));
		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(2),
			CHARLIE,
			election_hash,
			Votes::Yes
		));
		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(3),
			CHARLIE,
			election_hash,
			Votes::Yes
		));

		assert_ok!(TemplateModule::stop_election(RuntimeOrigin::root(), election_hash));
	});
}

#[test]
fn stop_already_inactive_election_fail() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));

		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};
		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info
		));

		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(1),
			ALICE,
			election_hash,
			Votes::Yes
		));
		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(2),
			CHARLIE,
			election_hash,
			Votes::Yes
		));
		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(3),
			CHARLIE,
			election_hash,
			Votes::Yes
		));

		assert_ok!(TemplateModule::stop_election(RuntimeOrigin::root(), election_hash));
		assert_noop!(
			TemplateModule::stop_election(RuntimeOrigin::root(), election_hash),
			Error::<Test>::InactiveElection
		);
	});
}

#[test]
fn get_winner_information_successfully() {
	new_test_ext().execute_with(|| {
		let election_hash = HashType::from(Hashing::hash_of(&40));

		const ALICE: <Test as frame_system::Config>::AccountId = 1;
		const CHARLIE: <Test as frame_system::Config>::AccountId = 3;

		let election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: Vec::new(),
			status: true,
			winner: None,
		};
		assert_ok!(TemplateModule::start_election(
			RuntimeOrigin::root(),
			election_hash,
			election_info
		));

		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(1),
			ALICE,
			election_hash,
			Votes::Yes
		));
		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(2),
			CHARLIE,
			election_hash,
			Votes::Yes
		));
		assert_ok!(TemplateModule::cast_vote(
			RuntimeOrigin::signed(3),
			CHARLIE,
			election_hash,
			Votes::Yes
		));

		assert_ok!(TemplateModule::stop_election(RuntimeOrigin::root(), election_hash));

		let final_election_info = ElectionInfo::<AccountId> {
			all_candidates: vec![ALICE, CHARLIE],
			voters_list: vec![1, 2, 3],
			status: false,
			winner: Some(CHARLIE),
		};

		assert_eq!(TemplateModule::election(election_hash), Some(final_election_info));
	});
}
