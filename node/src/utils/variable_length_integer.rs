/// Enum that represents the possible errors that can occur when creating a VarLenInt.
pub enum VarLenIntError {
    ErrorIncorrectAmountOfBytes,
}

/// Struct that represents a variable length integer. This is used in various places to
/// represent a type of integer that varies it size depending on its value.
#[derive(Debug, PartialEq)]
pub struct VarLenInt {
    bytes: Vec<u8>,
}

impl VarLenInt {
    /// Creates a new VarLenInt from a usize.
    pub fn new(value: usize) -> VarLenInt {
        let mut bytes = Vec::new();
        if value < 253 {
            bytes.push(value as u8);
        } else if value < 2usize.pow(16) {
            bytes.push(0xfd);
            bytes.extend_from_slice(&(value as u16).to_le_bytes());
        } else if value < 2usize.pow(32) {
            bytes.push(0xfe);
            bytes.extend_from_slice(&(value as u32).to_le_bytes());
        } else if value < 2usize.pow(64) {
            bytes.push(0xff);
            bytes.extend_from_slice(&(value as u64).to_le_bytes());
        }
        VarLenInt { bytes }
    }

    /// Creates a new VarLenInt from a slice of bytes.
    pub fn from_bytes(slice: &[u8]) -> Option<VarLenInt> {
        if slice.is_empty() {
            return None;
        }
        let mut bytes = Vec::new();
        let mut amount_of_bytes = 1;
        if slice[0] == 0xfd {
            amount_of_bytes = 3;
        }
        if slice[0] == 0xfe {
            amount_of_bytes = 5;
        }
        if slice[0] == 0xff {
            amount_of_bytes = 9;
        }
        if slice.len() < amount_of_bytes {
            return None;
        }
        for byte in slice.iter().take(amount_of_bytes) {
            bytes.push(*byte);
        }
        Some(VarLenInt { bytes })
    }

    /// Returns the value of the VarLenInt as a usize.
    pub fn to_usize(&self) -> usize {
        let mut value: usize = self.bytes[0] as usize;

        if self.bytes[0] == 0xfd {
            let array: [u8; 2] = [self.bytes[1], self.bytes[2]];
            value = u16::from_le_bytes(array) as usize;
        }
        if self.bytes[0] == 0xfe {
            let mut array: [u8; 4] = [0_u8; 4];
            array[..(self.amount_of_bytes() - 1)]
                .copy_from_slice(&self.bytes[1..self.amount_of_bytes()]);
            value = u32::from_le_bytes(array) as usize;
        }
        if self.bytes[0] == 0xff {
            let mut array: [u8; 8] = [0_u8; 8];
            array[..(self.amount_of_bytes() - 1)]
                .copy_from_slice(&self.bytes[1..self.amount_of_bytes()]);
            value = u64::from_le_bytes(array) as usize;
        }

        value
    }

    /// Returns the bytes of the VarLenInt as a vector of u8.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Returns the amount of bytes of the VarLenInt.
    pub fn amount_of_bytes(&self) -> usize {
        self.bytes.len()
    }
}
