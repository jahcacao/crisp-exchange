use crate::*;
use near_contract_standards::non_fungible_token::core::NonFungibleTokenResolver;
use near_sdk::{ext_contract, Gas, PromiseResult};

use super::{
    events::{EventLog, EventLogVariant, NftTransferLog},
    internal::{assert_one_yocto, refund_approved_account_ids},
    metadata::{JsonToken, TokenId},
};

const GAS_FOR_NFT_ON_TRANSFER: Gas = 25_000_000_000_000;

pub trait NonFungibleTokenCore {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );

    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool>;

    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
}

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
    fn nft_resolve_transfer(
        &mut self,
        authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,
    ) -> bool;
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let previous_token =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);
        refund_approved_account_ids(
            previous_token.owner_id.clone(),
            &previous_token.approved_account_ids,
        );
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let previous_token = self.internal_transfer(
            &sender_id,
            &receiver_id,
            &token_id,
            approval_id,
            memo.clone(),
        );
        let mut authorized_id = None;
        if sender_id != previous_token.owner_id {
            authorized_id = Some(sender_id.to_string());
        }
        ext_non_fungible_token_receiver::nft_on_transfer(
            sender_id,
            previous_token.owner_id.clone(),
            token_id.clone(),
            msg,
            &receiver_id,
            0,
            GAS_FOR_NFT_ON_TRANSFER,
        )
        .then(ext_self::nft_resolve_transfer(
            authorized_id,
            previous_token.owner_id,
            receiver_id.clone(),
            token_id,
            previous_token.approved_account_ids,
            memo,
            &receiver_id,
            0,
            GAS_FOR_NFT_ON_TRANSFER,
        ))
        .into()
    }

    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        if let Some(token) = self.tokens_by_id.get(&token_id) {
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            Some(JsonToken {
                token_id,
                owner_id: token.owner_id,
                metadata,
                approved_account_ids: token.approved_account_ids,
                royalty: token.royalty,
            })
        } else {
            None
        }
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        // authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        _approved_account_ids: Option<HashMap<AccountId, u64>>,
        // memo: Option<String>,
    ) -> bool {
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            if let Ok(return_token) = near_sdk::serde_json::from_slice::<bool>(&value) {
                if !return_token {
                    // refund_approved_account_ids(owner_id, &approved_account_ids);
                    return true;
                }
            }
        }
        let mut token = if let Some(token) = self.tokens_by_id.get(&token_id) {
            if token.owner_id != receiver_id {
                // refund_approved_account_ids(owner_id, &approved_account_ids);
                return true;
            }
            token
        } else {
            // refund_approved_account_ids(owner_id, &approved_account_ids);
            return true;
        };
        self.internal_remove_token_from_owner(&receiver_id.clone(), &token_id);
        self.internal_add_token_to_owner(&owner_id, &token_id);
        token.owner_id = owner_id.clone();
        refund_approved_account_ids(receiver_id.clone(), &token.approved_account_ids);
        // token.approved_account_ids = approved_account_ids;
        self.tokens_by_id.insert(&token_id, &token);
        let nft_transfer_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                authorized_id: None,
                old_owner_id: receiver_id.to_string(),
                new_owner_id: owner_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo: None,
            }]),
        };
        env::log(nft_transfer_log.to_string().as_bytes());
        false
    }
}
