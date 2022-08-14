use near_contract_standards::storage_management::StorageManagement;
use near_sdk::{Balance, near_bindgen, Promise};
use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};

#[derive(BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct VAccount {
    pub deposit: Balance,
    pub used: Balance
}

pub const STORAGE_PER_MAIL: Balance = 10;
pub const STORAGE_PER_ACCOUNT: Balance = 20;

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance {
        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());
        let registration_only = registration_only.unwrap_or(false);
        let amount = env::attached_deposit();
        let is_registered = self.accounts.contains_key(&account_id);

        if is_registered {
            let mut current_account = self.accounts.get(&account_id).unwrap();
            let new_amount = amount + current_account.deposit;
            current_account.deposit = new_amount;
            self.accounts.insert(&account_id, &current_account);
            return StorageBalance {
                total: U128(new_amount),
                available: U128(new_amount - current_account.used)
            }
        } else {
            let used = min_deposit();
            assert!(amount > used, "Amount deposit must be greater than min deposit");
            if registration_only {
                let refund = amount - used;
                Promise::new(env::predecessor_account_id()).transfer(refund);
                let new_account  = VAccount {deposit: used, used };
                self.accounts.insert(&account_id, &new_account);
                return StorageBalance {
                    total: U128(used),
                    available: U128(0)
                }
            } else {
                let new_account  = VAccount {deposit: amount, used};
                self.accounts.insert(&account_id, &new_account);
                return StorageBalance {
                    total: U128(amount),
                    available: U128(amount - used)
                }
            }
        }
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        let account_id  = env::predecessor_account_id();
        let real_amount = amount.unwrap().0;
        assert!(self.accounts.contains_key(&account_id), "account not registered");
        let mut vaccount = self.accounts.get(&account_id).unwrap();
        assert!((vaccount.deposit- vaccount.used) >= (real_amount), "Withraw too much");
        Promise::new(env::predecessor_account_id()).transfer(real_amount);
        vaccount.deposit = vaccount.deposit- real_amount;
        self.accounts.insert(&account_id, &vaccount);
        StorageBalance { 
            total: U128(vaccount.deposit),
            available: U128(vaccount.deposit-vaccount.used)
        }
    }

    #[allow(unused_variables)]
    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        let account_id  = env::predecessor_account_id();
        if let Some(vaccount) = self.accounts.get(&account_id) {
            Promise::new(env::predecessor_account_id()).transfer(vaccount.deposit);
            self.accounts.remove(&account_id);
            return true;
        } else {
            return false;
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let min_deposit = min_deposit();
        StorageBalanceBounds { 
            min: U128(min_deposit),
            max: None
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        if let Some(vaccount) = self.accounts.get(&account_id) {
            return Some(StorageBalance {
                total: U128(vaccount.deposit),
                available: U128(vaccount.deposit - vaccount.used)
            })
        } 
        return None;
    }
}

pub (crate) fn min_deposit() -> Balance {
    return env::storage_byte_cost() * STORAGE_PER_ACCOUNT;
}