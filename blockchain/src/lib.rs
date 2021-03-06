// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Shasper.

// Parity Shasper is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.

// Parity Shasper is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more
// details.

// You should have received a copy of the GNU General Public License along with
// Parity Shasper.  If not, see <http://www.gnu.org/licenses/>.
mod pool;
pub mod backend;
pub mod preset;

pub use pool::AttestationPool;
pub use shasper_runtime::{Block, StateExternalities};

use beacon::primitives::H256;
use beacon::types::*;
use beacon::{Error as BeaconError, BeaconState, BeaconExecutive, Config,
			 BLSConfig, Inherent, Transaction};
use std::sync::Arc;
use blockchain::{Block as BlockT, BlockExecutor, AsExternalities};
use lmd_ghost::JustifiableExecutor;
use core::marker::PhantomData;

use blockchain_rocksdb::RocksState as RocksStateT;

#[derive(Clone)]
pub struct MemoryState<C: Config> {
	state: BeaconState<C>,
}

impl<C: Config> From<BeaconState<C>> for MemoryState<C> {
	fn from(state: BeaconState<C>) -> Self {
		Self { state }
	}
}

impl<C: Config> Into<BeaconState<C>> for MemoryState<C> {
	fn into(self) -> BeaconState<C> {
		self.state
	}
}

impl<C: Config> StateExternalities for MemoryState<C> {
	type Config = C;

	fn state(&self) -> &BeaconState<C> {
		&self.state
	}

	fn state_mut(&mut self) -> &mut BeaconState<C> {
		&mut self.state
	}
}

impl<C: Config> AsExternalities<dyn StateExternalities<Config=C>> for MemoryState<C> {
	fn as_externalities(&mut self) -> &mut (dyn StateExternalities<Config=C> + 'static) {
		self
	}
}

#[derive(Clone)]
pub struct RocksState<C: Config> {
	state: BeaconState<C>,
}

impl<C: Config> From<BeaconState<C>> for RocksState<C> {
	fn from(state: BeaconState<C>) -> Self {
		Self { state }
	}
}

impl<C: Config> Into<BeaconState<C>> for RocksState<C> {
	fn into(self) -> BeaconState<C> {
		self.state
	}
}

impl<C: Config> StateExternalities for RocksState<C> {
	type Config = C;

	fn state(&self) -> &BeaconState<C> {
		&self.state
	}

	fn state_mut(&mut self) -> &mut BeaconState<C> {
		&mut self.state
	}
}

impl<C: Config> AsExternalities<dyn StateExternalities<Config=C>> for RocksState<C> {
	fn as_externalities(&mut self) -> &mut (dyn StateExternalities<Config=C> + 'static) {
		self
	}
}

impl<C: Config> RocksStateT for RocksState<C> {
	type Raw = BeaconState<C>;

	fn from_raw(state: BeaconState<C>, _db: Arc<::rocksdb::DB>) -> Self {
		Self { state }
	}

	fn into_raw(self) -> BeaconState<C> {
		self.state
	}
}

#[derive(Debug)]
pub enum Error {
	Beacon(BeaconError),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error { }

impl From<BeaconError> for Error {
	fn from(error: BeaconError) -> Error {
		Error::Beacon(error)
	}
}

#[derive(Clone)]
pub struct Executor<C: Config, BLS: BLSConfig> {
	_marker: PhantomData<(C, BLS)>,
}

impl<C: Config, BLS: BLSConfig> Executor<C, BLS> {
	pub fn new() -> Self {
		Self { _marker: PhantomData }
	}

	pub fn initialize_block(
		&self,
		state: &mut <Self as BlockExecutor>::Externalities,
		target_slot: u64,
	) -> Result<(), Error> {
		Ok(beacon::initialize_block::<C>(state.state_mut(), target_slot)?)
	}

	pub fn apply_inherent(
		&self,
		parent_block: &Block<C>,
		state: &mut <Self as BlockExecutor>::Externalities,
		inherent: Inherent,
	) -> Result<UnsealedBeaconBlock<C>, Error> {
		Ok(beacon::apply_inherent::<C, BLS>(&parent_block.0, state.state_mut(), inherent)?)
	}

	pub fn apply_extrinsic(
		&self,
		block: &mut UnsealedBeaconBlock<C>,
		state: &mut <Self as BlockExecutor>::Externalities,
		extrinsic: Transaction<C>,
	) -> Result<(), Error> {
		Ok(beacon::apply_transaction::<C, BLS>(block, state.state_mut(), extrinsic)?)
	}

	pub fn finalize_block(
		&self,
		block: &mut UnsealedBeaconBlock<C>,
		state: &mut <Self as BlockExecutor>::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::finalize_block::<C, BLS>(block, state.state_mut())?)
	}
}

impl<C: Config, BLS: BLSConfig> BlockExecutor for Executor<C, BLS> {
	type Error = Error;
	type Block = Block<C>;
	type Externalities = dyn StateExternalities<Config=C> + 'static;

	fn execute_block(
		&self,
		block: &Block<C>,
		state: &mut Self::Externalities,
	) -> Result<(), Error> {
		Ok(beacon::execute_block::<C, BLS>(&block.0, state.state_mut())?)
	}
}

impl<C: Config, BLS: BLSConfig> JustifiableExecutor for Executor<C, BLS> {
	type ValidatorIndex = u64;

	fn justified_active_validators(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Vec<Self::ValidatorIndex>, Self::Error> {
		let executive = BeaconExecutive::new(state.state_mut());
		Ok(executive.justified_active_validators())
	}

	fn justified_block_id(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Option<<Self::Block as BlockT>::Identifier>, Self::Error> {
		let justified_root = state.state().current_justified_checkpoint.root;
		if justified_root == H256::default() {
			Ok(None)
		} else {
			Ok(Some(justified_root))
		}
	}

	fn votes(
		&self,
		block: &Self::Block,
		state: &mut Self::Externalities,
	) -> Result<Vec<(Self::ValidatorIndex, <Self::Block as BlockT>::Identifier)>, Self::Error> {
		let executive = BeaconExecutive::new(state.state_mut());
		Ok(executive.block_vote_targets(&block.0)?)
	}
}
