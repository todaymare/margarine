/*
 * The layout of the binary shall be:
 * - Data
 * - Data Len
 * - Imports
 *   - DLL Count
 *   - DLL Name
 *     - Len
 *     - Name
 *   - Item count
 *   - for _ in 0..Item Count
 *      - Item Name
 *        - Len
 *        - Name: u8 * len
 * - Imports Len
 * - Funcs
 *   - Item Count: u64
 *   - for _ in 0..Item count
 *      - Len: u64
 *      - Name: u8 * len
 * - Crc32 of the data
 * - Crc32 of the imports 
 * - Crc32 of the funcs
 * - Magic Value for making sure nothing has been appended
*/

const MAGIC : &[u8] = b"NICETITS";

pub fn encode(data: &[u8], imports: &[(&str, Vec<&str>)], funcs: &[&str]) -> Vec<u8> {
    let hash_data = crc32fast::hash(data);
    let imports = {
        let mut vec = Vec::new();
        vec.extend_from_slice(&(imports.len() as u64).to_le_bytes());
        for (path, items) in imports.iter() {
            vec.extend_from_slice(&(path.len() as u64).to_le_bytes());
            vec.extend_from_slice(path.as_bytes());

            vec.extend_from_slice(&(items.len() as u64).to_le_bytes());
            for item in items.iter() {
                vec.extend_from_slice(&(item.len() as u64).to_le_bytes());
                vec.extend_from_slice(item.as_bytes());
            }
        }

        vec
    };

    let hash_imports = crc32fast::hash(&imports);

    let funcs = {
        let mut vec = Vec::new();
        vec.extend_from_slice(&(funcs.len() as u64).to_le_bytes());
        for f in funcs {
            vec.extend_from_slice(&(f.len() as u64).to_le_bytes());
            vec.extend_from_slice(f.as_bytes());
        }

        vec
    };

    let hash_funcs = crc32fast::hash(&funcs);

    let mut binary = Vec::new();
    binary.extend_from_slice(data);
    binary.extend_from_slice(&(data.len() as u64).to_le_bytes());

    binary.extend_from_slice(&*imports);
    binary.extend_from_slice(&(imports.len() as u64).to_le_bytes());

    binary.extend_from_slice(&*funcs);
    binary.extend_from_slice(&(funcs.len() as u64).to_le_bytes());
    
    binary.extend_from_slice(&hash_data.to_le_bytes());
    binary.extend_from_slice(&hash_imports.to_le_bytes());
    binary.extend_from_slice(&hash_funcs.to_le_bytes());
    binary.extend_from_slice(MAGIC);
    binary
}


pub fn decode(binary: &'_ [u8]) -> (Vec<(&'_ str, Vec<&'_ str>)>, &'_ [u8], Vec<&'_ str>) {
    assert_eq!(&binary[binary.len() - MAGIC.len()..], MAGIC, "magic not valid");
    let binary = &binary[..binary.len() - MAGIC.len()];

    let hash_funcs = u32::from_le_bytes(binary[binary.len() - 4..].try_into().unwrap());
    let binary = &binary[..binary.len() - 4];

    let hash_imports = u32::from_le_bytes(binary[binary.len() - 4..].try_into().unwrap());
    let binary = &binary[..binary.len() - 4];

    let hash_data = u32::from_le_bytes(binary[binary.len() - 4..].try_into().unwrap());
    let binary = &binary[..binary.len() - 4];

    // funcs
    let len = u64::from_le_bytes(binary[binary.len() - 8..].try_into().unwrap());
    let binary = &binary[..binary.len() - 8];

    let funcs = &binary[binary.len() - len as usize..];
    let funcs_hash = crc32fast::hash(funcs);
    assert_eq!(hash_funcs, funcs_hash, "The CRC32 hash of the imports is not valid");

    let binary = &binary[..binary.len() - len as usize];
    let funcs = {
        let func_count = u64::from_le_bytes(funcs[..8].try_into().unwrap());
        let mut funcs = &funcs[8..];

        let mut vec = Vec::with_capacity(func_count as usize);
        for _ in 0..func_count {
            let name_len = u64::from_le_bytes(funcs[..8].try_into().unwrap());
            funcs = &funcs[8..];
            
            let name = std::str::from_utf8(&funcs[..name_len as usize]).unwrap();
            funcs = &funcs[name_len as usize..];
            vec.push(name);
        }
        vec
    };


    // imports 
    let len = u64::from_le_bytes(binary[binary.len() - 8..].try_into().unwrap());
    let binary = &binary[..binary.len() - 8];

    let imports = &binary[binary.len() - len as usize..];
    let imports_hash = crc32fast::hash(imports);
    assert_eq!(hash_imports, imports_hash, "The CRC32 hash of the imports is not valid");

    let binary = &binary[..binary.len() - len as usize];
    let imports = {
        let dll_count = u64::from_le_bytes(imports[..8].try_into().unwrap());
        let mut imports = &imports[8..];

        let mut vec = Vec::with_capacity(dll_count as usize);
        for _ in 0..dll_count {
            let name_len = u64::from_le_bytes(imports[..8].try_into().unwrap());
            imports = &imports[8..];
            
            let dll_name = std::str::from_utf8(&imports[..name_len as usize]).unwrap();
            imports = &imports[name_len as usize..];

            let item_count = u64::from_le_bytes(imports[..8].try_into().unwrap());
            imports = &imports[8..];

            let mut items = Vec::with_capacity(item_count as usize);
            for _ in 0..item_count {
                let name_len = u64::from_le_bytes(imports[..8].try_into().unwrap());
                imports = &imports[8..];
                
                let item_name = std::str::from_utf8(&imports[..name_len as usize]).unwrap();
                imports = &imports[name_len as usize..];

                items.push(item_name);
            }

            vec.push((dll_name, items));
        }
        vec
    };


    // Data
    let data = {
        let len = u64::from_le_bytes(binary[binary.len() - 8..].try_into().unwrap());
        let binary = &binary[..binary.len() - 8];

        let data = &binary[binary.len() - len as usize..];
        let data_hash = crc32fast::hash(data);
        assert_eq!(hash_data, data_hash, "The CRC32 hash of the data is not valid");

        data
    };

    (imports, data, funcs)
}



