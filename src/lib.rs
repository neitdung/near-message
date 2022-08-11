use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, LookupMap, UnorderedSet};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, BorshStorageKey, assert_one_yocto, json_types::U128 };
use email::*;

mod email;
pub type EmailID = u128;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Sender,
    Receiver,
    Email,
    SenderMail { email_id : EmailID},
    ReceiverMail { email_id : EmailID}
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    senders: LookupMap<AccountId, UnorderedSet<EmailID>>,
    receivers: LookupMap<AccountId, UnorderedSet<EmailID>>,
    emails: UnorderedMap<EmailID, Email>,
    email_count: u128,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self { 
            senders: LookupMap::new(StorageKeys::Sender),
            receivers: LookupMap::new(StorageKeys::Receiver),
            emails: UnorderedMap::new(StorageKeys::Email),
            email_count: 0
        }
    }

    #[payable]
    pub fn send_mail(&mut self, receiver: AccountId, title: String, content: String) {
        assert_one_yocto();
        let current_count = self.email_count;
        self.email_count = self.email_count +1;
        let timestamp =env::block_timestamp();
        let email = Email {
            title,
            content,
            timestamp
        };
        self.emails.insert(&current_count, &email);
        let sender = env::predecessor_account_id();
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
    }

    pub fn get_email(&self, email_id: U128) -> Email {
        let real_email_id: EmailID = email_id.0;
        self.emails.get(&real_email_id).unwrap()
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
                let mail = self.emails.get(&index).unwrap();
                email_vec.push(mail);
            }
        }
        return email_vec;
    }

    pub fn get_mail_send(&self, sender: AccountId) -> Vec<Email> {
        let mut email_vec:Vec<Email> = Vec::new();
        if let Some(sender_vec) = self.senders.get(&sender) {
            for index in sender_vec.iter() {
                let mail = self.emails.get(&index).unwrap();
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