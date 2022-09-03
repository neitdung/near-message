use near_sdk::collections::{UnorderedMap, LookupMap, UnorderedSet};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::AccountId;
use near_sdk::json_types::U128;
use crate::email::UpgradeableEmail;
use crate::{Email, EmailID, VAccount};

#[derive(BorshSerialize, BorshDeserialize, Default, Clone)]
pub struct EmailV1 {
    pub title: String,
    pub content: String,
    pub timestamp: u64,
}

impl EmailV1 {
    pub fn into_current(&self) -> Email {
        Email {
            title: self.title.clone(),
            content: self.content.clone(),
            timestamp: self.timestamp,
            fee: U128(0)
        }
    }
}

#[derive(BorshDeserialize)]
pub struct ContractV1 {
    pub senders: LookupMap<AccountId, UnorderedSet<EmailID>>,
    pub receivers: LookupMap<AccountId, UnorderedSet<EmailID>>,
    pub emails: UnorderedMap<EmailID, UpgradeableEmail>,
    pub email_count: u128,
    pub accounts: LookupMap<AccountId, VAccount>
}