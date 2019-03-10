// Copyright 2018 Parity Technologies (UK) Ltd.
// This file is part of Substrate Shasper.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use primitives::H256;
use crate::eth1::{Eth1Data, Eth1DataVote};
use crate::attestation::{PendingAttestation, Crosslink};
use crate::validator::Validator;
use crate::block::BeaconBlockHeader;
use crate::consts::{
	SLOTS_PER_HISTORICAL_ROOT, LATEST_SLASHED_EXIT_LENGTH,
	LATEST_ACTIVE_INDEX_ROOTS_LENGTH, SHARD_COUNT,
	LATEST_RANDAO_MIXES_LENGTH,
};

pub struct BeaconState {
	// Misc
	pub slot: u64,
	pub genesis_time: u64,
	pub fork: Fork, // For versioning hard forks

	// Validator registry
	pub validator_registry: Vec<Validator>,
	pub validator_balances: Vec<u64>,
	pub validator_registry_update_epoch: u64,

	// Randomness and committees
	pub latest_randao_mixes: [H256; LATEST_RANDAO_MIXES_LENGTH],
	pub previous_shuffling_start_shard: u64,
	pub current_shuffling_start_shard: u64,
	pub previous_shuffling_epoch: u64,
	pub current_shuffling_epoch: u64,
	pub previous_shuffling_seed: H256,
	pub current_shuffling_seed: H256,

	// Finality
	pub previous_epoch_attestations: Vec<PendingAttestation>,
	pub current_epoch_attestations: Vec<PendingAttestation>,
	pub previous_justified_epoch: u64,
	pub justified_epoch: u64,
	pub justification_bitfield: u64,
	pub finalized_epoch: u64,

	// Recent state
	pub latest_crosslinks: [Crosslink; SHARD_COUNT],
	pub latest_block_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
	pub latest_state_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
	pub latest_active_index_roots: [H256; LATEST_ACTIVE_INDEX_ROOTS_LENGTH],
	pub latest_slashed_balances: [u64; LATEST_SLASHED_EXIT_LENGTH], // Balances slashed at every withdrawal period
	pub latest_block_header: BeaconBlockHeader,
	pub historical_roots: Vec<H256>,

	// Ethereum 1.0 chain data
	pub latest_eth1_data: Eth1Data,
	pub eth1_data_votes: Vec<Eth1DataVote>,
	pub deposit_index: u64,
}

pub struct HistoricalBatch {
	/// Block roots
	pub block_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
	/// State roots
	pub state_roots: [H256; SLOTS_PER_HISTORICAL_ROOT],
}

pub struct Fork {
	/// Previous fork version
	pub previous_version: u64,
	/// Current fork version
	pub current_version: u64,
	/// Fork epoch number
	pub epoch: u64,
}
