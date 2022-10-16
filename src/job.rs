use crate::op_data::OpData;

/// 矿机任务所需的信息
#[derive(Debug, Clone)]
pub struct JobInfo {
    pub header_hash: [u8; 32],
    pub seed_hash: String,
    pub share_target_hex: String,
    pub block_target_hex: String,
    pub height: u32,
    pub block_bits_hex: String,
    pub refresh: bool,
    pub header: Vec<u8>,
    pub external_txs: Vec<String>,
    pub coinbase_tx: Vec<u8>,
}

impl JobInfo {
    pub fn to_resp_str(&self, id: &str) -> String {
        format!(
            "{{\"params\": [\"{}\", \"{}\", \"{}\", \"{}\", {}, \"{}\", \"{}\"], \"id\": null, \"method\": \"mining.notify\"}}",
            id,
            hex::encode(self.header_hash),
            self.seed_hash,
            self.share_target_hex,
            self.refresh,
            self.height,
            self.block_bits_hex,
        )
    }

    pub fn build_block(&self, nonce: &str, mix_hash: &str) -> String {
        let op_data = OpData::default().var_push_num(self.external_txs.len() as u64 + 1);
        format!(
            "{}{}{}{}{}{}",
            hex::encode(&self.header),
            nonce,
            mix_hash,
            hex::encode(op_data.as_slice()),
            hex::encode(&self.coinbase_tx),
            self.external_txs.join(",")
        )
    }
}
