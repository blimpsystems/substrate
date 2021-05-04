// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

//! Tests for Assets Freezer pallet.

use super::*;
use crate::{Error, mock::*};
use sp_runtime::TokenError;
use frame_support::{
	assert_ok,
	assert_noop,
	traits::{
		Currency, 
		fungibles::{
			Inspect,
			MutateHold,
			UnbalancedHold,
			Transfer,
		},
	},
};
use pallet_balances::Error as BalancesError;
use pallet_assets::Error as AssetsError;

fn last_event() -> mock::Event {
	frame_system::Pallet::<Test>::events().pop().expect("Event expected").event
}

#[test]
fn basic_minting_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 100));
		assert_eq!(AssetsFreezer::balance(0, &1), 100);
		assert_eq!(AssetsFreezer::total_issuance(0), 100);
	});
}

#[test]
fn hold_and_release_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 200));
		assert_eq!(AssetsFreezer::can_hold(0, &1, 100), true);
		assert_ok!(AssetsFreezer::hold(0, &1, 100));
		assert_eq!(AssetsFreezer::balance(0, &1), 200);
		assert_eq!(AssetsFreezer::balance_on_hold(0, &1), 100);
		assert_ok!(AssetsFreezer::release(0, &1, 50, true));
		assert_eq!(AssetsFreezer::balance_on_hold(0, &1), 50);
		assert_ok!(AssetsFreezer::release(0, &1, 50, true));
		assert_eq!(AssetsFreezer::balance_on_hold(0, &1), 0);
		assert_eq!(AssetsFreezer::balance(0, &1), 200);
	});
}

#[test]
fn decrease_and_destroy_held_asset_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 200));
		assert_eq!(AssetsFreezer::can_hold(0, &1, 100), true);
		assert_ok!(AssetsFreezer::hold(0, &1, 100));
		assert_eq!(AssetsFreezer::balance(0, &1), 200);
		assert_eq!(AssetsFreezer::balance_on_hold(0, &1), 100);
		assert_ok!(AssetsFreezer::decrease_balance_on_hold(0, &1, 50));
		assert_eq!(AssetsFreezer::balance_on_hold(0, &1), 50);
		assert_ok!(AssetsFreezer::release(0, &1, 50, true));
		assert_eq!(AssetsFreezer::balance_on_hold(0, &1), 0);
		assert_eq!(AssetsFreezer::balance(0, &1), 150);
	});
}

#[test]
fn transfer_held_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(Origin::signed(1), 0, 1, 200));
		assert_ok!(AssetsFreezer::hold(0, &1, 100));
		assert_eq!(AssetsFreezer::balance_on_hold(0, &1), 100);
		assert_noop!(AssetsFreezer::transfer(0, &1, &2, 150, WhenDust::Dispose), AssetsError::<Test>::BalanceLow);
		// Can't create the account with just a chunk of held balance - there needs to already be
		// the minimum deposit.
		assert_noop!(AssetsFreezer::transfer_held(0, &1, &2, 100, true), TokenError::CannotCreate);
		assert_ok!(Assets::mint(Origin::signed(1), 0, 2, 1));
		assert_noop!(AssetsFreezer::transfer_held(0, &1, &2, 150, true), TokenError::NoFunds);
		assert_eq!(AssetsFreezer::transfer_held(0, &1, &2, 100, true), Ok(100));
		assert_noop!(AssetsFreezer::transfer_held(0, &1, &2, 50, true), TokenError::NoFunds);
		assert_eq!(AssetsFreezer::balance(0, &1), 100);
		assert_eq!(AssetsFreezer::balance(0, &2), 101);
		assert_eq!(AssetsFreezer::balance_on_hold(0, &1), 0);
		assert_eq!(AssetsFreezer::balance_on_hold(0, &2), 100);
	});
}