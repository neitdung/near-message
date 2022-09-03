use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::U128;
use crate::EmailV1;
#[derive(BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Email {
    pub title: String,
    pub content: String,
    pub timestamp: u64,
    pub fee: U128,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum UpgradeableEmail {
    V1(EmailV1),
    Current(Email)
}

impl UpgradeableEmail {
    /// Upgrades from other versions to the currently used version.
    pub fn into_current(self) -> Email {
        match self {
            UpgradeableEmail::Current(email) => email,
            UpgradeableEmail::V1(email) => email.into_current(),
        }
    }
}

impl From<Email> for UpgradeableEmail {
    fn from(email: Email) -> Self {
        UpgradeableEmail::Current(email)
    }
}
