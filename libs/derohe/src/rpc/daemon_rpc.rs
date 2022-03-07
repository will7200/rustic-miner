use super::helpers;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetBlockTemplatePararms {
    #[serde(rename = "wallet_address")]
    pub Wallet_Address: String,
    #[serde(rename = "block")]
    pub Block: bool,
    #[serde(rename = "miner")]
    pub Miner: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetBlockTemplateResult {
    #[serde(rename = "jobid")]
    pub JobID: String,
    #[serde(default, rename = "blocktemplate_blob", deserialize_with = "helpers::null_to_default")]
    pub Blocktemplate_blob: String,
    #[serde(default, rename = "blockhashing_blob", deserialize_with = "helpers::null_to_default")]
    pub Blockhashing_blob: String,
    #[serde(rename = "difficulty")]
    pub Difficulty: String,
    #[serde(rename = "difficultyuint64")]
    pub Difficultyuint64: u64,
    #[serde(rename = "height")]
    pub Height: u64,
    #[serde(rename = "prev_hash")]
    pub Prev_Hash: String,
    #[serde(rename = "epochmilli")]
    pub EpochMilli: u64,
    #[serde(rename = "blocks")]
    pub Blocks: u64,
    #[serde(rename = "miniblocks")]
    pub MiniBlocks: u64,
    #[serde(rename = "lasterror")]
    pub LastError: String,
    #[serde(rename = "status")]
    pub Status: String,
}