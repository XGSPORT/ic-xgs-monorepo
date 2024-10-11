use serde::Serialize;
use serde_cbor::Serializer;

pub fn cbor_serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut data = vec![];
    let mut serializer = Serializer::new(&mut data);
    serializer.self_describe().map_err(|e| e.to_string())?;
    value
        .serialize(&mut serializer)
        .map_err(|e| e.to_string())?;
    Ok(data)
}
