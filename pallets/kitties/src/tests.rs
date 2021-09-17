use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::{OnFinalize, OnInitialize};

pub const KITTY_RESERVE: u128 = 1_000;
pub const NOBODY: u64 = 99;

fn run_to_block( n: u64) {
	while System::block_number() < n {
		KittiesModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number()+1);
		System::on_initialize(System::block_number());
		KittiesModule::on_initialize(System::block_number());
	}
}

#[test]
fn can_create_work() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(KittiesModule::create(Origin::signed(ALICE)));
        System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittyCreate(
            1, 0,
        )));
        assert_eq!(KittiesCount::<Test>::get(), 1);
        assert_eq!(Owner::<Test>::get(0), Some(ALICE));
        assert_eq!(Balances::reserved_balance(ALICE), KITTY_RESERVE);
    });
}

#[test]
fn can_create_faile_not_enough_money() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            KittiesModule::create(Origin::signed(NOBODY)),
            Error::<Test>::MoneyNotEnough
        );
    });
}

#[test]
fn can_transfer_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(ALICE)));
        assert_eq!(KittiesCount::<Test>::get(), 1);
        assert_eq!(Balances::reserved_balance(1), KITTY_RESERVE);
        assert_ok!(KittiesModule::transfer(Origin::signed(1), BOB, 0));
        System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittyTransfer(
            1, 2, 0,
        )));
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::reserved_balance(2), KITTY_RESERVE);
    });
}

#[test]
fn can_transfer_faile_not_enough_money() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            KittiesModule::create(Origin::signed(NOBODY)),
            Error::<Test>::MoneyNotEnough
        );
    });
}

#[test]
fn can_transfer_failed_not_owner() {
    new_test_ext().execute_with(|| {
        assert_noop! {
            KittiesModule::transfer(Origin::signed(1),BOB,99),
            Error::<Test>::NotOwner
        }
    });
}

#[test]
fn can_transfer_failed_already_owned() {
    new_test_ext().execute_with(|| {
        assert_noop! {
            KittiesModule::transfer(Origin::signed(1),ALICE,0),
            Error::<Test>::AlreadyOwned
        }
    });
}

#[test]
fn can_bread_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1)));
        assert_ok!(KittiesModule::create(Origin::signed(1)));
        assert_eq!(KittiesCount::<Test>::get(), 2);
        assert_eq!(Owner::<Test>::get(0), Some(1));
        assert_eq!(Owner::<Test>::get(1), Some(1));
        assert_eq!(Balances::reserved_balance(1), 2 * KITTY_RESERVE);

        assert_ok!(KittiesModule::bread(Origin::signed(1), 0, 1));
        System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittyCreate(
            1, 2,
        )));
        assert_eq!(KittiesCount::<Test>::get(), 3);
        assert_eq!(Owner::<Test>::get(2), Some(1));
        assert_eq!(Balances::reserved_balance(1), 3 * KITTY_RESERVE);
    });
}

#[test]
fn can_bread_failed_invalid_same_parent_index() {
    new_test_ext().execute_with(|| {
        assert_noop! {
            KittiesModule::bread(Origin::signed(1),1,1),
            Error::<Test>::SameParentIndex
        }
    });
}

#[test]
fn can_bread_failed_invalid_kittyindex() {
    new_test_ext().execute_with(|| {
        assert_noop! {
            KittiesModule::bread(Origin::signed(1),0,1),
            Error::<Test>::InvalidKittyIndex
        }
    });
}

#[test]
fn can_sale_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1)));
        assert_ok!(KittiesModule::sale(Origin::signed(1), 0, Some(5_000)));
        System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittyForSale(
            1,
            0,
            Some(5_000),
        )));
    });
}

#[test]
fn can_sale_failed_not_owner() {
    new_test_ext().execute_with(|| {
        assert_noop! {
            KittiesModule::sale(Origin::signed(1),0,Some(5_000)),
            Error::<Test>::NotOwner
        }
    });
}

#[test]
fn can_buy_failed_not_owner() {
    new_test_ext().execute_with(|| {
        assert_noop! {
            KittiesModule::buy(Origin::signed(1),99),
            Error::<Test>::NotOwner
        }
    });
}

#[test]
fn can_buy_failed_not_for_sale() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1)));
        assert_noop! {
            KittiesModule::buy(Origin::signed(2),0),
            Error::<Test>::NotForSale
        }
    });
}

#[test]
fn can_buy_failed_already_owned() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1)));
        assert_ok!(KittiesModule::sale(Origin::signed(1), 0, Some(5_000)));
        assert_noop! {
            KittiesModule::buy(Origin::signed(1),0),
            Error::<Test>::AlreadyOwned
        }
    });
}

#[test]
fn can_buy_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(KittiesModule::create(Origin::signed(1)));
        assert_ok!(KittiesModule::sale(Origin::signed(1), 0, Some(8_000)));
        assert_eq!(Owner::<Test>::get(0), Some(1));
        assert_eq!(KittyPrices::<Test>::get(0), Some(8_000));

        assert_ok!(KittiesModule::buy(Origin::signed(2), 0));
        //检查事件
        System::assert_last_event(mock::Event::KittiesModule(crate::Event::KittySaleOut(
            2,
            0,
            Some(8_000),
        )));

        assert_eq!(Balances::free_balance(1), 10_000 + 8_000);
        assert_eq!(
            Balances::free_balance(2),
            20_000 - 8_000 - KITTY_RESERVE
        );
        assert_eq!(Balances::reserved_balance(1), 0);
        assert_eq!(Balances::reserved_balance(2), KITTY_RESERVE);

        assert_eq!(Owner::<Test>::get(0), Some(2));
        assert_eq!(KittyPrices::<Test>::get(0), None);
    });
}
