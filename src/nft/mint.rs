use crate::{
    nft::{
        events::{EventLog, EventLogVariant, NftMintLog},
        metadata::{Token, TokenId, TokenMetadata},
    },
    *,
};

#[near_bindgen]
impl Contract {
    #[private]
    pub fn nft_mint(&mut self, token_id: TokenId, receiver_id: AccountId, metadata: TokenMetadata) {
        let royalty = HashMap::new();
        let token = Token {
            owner_id: receiver_id,
            approved_account_ids: Default::default(),
            next_approval_id: 0,
            royalty,
        };
        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token already exists"
        );
        self.tokens_by_id.insert(&token_id, &token);
        self.token_metadata_by_id.insert(&token_id, &metadata);
        self.internal_add_token_to_owner(&token.owner_id, &token_id);
        let nft_mint_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: token.owner_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo: None,
            }]),
        };
        env::log(&nft_mint_log.to_string().as_bytes());
    }

    #[private]
    #[payable]
    pub fn nft_burn(&mut self, token_id: TokenId) {
        assert!(
            self.tokens_by_id.get(&token_id).is_some(),
            "Token does not exist exists"
        );
        let token = self.tokens_by_id.get(&token_id).unwrap();
        self.token_metadata_by_id.remove(&token_id);
        self.internal_remove_token_from_owner(&token.owner_id, &token_id);
    }
}
