use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone, Default)]
pub struct OpData {
    inner: Vec<u8>,
}

impl OpData {
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }

    /// https://en.bitcoin.it/wiki/Protocol_documentation#Variable_length_integer
    pub fn var_push_num(mut self, i: u64) -> Self {
        if i < 0xfd {
            self.inner.push(i as u8)
        } else if i <= 0xffff {
            self.inner.push(0xfd);
            let mut tmp = [0; 2];
            LittleEndian::write_u16(&mut tmp, i as u16);
            self.inner.extend_from_slice(&tmp);
        } else if i <= 0xffff_ffff {
            self.inner.push(0xfe);
            let mut tmp = [0; 4];
            LittleEndian::write_u32(&mut tmp, i as u32);
            self.inner.extend_from_slice(&tmp);
        } else {
            self.inner.push(0xff);
            let mut tmp = [0; 8];
            LittleEndian::write_u64(&mut tmp, i);
            self.inner.extend_from_slice(&tmp);
        }
        self
    }

    pub fn push_u8(mut self, data: u8) -> Self {
        self.inner.push(data);
        self
    }

    pub fn push_u32(mut self, i: u32) -> Self {
        let mut data = vec![0; 4];
        LittleEndian::write_u32(&mut data, i);
        self.inner.extend(&data);
        self
    }

    pub fn push_u64(mut self, i: u64) -> Self {
        let mut data = vec![0; 8];
        LittleEndian::write_u64(&mut data, i);
        self.inner.extend(&data);
        self
    }

    pub fn push_slice(mut self, data: &[u8]) -> Self {
        self.inner.extend_from_slice(data);
        self
    }

    pub fn op_push_slice(mut self, data: &[u8]) -> Self {
        // Start with a PUSH opcode
        match data.len() as u64 {
            n if n < 0x4c => {
                self.inner.push(n as u8);
            }
            n if n < 0x100 => {
                self.inner.push(0x4c);
                self.inner.push(n as u8);
            }
            n if n < 0x10000 => {
                self.inner.push(0x4d);
                self.inner.push((n % 0x100) as u8);
                self.inner.push((n / 0x100) as u8);
            }
            n if n < 0x100000000 => {
                self.inner.push(0x4e);
                self.inner.push((n % 0x100) as u8);
                self.inner.push(((n / 0x100) % 0x100) as u8);
                self.inner.push(((n / 0x10000) % 0x100) as u8);
                self.inner.push((n / 0x1000000) as u8);
            }
            _ => panic!("tried to put a 4bn+ sized object into a script!"),
        }
        // Then push the raw bytes
        self.inner.extend(data.iter().cloned());
        self
    }
}
