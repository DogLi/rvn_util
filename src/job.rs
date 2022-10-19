use crate::op_data::OpData;
use byteorder::{BigEndian, ByteOrder};

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
    pub timestamp: u32,
}

pub fn nonce(miner_index: u64, job_id: u32) -> String {
    let mut nonce = [0; 12];
    BigEndian::write_u32(&mut nonce[0..4], job_id);
    BigEndian::write_u64(&mut nonce[4..], miner_index);
    hex::encode(nonce)
}

pub fn job_id_from_nonce(nonce: &str) -> u32 {
    let nonce = hex::decode(nonce).unwrap();
    BigEndian::read_u32(&nonce[0..4])
}

impl JobInfo {
    pub fn to_resp_str(&self, job_id: u32, miner_id: u64) -> String {
        let nonce_hex = nonce(miner_id, job_id);
        format!(
            "{{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\"]}}",
            nonce_hex,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_job_id() {
        let a = job_id();
        println!("id 1: {}", a);
        println!("{:?}", hex::decode(&a).unwrap());
    }
}
