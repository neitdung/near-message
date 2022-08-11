use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, BorshStorageKey };

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Message {
    messages: UnorderedMap<AccountId, String>,
    records: UnorderedMap<AccountId, String>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Message,
    Record
}

#[near_bindgen]
impl Message {
    #[init]
    pub fn new() -> Self {
        Self {
            messages: UnorderedMap::new(StorageKeys::Message),
            records: UnorderedMap::new(StorageKeys::Record)
        }
    }

    pub fn set_message(&mut self, mess: String) {
        let account_id = env::signer_account_id();
        self.messages.insert(&account_id, &mess);
    }

    pub fn get_message(&self, account_id: AccountId) -> String {
        self.messages.get(&account_id).unwrap()
    }

    pub fn get_messages(&self, from_index: u64, limit: u64) -> Vec<(AccountId, String)> {
        let keys = self.messages.keys_as_vector();
        let values = self.messages.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| (keys.get(index).unwrap(), values.get(index).unwrap()))
            .collect()
    }
}