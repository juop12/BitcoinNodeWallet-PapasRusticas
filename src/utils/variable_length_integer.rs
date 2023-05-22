
/// Enum that represents the possible errors that can occur when creating a VarLenInt
pub enum VarLenIntError {
    ErrorIncorrectAmountOfBytes,
}

/// Struct that represents a variable length integer. This is used in various places to
/// represent a type of integer that varies it size depending on its value.
#[derive(Debug, PartialEq)]
pub struct VarLenInt{
    bytes: Vec<u8>
}

impl VarLenInt{
    /// -
    pub fn new(value :usize) -> VarLenInt {
        let mut bytes = Vec::new();
        if value < 253 {
            bytes.push(value as u8);
        } else if value  < 2usize.pow(16) {
            bytes.push(0xfd);
            bytes.extend_from_slice(&(value as u16).to_le_bytes());
        }
        else if value  < 2usize.pow(32) {
            bytes.push(0xfe);
            bytes.extend_from_slice(&(value as u32).to_le_bytes());
        }
        else if value  < 2usize.pow(64) {
            bytes.push(0xff);
            bytes.extend_from_slice(&(value as u64).to_le_bytes());
        }
        VarLenInt { bytes }
    }

    /// -
    pub fn from_bytes(slice : &[u8])-> VarLenInt{
        let mut bytes = Vec::new();
        let mut amount_of_bytes = 1;
        if slice[0] == 0xfd{
            amount_of_bytes = 3;
        }
        if slice[0] == 0xfe{
            amount_of_bytes = 5;
        }
        if slice[0] == 0xff{
            amount_of_bytes = 9;
        }
        for i in 0..amount_of_bytes{
            bytes.push(slice[i]);
        }
        VarLenInt{ bytes }
    }

    /// -
    pub fn to_usize(&self)->usize{
        let mut value :usize = self.bytes[0] as usize;
        
        if self.bytes[0] == 0xfd {
            let array: [u8; 2] = [self.bytes[1], self.bytes[2]];
            value = u16::from_le_bytes(array) as usize;
        }
        if self.bytes[0] == 0xfe {
            let mut array: [u8; 4] = [0_u8; 4];
            for i in 0..(self.amount_of_bytes() - 1){
                array[i] = self.bytes[i+1];
            }
            value = u32::from_le_bytes(array) as usize;
        }
        if self.bytes[0] == 0xff {
            let mut array: [u8; 8] = [0_u8; 8];
            for i in 0..(self.amount_of_bytes() - 1){
                array[i] = self.bytes[i+1];
            }
            value = u64::from_le_bytes(array) as usize;
        }

        return value;
    }

    /// -
    pub fn to_bytes(&self)-> Vec<u8>{
        self.bytes.clone()
    }

    /// -
    pub fn amount_of_bytes(&self)->usize{
        self.bytes.len()
    }
}