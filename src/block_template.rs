use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::address::Address;
use crate::job::JobInfo;
use crate::merkle::merkel_hash;
use crate::op_data::OpData;
use crate::script::Script;

/// RPC 返回的交易数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub data: String,
    pub txid: String,
    pub hash: String,
    pub fee: u64,
    pub sigops: u32,
    pub weight: u32,
    depends: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTemplateInfo {
    capabilities: Vec<String>,
    pub version: u32,
    rules: Vec<String>,
    vbavailable: HashMap<String, u32>,
    vbrequired: u32,
    pub previousblockhash: String,
    pub transactions: Vec<Transaction>,
    #[serde(rename = "coinbaseaux")]
    coinbase_aux: HashMap<String, String>,
    pub coinbasevalue: u64,
    #[serde(rename = "longpollid")]
    long_poll_id: String,
    pub target: String,
    pub mintime: u64,
    mutable: Vec<String>,
    #[serde(rename = "noncerange")]
    noncerange: String,
    #[serde(rename = "sigoplimit")]
    sigop_limit: u32,
    #[serde(rename = "sizelimit")]
    size_limit: u64,
    #[serde(rename = "weightlimit")]
    weight_limit: u64,
    #[serde(rename = "curtime")]
    pub cur_time: u64,
    pub bits: String,
    pub height: u32,
    pub default_witness_commitment: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct BlockTemplate {
    pub pool_addr: Address,
    pub pool_info: String,
    pub coinbase_tx: Vec<u8>,
    pub coinbase_txid: [u8; 32],
    pub seed_hash: [u8; 32],
    pub header: Vec<u8>,
    pub header_hash: [u8; 32],
    pub prev_hash: Vec<u8>,
    pub timestamp: u32,
    pub external_txs: Vec<String>,
    pub target_hex: String,
    pub bits_hex: String,
    pub witness_hex: String,
    pub version: u32,
    pub height: u32,
}

const KAWPOW_EPOCH_LENGTH: usize = 7500;

fn now() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

impl BlockTemplate {
    pub fn new(
        template_info: &BlockTemplateInfo,
        pool_addr: Address,
        pool_info: String,
    ) -> Result<Self> {
        let seed_hash = Self::seed_hash(template_info.height);
        let script = Script::coinbase_script(template_info.height, &pool_info)?;
        let coinbase_txin = Self::coinbase_txin(&script);
        let vout_to_miner = pool_addr.vout_to_miner();
        let witness_vout = hex::decode(&template_info.default_witness_commitment)?;

        // generate coinbase tx
        let coinbase_tx = OpData::default()
            .push_u32(1)
            .push_slice(&[0x00, 0x01, 0x01])
            .push_slice(&coinbase_txin)
            .push_u8(0x02)
            .push_u64(template_info.coinbasevalue)
            .op_push_slice(&vout_to_miner)
            .push_slice(&[0; 8])
            .op_push_slice(&witness_vout)
            .push_slice(&[0x01, 0x20])
            .push_slice(&[0; 32])
            .push_slice(&[0; 4]);

        // generate coinbase txid
        let coinbase_no_wit = OpData::default()
            .push_u32(1)
            .push_u8(0x01)
            .push_slice(&coinbase_txin)
            .push_u8(0x02)
            .push_u64(template_info.coinbasevalue)
            .op_push_slice(&vout_to_miner)
            .push_slice(&[0; 8])
            .op_push_slice(&witness_vout)
            .push_slice(&[0; 4]);

        let coinbase_txid = dsha256(coinbase_no_wit.as_slice());
        let mut txids = vec![coinbase_txid];
        let txids2: Vec<_> = template_info
            .transactions
            .iter()
            .map(|s| {
                let mut h = hex::decode(&s.txid).expect("invalid txid");
                h.reverse();
                h.try_into().unwrap()
            })
            .collect();
        txids.extend_from_slice(&txids2);
        let incoming_txs: Vec<_> = template_info
            .transactions
            .iter()
            .map(|s| s.data.clone())
            .collect();
        let merkle = merkel_hash(txids);

        // calculate header
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        let mut prev_hash = hex::decode(&template_info.previousblockhash).unwrap();
        prev_hash.reverse();
        let mut bits_hex = hex::decode(&template_info.bits).unwrap();
        bits_hex.reverse();
        let op_data_header = OpData::default()
            .push_u32(template_info.version)
            .push_slice(&prev_hash)
            .push_slice(&merkle)
            .push_u32(ts)
            .push_slice(&bits_hex)
            .push_u32(template_info.height);
        let header = op_data_header.as_slice().to_vec();
        let mut header_hash = dsha256(&header);
        header_hash.reverse();

        let obj = Self {
            pool_addr,
            pool_info,
            coinbase_tx: coinbase_tx.as_slice().to_vec(),
            witness_hex: template_info.default_witness_commitment.clone(),
            coinbase_txid,
            seed_hash,
            header,
            header_hash,
            prev_hash,
            timestamp: ts,
            external_txs: incoming_txs,
            target_hex: template_info.target.clone(),
            bits_hex: template_info.bits.clone(),
            version: template_info.version,
            height: template_info.height,
        };
        Ok(obj)
    }

    /// target_hex: like "00000001ffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    pub fn create_job(&self, target_hex: String, refresh: bool) -> JobInfo {
        JobInfo {
            header_hash: self.header_hash,
            seed_hash: hex::encode(self.seed_hash),
            share_target_hex: target_hex,
            block_target_hex: self.target_hex.clone(),
            height: self.height,
            block_bits_hex: self.bits_hex.clone(),
            refresh,
            header: self.header.clone(),
            external_txs: self.external_txs.clone(),
            coinbase_tx: self.coinbase_tx.clone(),
            timestamp: self.timestamp,
        }
    }

    fn coinbase_txin(script: &Script) -> Vec<u8> {
        let mut data = vec![0; 32];
        data.extend_from_slice(&[0xff; 4]);
        data.push(script.as_slice().len() as u8);
        data.extend_from_slice(script.as_slice());
        data.extend_from_slice(&[0xff; 4]);
        data
    }

    fn seed_hash(height: u32) -> [u8; 32] {
        let mut seed: [u8; 32] = Default::default();
        if height < KAWPOW_EPOCH_LENGTH as u32 {
            return seed;
        }
        for _ in 0..height / KAWPOW_EPOCH_LENGTH as u32 {
            let mut hasher = Keccak256::default();
            hasher.update(&mut seed);
            seed = hasher.finalize().to_vec().try_into().unwrap();
        }
        seed
    }

    pub fn is_new_template(&self, template_info: &BlockTemplateInfo) -> bool {
        self.height != template_info.height
            || now() - self.timestamp > 60
            || self.witness_hex != template_info.default_witness_commitment
    }
}

