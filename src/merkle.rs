use crate::block_template::dsha256;
use std::collections::VecDeque;

pub fn merkel_hash(txids: Vec<[u8; 32]>) -> [u8; 32] {
    if txids.is_empty() {
        return dsha256(b"")
    }
    if txids.len() == 1 {
        return txids[0];
    }
    let mut txids: VecDeque<_> = txids.into_iter().collect();
    let mut data = Vec::with_capacity(32 * 2);
    while txids.len() > 1 {
        if txids.len() % 2 == 1 {
            txids.push_back(*txids.back().unwrap());
        }
        let mut tmp_txids = VecDeque::new();
        while !txids.is_empty() {
            let first = txids.pop_front().unwrap();
            let second = txids.pop_front().unwrap();
            data.clear();
            data.extend_from_slice(&first);
            data.extend_from_slice(&second);
            let hash = dsha256(&data);
            tmp_txids.push_back(hash);
        }
        txids = tmp_txids;
    }
    txids.pop_front().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merkel_hash() {
        // even number
        let txids = vec![
            "ec2d3ab8906000942dfffc6fb4793e2f95130e41a64fb693c3512119d3a96e8d",
            "ac23877029f22329372c8c9382f22ecdd480b829561c99b4ee28a4bce4b16c17",
            "5bebb64036b0733ed3230a10dc1e93f8ecae0f324239e5928331b3b4adbc79c5",
            "784f313ab617c14e08139f0e4257304eda8a82b6d1ed142d0d5d02d8d9772fde",
        ];
        let txids: Vec<_> = txids
            .into_iter()
            .map(|s| {
                let mut h = hex::decode(s).expect("invalid txid");
                h.reverse();
                h.try_into().unwrap()
            })
            .collect();

        let hash = merkel_hash(txids);
        let hash_exp = [
            164, 138, 132, 38, 242, 38, 43, 175, 80, 36, 97, 27, 230, 230, 92, 110, 198, 155, 84,
            180, 201, 165, 88, 181, 44, 125, 16, 244, 103, 183, 95, 83,
        ];
        assert_eq!(hash, hash_exp);

        // odd number
        let txids = vec![
            "ac23877029f22329372c8c9382f22ecdd480b829561c99b4ee28a4bce4b16c17",
            "5bebb64036b0733ed3230a10dc1e93f8ecae0f324239e5928331b3b4adbc79c5",
            "784f313ab617c14e08139f0e4257304eda8a82b6d1ed142d0d5d02d8d9772fde",
        ];
        let txids: Vec<_> = txids
            .into_iter()
            .map(|s| {
                let mut h = hex::decode(s).expect("invalid txid");
                h.reverse();
                h.try_into().unwrap()
            })
            .collect();

        let hash_exp = [
            43, 193, 31, 177, 111, 252, 63, 155, 251, 38, 69, 218, 188, 229, 206, 115, 142, 87,
            238, 163, 154, 177, 210, 12, 134, 219, 208, 184, 173, 181, 242, 210,
        ];
        let hash = merkel_hash(txids);
        assert_eq!(hash, hash_exp);

        // empty
        let hash = merkel_hash(Vec::<[u8; 32]>::new());
        let hash_exp = [
            93_u8, 246, 224, 226, 118, 19, 89, 211, 10, 130, 117, 5, 142, 41, 159, 204, 3, 129, 83,
            69, 69, 245, 92, 244, 62, 65, 152, 63, 93, 76, 148, 86,
        ];
        assert_eq!(hash, hash_exp);
    }
}
