#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
	codec::{Decode, Encode},
	dispatch::Vec,
	sp_runtime::RuntimeDebug,
};
use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum Votes {
	Yes,
	No,
}

/// election_hash -> ElectionInfo {all_candidates, voter_list, status }
/// candidate -> CandidateInfo {total_aye, total_nay, election_hash }
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ElectionInfo<AccountId> {
	all_candidates: Vec<AccountId>,
	voters_list: Vec<AccountId>,
	status: bool,
	winner: Option<AccountId>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CandidateInfo<AccountId, Hash> {
	election_hash: Hash,
	total_aye_votes: Vec<AccountId>,
	total_naye_votes: Vec<AccountId>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn election)]
	pub type Election<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, ElectionInfo<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn candidate)]
	pub type Candidate<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		CandidateInfo<T::AccountId, T::Hash>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn election_request)]
	pub type CandidatesList<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CandidateRequestRaised { candidate: T::AccountId },
		ElectionStarted { election_info: ElectionInfo<T::AccountId> },
		CastVote { candidate: T::AccountId, response: Votes },
		Winner { election_hash: T::Hash, election_info: ElectionInfo<T::AccountId> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// If user try to raise the same request
		UserAlreadyPresent,
		/// If election has already started
		ElectionAlreadyStarted,
		/// If the request is already approved
		AlreadyApproved,
		///  If the election status if false
		InactiveElection,
		/// If user try to access the wrong election information
		InvalidElectionHash,
		/// If the user not present in the election candidate list
		InvalidCandidateChosen,
		/// if a user to tries to vote more than once
		DuplicateVoteNotAllowed,
		/// if wrong candidate chosen
		CandidateNotAvailable,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///! Any user can raise a request to join as a candidate
		#[pallet::call_index(0)]
		#[pallet::weight({10_000})]
		pub fn raise_join_request_as_candidate(origin: OriginFor<T>) -> DispatchResult {
			// check the origin should be signed
			let who = ensure_signed(origin)?;

			// Check for duplicate
			let mut all_candidates = CandidatesList::<T>::get();
			ensure!(!all_candidates.contains(&who), Error::<T>::UserAlreadyPresent);

			all_candidates.push(who.clone());
			CandidatesList::<T>::put(all_candidates);

			Self::deposit_event(Event::<T>::CandidateRequestRaised { candidate: who });

			Ok(())
		}

		///! Only Sudo can perform this task
		///! Sudo adds the valid candidates in the candidate list
		///! Sudo change the status of the election. (inactive -> active)
		#[pallet::call_index(2)]
		#[pallet::weight({10_000})]
		pub fn start_election(
			origin: OriginFor<T>,
			election_hash: T::Hash,
			election: ElectionInfo<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let is_available = Election::<T>::contains_key(election_hash);

			ensure!(!is_available, Error::<T>::ElectionAlreadyStarted);

			Election::<T>::insert(election_hash, &election);

			Self::deposit_event(Event::<T>::ElectionStarted { election_info: election });

			Ok(())
		}

		///! Any user can choose any valid candidate and cast the vote
		/// Assuming all the candidates should have at least 1 vote
		///! Duplicate votes are not allowed
		#[pallet::call_index(4)]
		#[pallet::weight({10_000})]
		pub fn cast_vote(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			election_hash: T::Hash,
			vote: Votes,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// update the election storage
			let mut election =
				Election::<T>::get(election_hash).ok_or(Error::<T>::InvalidElectionHash)?;

			ensure!(
				election.all_candidates.contains(&candidate),
				Error::<T>::InvalidCandidateChosen
			);
			// check the status of the current election
			// todo!()
			ensure!(election.status,Error::<T>::InactiveElection);

			ensure!(!election.voters_list.contains(&who), Error::<T>::DuplicateVoteNotAllowed);

			election.voters_list.push(who.clone());

			Election::<T>::insert(election_hash, election.clone());

			// update the candidate storage.
			let already_initiated = Candidate::<T>::contains_key(&candidate);

			if already_initiated {
				let mut candidate_detail =
					Candidate::<T>::get(&candidate).ok_or(Error::<T>::InvalidCandidateChosen)?;

				//Cast vote and update the storage
				// todo!()
				if vote == Votes::Yes{
					candidate_detail.total_aye_votes.push(who.clone());
				}else{
					candidate_detail.total_naye_votes.push(who.clone());
				}

				// update the current vote
				// todo!()
				Candidate::<T>::insert(&candidate,candidate_detail.clone());

			} else {
				// Initiate the new candidate info
				// todo!()
				let new_candidate=CandidateInfo{
					election_hash:election_hash.clone(),total_aye_votes:if vote == Votes::Yes{vec![who.clone()]} else {
						vec![]
					},
					total_naye_votes:if vote == Votes::No{
						vec![who.clone()]
					}else{
						vec![]
					},
				};

				//Update the current vote and storage
				// todo!()
				Candidate::<T>::insert(&candidate,new_candidate.clone());
			}

			Self::deposit_event(Event::<T>::CastVote { candidate, response: vote });
			Ok(())
		}

		///! Only sudo can perform this task
		///! when election stops then the winner will be announced.
		/// Assumption:
		/// 1. As we are iterating over the vector of candidates so if two candidates have common no of votes then first user will be the winner.
		/// 2. All candidate have atleast one vote.
		#[pallet::call_index(5)]
		#[pallet::weight({10_000})]
		pub fn stop_election(origin: OriginFor<T>, election_hash: T::Hash) -> DispatchResult {
			ensure_root(origin.clone())?;

			let mut election =
				Election::<T>::get(election_hash).ok_or(Error::<T>::InvalidElectionHash)?;

			ensure!(election.status, Error::<T>::InactiveElection);
			election.status = false;

			// calculate the winner...
			let all_candidates = &election.all_candidates;

			let mut vote_count = 0;
			for i in all_candidates {
				let vote_info = Candidate::<T>::get(i).ok_or(Error::<T>::CandidateNotAvailable)?;
				if vote_count < vote_info.total_aye_votes.len() {
					vote_count = vote_info.total_aye_votes.len();
					election.winner = Some(i.clone());
				}
			}

			Election::<T>::insert(election_hash, &election);

			Self::deposit_event(Event::<T>::Winner { election_hash, election_info: election });

			Ok(())
		}
	}
}