pub fn dsha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(&result);
    hasher.finalize().as_slice().try_into().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_dsha_256() {
        let data = b"hello world";
        let r = dsha256(&data.as_slice());
        let result_exp = [
            188_u8, 98, 212, 184, 13, 158, 54, 218, 41, 193, 108, 93, 77, 159, 17, 115, 31, 54, 5,
            44, 114, 64, 26, 118, 194, 60, 15, 181, 169, 183, 68, 35,
        ];
        assert_eq!(r, result_exp)
    }

    #[test]
    fn test_block_template() {
        let s = r#"{"capabilities": ["proposal"], "version": 805306368, "rules": ["assets", "messaging_restricted", "transfer_script", "enforce_value", "coinbase"], "vbavailable": {}, "vbrequired": 0, "previousblockhash": "0000000000003d02fdcce5f8e62741b431eb8677d878b96b41033ce436551f14", "transactions": [{"data": "02000000016c38ea64a528c14165bb70c1acf81c301a2d0ed7dcdef4aec58baafb78a29eee090000006a47304402206fbf6b12649facb89440a6870ffa4bfb81ee7ce274c5b4464b050d3b0333ffcf022070e5cc2f7d6b4a87c84c07546d767c3f080cc56a6b290212960eac0dc3333379012102761a92416b225d00f5ae5aec9e192ef1a8fd5c9887f75191b96f4d9e52d9cb95feffffff1db70bb83b000000001976a91406c6517f33ebf4518fecbdef2f011bcc74d5bdce88ac75b3ba3b000000001976a91406d298591456244b1747a2b15a8943c2b671181388acd92ecb3b000000001976a91409791086f0944a0314426252dca10081193ebffa88ac662dca3b000000001976a9140db2ac258e490441ad2f4405969cc33a908f0b7d88ac5dcbc33b000000001976a9141996438233f461a58f63468063c25a2c4f01233188ac1195d73b000000001976a9141c3a341245e08fc5283476b95bf0af7494b9821a88ac2bd3493c000000001976a9141d5d0f766bba2916f6dc174a224b6503b8a010b488ace5b5c33b000000001976a91424ec399e1f8311ade603e79896a37ed11a3cde6388ac7004e53c000000001976a9142839c671e5a0e83df3c930181ddfe9b0d178cd7a88ace1fc023c000000001976a9142d6a61657f16e9de1baed3eef730484ccd0853a788ac40013e3c000000001976a91431062a27fb63bdc7a788da233f2914425ca70b2c88ac8c64e43b000000001976a914587b5af7272f890c01e1477346f5a94ce88294c488ac5534ce3b000000001976a9146f144df83b2b6260fd540e13b00e1a003541ef7488ac2b87d053000000001976a91480e7aac2065f7527778a584e895f33609f96c11888ac3fc8c13b000000001976a914a1c918588f2ef2e3577d9b9aa4a3b1f610957c3888ac179cd23b000000001976a914a4a696074b20c80c4a3bf12230cd3d6c12f0c93c88aced1c334a000000001976a914a4ef18ccf68154b8f287a9aac84f553e505ad09888acaca4063c000000001976a914a6da027d5f3b92759eaaceb97dde0b4c83b7c95688acbe20f94a000000001976a914aa330dbd34ad84a301dd09406ad89351b151c19988ac5eac7258000000001976a914d12e236654887b67cf4a3c9282df4bf9af7be24388acd44bac3b000000001976a914da92550faa339ef299df238654634b4ccb698a8a88ac0cd0a13b000000001976a914e027e1dc3a412eec7ce177bfa61304dcf10168e888ac8a9eaa3b000000001976a914e148329f72abb0e6acca0b135cd296cbd012a74c88ac87ddc93b000000001976a914e2a3e4a71a191f7841d6ec162f05e4a77432994088ac37f693740b0000001976a914496fe43cb89975bd9f23e7b2d0aac39f9fd3b8fd88ac817c053c000000001976a914e8bbb6e50ffe887b36ba71aca2099c2589cfad4a88ac526f3a2a010000001976a914fc74e7b98d38e183ec1daa88043f52b611ce689a88acff96d43b000000001976a914fd4f0bcc822679e48b651cbf7c8dec6470d6fccc88ac6766f03b0000000017a914ad974e8859312c1afbe828aadfaa2e1e8f8d690087ca042600", "txid": "784f313ab617c14e08139f0e4257304eda8a82b6d1ed142d0d5d02d8d9772fde", "hash": "784f313ab617c14e08139f0e4257304eda8a82b6d1ed142d0d5d02d8d9772fde", "depends": [], "fee": 1158686, "sigops": 112, "weight": 4564}, {"data": "0200000006055872f60b25de3d30f6490d4509e339cf91899cc5b7f0b33a61fbf5f4563562270400006b4830450221008127dbdab649acec14a2f925d7130723171394d66f2048f1f2d41974c5f05e7202201fd302d6a95343449a397640bc24f8164979432112357cace32bd2de5db8c5950121038be5cc928bf73d3895d28f61ae4fe70863e4ba16f7f1d255edde5908d2c82d35feffffff17507bb936a3c6e674438139c20106ff9c452e742f0ad2297706a4642cc2b0990e0500006b483045022100d62134a0aa4f57d53ab63fda6fbe36e39ff5992a9fdc034e812e940ba7ebc2ad02207d57cf5deef5a1f44ea683d672a5f6a9a72bcf1db34a4a090dcb81d0c38e67530121038be5cc928bf73d3895d28f61ae4fe70863e4ba16f7f1d255edde5908d2c82d35feffffff3a664aecc01ac73fae20304b41bde95213f26c285b3d69a6bd7cd12840d269c6010000006b483045022100ffcd44525273d4c26db881ff514a10b0f77690c9bb6f941d71015f13a857ee7102203c23c9a815e90470e1ad8c6cbb86aab47828a57f89a6a8cda100a3d14df23aee012102cac321783fe7f568a41536ca82ce2347cc5881b787c3bb4aad545da56e52d23cfeffffff73b6de01ef58e8d530850defdf1c1425aa667fd11f0ef85b46f73b368ebe4374000000006b4830450221009355c15204e4be3f92894d4cf186370a6b55bb107f9e029e25951bb2d991e37902201ff8f6f1744602d6a1752887672c4577952ca7a2f3cd9ff546e5e44fa375120c012102cac321783fe7f568a41536ca82ce2347cc5881b787c3bb4aad545da56e52d23cfeffffffaf6be309e92daba9f650cb31c85d7c526ed95a804a9557d7409dfba04bb28456120400006b483045022100e26b98a78b89180d2d4b1e62fe8346eae7e5006538538beaf984816452f805b902207e28c4aa3019510ffd664fb1f47ec31f271e8793cee8200f966eb8521c3cfa3c0121038be5cc928bf73d3895d28f61ae4fe70863e4ba16f7f1d255edde5908d2c82d35fefffffff4e5daf0949f53de19d8a7f94af2e61757e826960b3450dac46d92acadeed321b20400006b483045022100bac393e39f96f3e46c47d9d3a333fe25fcf6294f779974fbcf13cd9b6a105a27022032fc4fbef176d0ea6d2f0fbd64b712bbc33986d8fb62f05e13f762e27e9bcbee0121038be5cc928bf73d3895d28f61ae4fe70863e4ba16f7f1d255edde5908d2c82d35feffffff01225e132a4a00000017a9149349af08ae9c4ee5559bb664104f5e0cc0f102a987d2042600", "txid": "ec2d3ab8906000942dfffc6fb4793e2f95130e41a64fb693c3512119d3a96e8d", "hash": "ec2d3ab8906000942dfffc6fb4793e2f95130e41a64fb693c3512119d3a96e8d", "depends": [], "fee": 942997, "sigops": 0, "weight": 3720}, {"data": "0100000001cb75abe448ad02acf24a70964d210708cbe42646186a2c0c864adc621884ecbf190000006a47304402201dc56ec27da38ac807d67888767611f9e69894068240d53647b5c5f9c6532d7d02201dbd252c296e43ee595525e05e461f1e9e7e26482e52d3468ab7ca5ba03d67d6012102dade2431fc06b2ba964147283ba44fc5073a216a11ec0688c5580602cea20ba9ffffffff012d32a93b000000001976a91484173dfafb6fc629de0f372e8ea20a0b89cf31b088ac00000000", "txid": "ac23877029f22329372c8c9382f22ecdd480b829561c99b4ee28a4bce4b16c17", "hash": "ac23877029f22329372c8c9382f22ecdd480b829561c99b4ee28a4bce4b16c17", "depends": [], "fee": 193325, "sigops": 4, "weight": 764}, {"data": "0100000001cb75abe448ad02acf24a70964d210708cbe42646186a2c0c864adc621884ecbf390000006b483045022100c6e763d10a998b0b51acb3e8d3340d0c0751da711cbb93e16c63a4b8a55d808c022019516d6650553dc51f838873f48689eda94f83fa6c3a4822ec03ceaca79f251b0121025751f9b5946413e83be6b8b61778712dc160ff8af350fe31fc505b757c4627cbffffffff0121d8e13b000000001976a91484173dfafb6fc629de0f372e8ea20a0b89cf31b088ac00000000", "txid": "5bebb64036b0733ed3230a10dc1e93f8ecae0f324239e5928331b3b4adbc79c5", "hash": "5bebb64036b0733ed3230a10dc1e93f8ecae0f324239e5928331b3b4adbc79c5", "depends": [], "fee": 193325, "sigops": 4, "weight": 768}], "coinbaseaux": {"flags": ""}, "coinbasevalue": 250002488333, "longpollid": "0000000000003d02fdcce5f8e62741b431eb8677d878b96b41033ce436551f142904428", "target": "0000000000005ab50d0000000000000000000000000000000000000000000000", "mintime": 1665555669, "mutable": ["time", "transactions", "prevblock"], "noncerange": "00000000ffffffff", "sigoplimit": 80000, "sizelimit": 8000000, "weightlimit": 8000000, "curtime": 1665556235, "bits": "1a5ab50d", "height": 2491604, "default_witness_commitment": "6a24aa21a9edb7efcd0c5c29e3890f1e06bee21568fcbeda8ae211a48c1fb336358729edbb47"}"#;
        let template_info: BlockTemplateInfo = serde_json::from_str(s).unwrap();
        let pool_addr = Address::from_str("RNs3ne88DoNEnXFTqUrj6zrYejeQpcj4jk").unwrap();
        let template = BlockTemplate::new(
            template_info,
            pool_addr,
            "with a little help from http://github.com/kralverde/ravencoin-stratum-proxy"
                .to_string(),
        )
        .unwrap();

        println!("{:?}", template)
    }
}
