// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Traits and associated utilities for scheduling dispatchables in FRAME.

pub use chrono_light::prelude::*;
use codec::{Codec, Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::{fmt::Debug, prelude::*, result::Result};

/// Priority with which a call is scheduled. It's just a linear amount with lowest values meaning
/// higher priority.
pub type Priority = u8;

/// Anything of this value or lower will definitely be scheduled on the block that they ask for,
/// even if it breaches the `MaximumWeight` limitation.
pub const HARD_DEADLINE: Priority = 63;

/// Type representing an encodable value or the hash of the encoding of such a value.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MaybeHashed<T, Hash> {
    /// The value itself.
    Value(T),
    /// The hash of the encoded value which this value represents.
    Hash(Hash),
}

impl<T, H> From<T> for MaybeHashed<T, H> {
    fn from(t: T) -> Self {
        MaybeHashed::Value(t)
    }
}

/// Error type for `MaybeHashed::lookup`.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum LookupError {
    /// A call of this hash was not known.
    Unknown,
    /// The preimage for this hash was known but could not be decoded into a `Call`.
    BadFormat,
}

impl<T: Decode, H> MaybeHashed<T, H> {
    pub fn as_value(&self) -> Option<&T> {
        match &self {
            Self::Value(c) => Some(c),
            Self::Hash(_) => None,
        }
    }

    pub fn as_hash(&self) -> Option<&H> {
        match &self {
            Self::Value(_) => None,
            Self::Hash(h) => Some(h),
        }
    }

    pub fn ensure_requested<P: PreimageProvider<H>>(&self) {
        match &self {
            Self::Value(_) => (),
            Self::Hash(hash) => P::request_preimage(hash),
        }
    }

    pub fn ensure_unrequested<P: PreimageProvider<H>>(&self) {
        match &self {
            Self::Value(_) => (),
            Self::Hash(hash) => P::unrequest_preimage(hash),
        }
    }

    pub fn resolved<P: PreimageProvider<H>>(self) -> (Self, Option<H>) {
        match self {
            Self::Value(c) => (Self::Value(c), None),
            Self::Hash(h) => match P::get_preimage(&h) {
                Some(data) => match T::decode(&mut &data[..]) {
                    Ok(c) => (Self::Value(c), Some(h)),
                    Err(_) => (Self::Hash(h), None),
                },
                None => (Self::Hash(h), None),
            },
        }
    }
}

/// A type that can be used as a scheduler.
pub trait Anon<BlockNumber, Call, Origin> {
    /// An address which can be used for removing a scheduled task.
    type Address: Codec + Clone + Eq + EncodeLike + Debug + TypeInfo;
    /// A means of expressing a call by the hash of its encoded data.
    type Hash;

    /// Schedule a dispatch to happen at the beginning of some block in the future.
    ///
    /// This is not named.
    fn schedule(
        schedule: Schedule,
        priority: Priority,
        origin: Origin,
        call: MaybeHashed<Call, Self::Hash>,
    ) -> Result<Self::Address, DispatchError>;

    /// Cancel a scheduled task. If periodic, then it will cancel all further instances of that,
    /// also.
    ///
    /// Will return an error if the `address` is invalid.
    ///
    /// NOTE: This guaranteed to work only *before* the point that it is due to be executed.
    /// If it ends up being delayed beyond the point of execution, then it cannot be cancelled.
    ///
    /// NOTE2: This will not work to cancel periodic tasks after their initial execution. For
    /// that, you must name the task explicitly using the `Named` trait.
    fn cancel(address: Self::Address) -> Result<(), ()>;

    /// Reschedule a task. For one-off tasks, this dispatch is guaranteed to succeed
    /// only if it is executed *before* the currently scheduled block. For periodic tasks,
    /// this dispatch is guaranteed to succeed only before the *initial* execution; for
    /// others, use `reschedule_named`.
    ///
    /// Will return an error if the `address` is invalid.
    fn reschedule(
        address: Self::Address,
        new_schedule: Schedule,
    ) -> Result<Self::Address, DispatchError>;

    /// Return the next dispatch time for a given task.
    ///
    /// Will return an error if the `address` is invalid.
    fn next_dispatch_time(address: Self::Address) -> Result<BlockNumber, ()>;
}

/// A type that can be used as a scheduler.
pub trait Named<BlockNumber, Call, Origin> {
    /// An address which can be used for removing a scheduled task.
    type Address: Codec + Clone + Eq + EncodeLike + sp_std::fmt::Debug;
    /// A means of expressing a call by the hash of its encoded data.
    type Hash;

    /// Schedule a dispatch to happen at the beginning of some block in the future.
    ///
    /// - `id`: The identity of the task. This must be unique and will return an error if not.
    fn schedule_named(
        id: Vec<u8>,
        schedule: Schedule,
        priority: Priority,
        origin: Origin,
        call: MaybeHashed<Call, Self::Hash>,
    ) -> Result<Self::Address, ()>;

    /// Cancel a scheduled, named task. If periodic, then it will cancel all further instances
    /// of that, also.
    ///
    /// Will return an error if the `id` is invalid.
    ///
    /// NOTE: This guaranteed to work only *before* the point that it is due to be executed.
    /// If it ends up being delayed beyond the point of execution, then it cannot be cancelled.
    fn cancel_named(id: Vec<u8>) -> Result<(), ()>;

    /// Reschedule a task. For one-off tasks, this dispatch is guaranteed to succeed
    /// only if it is executed *before* the currently scheduled block.
    fn reschedule_named(
        id: Vec<u8>,
        new_schedule: Schedule,
    ) -> Result<Self::Address, DispatchError>;

    /// Return the next dispatch time for a given task.
    ///
    /// Will return an error if the `id` is invalid.
    fn next_dispatch_time(id: Vec<u8>) -> Result<BlockNumber, ()>;
}

use frame_support::traits::PreimageProvider;
//use super::PreimageProvider;
