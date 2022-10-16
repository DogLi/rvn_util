use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Address {
    inner: String,
    testnet: bool,
}

impl FromStr for Address {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let checker = bs58::decode(&s).with_check(None).into_vec()?;
        let testnet = if checker[0] == 111 {
            Ok::<bool, Self::Err>(true)
        } else if checker[0] == 60 {
            Ok(false)
        } else {
            bail!("Invalid Address")
        }?;
        Ok(Self {
            inner: s.into(),
            testnet,
        })
    }
}

impl Address {
    pub fn vout_to_miner(&self) -> Vec<u8> {
        let checker = bs58::decode(&self.inner)
            .with_check(None)
            .into_vec()
            .unwrap();
        // let checker = self.inner.from_base58().unwrap();
        let mut data = vec![0x76, 0xa9, 0x14];
        data.extend_from_slice(&checker[1..]);
        data.extend_from_slice(&[0x88, 0xac]);
        data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_address() {
        let addr = Address::from_str("RNs3ne88DoNEnXFTqUrj6zrYejeQpcj4jk").unwrap();
        let out = addr.vout_to_miner();
        let out_exp = vec![
            118_u8, 169, 20, 149, 0, 219, 97, 53, 71, 189, 57, 112, 252, 206, 194, 167, 169, 9,
            185, 46, 117, 0, 89, 136, 172,
        ];
        assert_eq!(out, out_exp)
    }
}
