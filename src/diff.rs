use anyhow::{bail, Result};
pub use bitcoin::util::uint::Uint256;
use std::collections::VecDeque;

pub fn uint256_from_hash(s: &str) -> Result<Uint256> {
    let s = s.trim_start_matches("0x");
    let raw = hex::decode(s)?;
    let num = Uint256::from_be_slice(&raw)?;
    Ok(num)
}

pub fn uint256_from_bytes(d: [u8; 32]) -> Uint256 {
    let be_data: Vec<_> = d.into_iter().rev().collect();
    Uint256::from_be_slice(&be_data).unwrap()
}

pub fn bits2target(bits: u32) -> Uint256 {
    // from https://docs.rs/bitcoin/0.23.0/src/bitcoin/blockdata/block.rs.html#126-146
    let (mant, expt) = {
        let unshifted_expt = bits >> 24;
        if unshifted_expt <= 3 {
            ((bits & 0xFFFFFF) >> (8 * (3 - unshifted_expt as usize)), 0)
        } else {
            (bits & 0xFFFFFF, 8 * ((bits >> 24) - 3))
        }
    };

    // The mantissa is signed but may not be negative
    if mant > 0x7FFFFF {
        Default::default()
    } else {
        Uint256::from_u64(mant as u64).unwrap() << (expt as usize)
    }
}

pub fn parse_bits(str: &str) -> Result<u32> {
    let b = hex::decode(&str)?;
    if b.len() != 4 {
        bail!("invalid bits");
    }
    Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn unit_target() -> Uint256 {
    Uint256::from_u64(0xFFFF).unwrap() << 208
    // 第一字节是最低位
    // Uint256([
    //     0xffffffffffffffffu64,
    //     0xffffffffffffffffu64,
    //     0xffffffffffffffffu64,
    //     0x00000000ffffffffu64,
    // ])
}

/// 计算目标值对应的难度
pub fn target2diff(target: Uint256) -> u64 {
    (unit_target() / target).low_u64()
}

/// Uint256 相除得到浮点数
fn uint256_div(divided: Uint256, divisor: Uint256, decimal_len: usize) -> Result<f64> {
    if divisor == Uint256::from_u64(0u64).unwrap() {
        bail!("zero divisor")
    }
    if decimal_len > 10 {
        bail!("decimal > 10");
    }

    let d = divided / divisor;
    {
        let bytes = d.as_bytes();
        if bytes[3] != 0 || bytes[2] != 0 || bytes[1] != 0 {
            bail!("div result too big");
        }
    }
    let mut decimals: VecDeque<u64> = VecDeque::with_capacity(decimal_len + 1);

    let mut divided = divided;
    let mut divided_result: u64 = d.low_u64();
    decimals.push_back(divided_result);
    for _i in 0..decimal_len {
        divided = divided - divisor * Uint256::from_u64(divided_result).unwrap();
        divided = divided * Uint256::from_u64(10).unwrap();
        divided_result = (divided / divisor).low_u64() % 10;
        decimals.push_back(divided_result);
    }

    let mut result = decimals.pop_front().unwrap() as f64;
    let mut ratio = 0.1;
    for d in decimals {
        result += d as f64 * ratio;
        ratio /= 10.0;
    }
    Ok(result)
}

/// 计算难度值对应的目标值
pub fn diff2target(diff: u64) -> Uint256 {
    if diff == 0 {
        return Uint256([
            0xffffffffffffffffu64,
            0xffffffffffffffffu64,
            0xffffffffffffffffu64,
            0xffffffffffffffffu64,
        ]);
    }

    unit_target() / Uint256([diff, 0, 0, 0])
}

/// 仅用于计算链上难度，不要用于性能敏感的场合
pub fn target2diff_f64(target: Uint256) -> Result<f64> {
    uint256_div(unit_target(), target, 10)
}

#[cfg(test)]
mod test {
    use super::*;
    pub use bitcoin::util::uint::Uint256;

    #[test]
    fn test_bits() {
        let bits = "1e0090f9";
        let bits_num = parse_bits(bits).unwrap();
        let block_target = bits2target(bits_num);
        let block_target2 =
            uint256_from_hash("00000090f9000000000000000000000000000000000000000000000000000000")
                .unwrap();
        assert_eq!(block_target2, block_target);
    }

    #[test]
    fn test_diff() {
        let mix_target = uint256_from_bytes([
            146, 149, 38, 139, 144, 227, 187, 148, 138, 108, 170, 235, 138, 113, 53, 205, 105, 90,
            13, 49, 105, 33, 82, 87, 104, 157, 171, 146, 119, 210, 83, 156,
        ]);
        let block_target =
            uint256_from_hash("00000090f9000000000000000000000000000000000000000000000000000000")
                .unwrap();
        // assert!(share_target > mix_target);
        let real_target = uint256_from_bytes([
            85, 209, 227, 98, 133, 167, 126, 147, 55, 145, 98, 149, 234, 155, 102, 64, 122, 242,
            245, 169, 3, 210, 8, 12, 232, 167, 168, 87, 43, 45, 232, 23,
        ]);
        println!("real target: {}", real_target);
        println!("mix target: {}", mix_target);
        let diff = target2diff(real_target);
        let diff_f64 = target2diff_f64(mix_target).unwrap();
        println!("diff: {}, {}", diff, diff_f64);
        assert!(mix_target > block_target);
    }
}
