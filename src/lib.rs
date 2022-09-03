use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, LookupMap, UnorderedSet};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, BorshStorageKey, json_types::U128, Promise };
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement};
use email::*;
use storage_impl::*;
use legacy::*;

mod email;
mod storage_impl;
mod legacy;
pub type EmailID = u128;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Sender,
    Receiver,
    Email,
    SenderMail { email_id : EmailID},
    ReceiverMail { email_id : EmailID},
    Account
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub senders: LookupMap<AccountId, UnorderedSet<EmailID>>,
    pub receivers: LookupMap<AccountId, UnorderedSet<EmailID>>,
    pub emails: UnorderedMap<EmailID, UpgradeableEmail>,
    pub email_count: u128,
    pub accounts: LookupMap<AccountId, VAccount>,
    pub donation_count: u128,
    pub donation_account: AccountId
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self { 
            senders: LookupMap::new(StorageKeys::Sender),
            receivers: LookupMap::new(StorageKeys::Receiver),
            emails: UnorderedMap::new(StorageKeys::Email),
            email_count: 0,
            accounts: LookupMap::new(StorageKeys::Account),
            donation_count: 0,
            donation_account: AccountId::new_unchecked("dev-1662036494830-25321560561944".to_string())
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let contract_v1: ContractV1 = env::state_read().expect("Cannot read state");
        Contract { 
            senders: contract_v1.senders,
            receivers: contract_v1.receivers,
            emails: contract_v1.emails,
            email_count: contract_v1.email_count,
            accounts: contract_v1.accounts,
            donation_count: 0,
            donation_account: AccountId::new_unchecked("dev-1662036494830-25321560561944".to_string())
        }
    }

    #[payable]
    pub fn send_mail(&mut self, receiver: AccountId, title: String, content: String) -> Promise {
        let sender = env::predecessor_account_id();
        assert!(self.accounts.contains_key(&sender), "Account not registered");
        assert!(self.can_send_mail(sender.clone()), "Not deposit enough");

        let mut vaccount = self.accounts.get(&sender).unwrap();
        vaccount.used += STORAGE_PER_MAIL * env::storage_byte_cost();
        self.accounts.insert(&sender, &vaccount);
        let current_count = self.email_count;
        self.email_count = self.email_count +1;
        let timestamp =env::block_timestamp();
        let fee = U128(env::attached_deposit());
        let email = Email {
            title,
            content,
            timestamp,
            fee
        };
        let upgradeable_email = UpgradeableEmail::from(email);
        self.emails.insert(&current_count, &upgradeable_email);
        if let Some(mut sender_vec) = self.senders.get(&sender) {
            sender_vec.insert(&current_count);
            self.senders.insert(&sender, &sender_vec);
        } else {
            let mut sender_vec_new = UnorderedSet::new(StorageKeys::SenderMail { email_id: current_count });
            sender_vec_new.insert(&current_count);
            self.senders.insert(&sender, &sender_vec_new);
        }

        if let Some(mut receiver_vec) = self.receivers.get(&receiver) {
            receiver_vec.insert(&current_count);
            self.receivers.insert(&receiver, &receiver_vec);
        } else {
            let mut receiver_vec_new = UnorderedSet::new(StorageKeys::ReceiverMail  { email_id: current_count });
            receiver_vec_new.insert(&current_count);
            self.receivers.insert(&receiver, &receiver_vec_new);
        }
        Promise::new(receiver).transfer(fee.0)
    }

    pub fn get_email(&self, email_id: U128) -> Email {
        let real_email_id: EmailID = email_id.0;
        self.emails.get(&real_email_id).unwrap().into_current()
    }

    pub fn delete_mail(&mut self, email_id: U128) {
        let real_email_id: EmailID = email_id.0;
        let sender = env::predecessor_account_id();
        assert!(!self.senders.get(&sender).unwrap().contains(&real_email_id), "Caller is not sender");
        self.emails.remove(&real_email_id);
    }

    pub fn mail_exist(&self) -> u64 {
        self.emails.keys_as_vector().len()
    }

    pub fn get_mail_receive(&self, receiver: AccountId) -> Vec<Email> {
        let mut email_vec:Vec<Email> = Vec::new();
        if let Some(receiver_vec) = self.receivers.get(&receiver) {
            for index in receiver_vec.iter() {
                let mail = self.emails.get(&index).unwrap().into_current();
                email_vec.push(mail);
            }
        }
        return email_vec;
    }

    pub fn get_mail_send(&self, sender: AccountId) -> Vec<Email> {
        let mut email_vec:Vec<Email> = Vec::new();
        if let Some(sender_vec) = self.senders.get(&sender) {
            for index in sender_vec.iter() {
                let mail = self.emails.get(&index).unwrap().into_current();
                email_vec.push(mail);
            }
        }
        return email_vec;
    }

    pub fn get_mail_receive_num(&self, receiver: AccountId) -> u64 {
        if let Some(receiver_vec) = self.receivers.get(&receiver) {
           return receiver_vec.len();
        }
        0
    }

    pub fn get_mail_send_num(&self, sender: AccountId) -> u64 {
        if let Some(sender_vec) = self.senders.get(&sender) {
           return sender_vec.len();
        }
        0
    }

    pub fn mail_delete(&self) -> U128 {
        let mail_exist:u128 = self.emails.keys_as_vector().len().into();
        U128(self.email_count - mail_exist)
    }
}

impl Contract {
    pub fn can_send_mail(&self, account_id: AccountId) -> bool {
        let available_storage = self.storage_balance_of(account_id.clone()).unwrap().available.0;

        if let Some(sender) = self.senders.get(&account_id) {
            let mail_len: u128 = (sender.len() +1).into();
            return available_storage > (mail_len * STORAGE_PER_MAIL * env::storage_byte_cost());
        }
        return available_storage > (STORAGE_PER_MAIL * env::storage_byte_cost());
    }
}