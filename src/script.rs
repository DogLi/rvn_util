use crate::op_data::OpData;
use anyhow::Result;
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone, Default)]
pub struct Script {
    inner: OpData,
}

impl Script {
    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }

    pub fn coinbase_script(height: u32, arbitrary_data: &str) -> Result<Self> {
        let mut data = OpData::default();
        let mut bip34_height = vec![0; 4];
        LittleEndian::write_u32(&mut bip34_height, height);
        let bip34_len = bip34_height.iter().position(|i| i == &0).unwrap_or(4);
        let arbit_data = arbitrary_data.as_bytes();
        data = data
            .op_push_slice(&bip34_height[0..bip34_len])
            .push_u8(0)
            .op_push_slice(arbit_data);
        Ok(Self { inner: data })
    }
}

#[cfg(test)]
mod test {
    pub use super::*;

    #[test]
    fn test_script() {
        let height = 2491604;
        let arbitrary_data =
            "with a little help from http://github.com/kralverde/ravencoin-stratum-proxy";
        let script = Script::coinbase_script(height, arbitrary_data).unwrap();
        let expect = vec![
            3, 212, 4, 38, 0, 75, 119, 105, 116, 104, 32, 97, 32, 108, 105, 116, 116, 108, 101, 32,
            104, 101, 108, 112, 32, 102, 114, 111, 109, 32, 104, 116, 116, 112, 58, 47, 47, 103,
            105, 116, 104, 117, 98, 46, 99, 111, 109, 47, 107, 114, 97, 108, 118, 101, 114, 100,
            101, 47, 114, 97, 118, 101, 110, 99, 111, 105, 110, 45, 115, 116, 114, 97, 116, 117,
            109, 45, 112, 114, 111, 120, 121,
        ];
        assert_eq!(expect, script.as_slice().to_vec());
    }
}
