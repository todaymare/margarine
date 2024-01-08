/*
 * The layout of the binary shall be:
 * - Binary
 * - Data
 * - Data Len
 * - Crc32 of the data
 * - Magic Value for making sure nothing has been appended
*/

const MAGIC : &[u8] = b"NICETITS";

pub fn encode(binary: &mut Vec<u8>, data: &[u8]) {
    let hash = crc32fast::hash(data);

    binary.extend_from_slice(data);
    binary.extend_from_slice(&(data.len() as u64).to_le_bytes());
    binary.extend_from_slice(&hash.to_le_bytes());
    binary.extend_from_slice(MAGIC);
}


pub fn decode(binary: &[u8]) -> Vec<u8> {
    assert_eq!(&binary[binary.len() - MAGIC.len()..], MAGIC, "magic not valid");
    let binary = &binary[..binary.len() - MAGIC.len()];

    let hash = u32::from_le_bytes(binary[binary.len() - 4..].try_into().unwrap());
    let binary = &binary[..binary.len() - 4];

    let len = u64::from_le_bytes(binary[binary.len() - 8..].try_into().unwrap());
    let binary = &binary[..binary.len() - 8];

    let data = &binary[binary.len() - len as usize..];

    let data_hash = crc32fast::hash(data);
    assert_eq!(hash, data_hash, "The CRC32 hash of the data is not valid");

    data.to_vec()
}
