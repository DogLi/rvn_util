use crate::op_data::OpData;
use rand::Rng;

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
    pub fn to_resp_str(&self) -> String {
        let id = job_id();
        format!(
            "{{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"{}\",\"{}\",\"{}\",\"{}\",{},\"{}\",\"{}\"]}}",
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


pub fn job_id() -> String {
    const CHARSET: &[u8] = b"abcdef0123456789";
    const  ID_LEN: usize = 12;
    let mut rng = rand::thread_rng();

    let id: String = (0..ID_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    id
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
