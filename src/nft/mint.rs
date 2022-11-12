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
        // create a royalty map to store in the token
        let royalty = HashMap::new();

        //specify the token struct that contains the owner ID
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,
            //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
            royalty,
        };

        //insert the token ID and token struct and make sure that the token doesn't exist
        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token already exists"
        );

        self.tokens_by_id.insert(&token_id, &token);
        //insert the token ID and metadata
        self.token_metadata_by_id.insert(&token_id, &metadata);

        //call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                // Owner of the token.
                owner_id: token.owner_id.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
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
