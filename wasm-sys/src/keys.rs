use zk_mahjong_core::{babyjubjub::{SecretKey, PublicKey}, elgamal::MaskedMessage};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "zkMahjongKeyGen")]
pub fn key_gen() -> JsValue {
    let key = SecretKey::random();
    serde_wasm_bindgen::to_value(&key).unwrap()
}

#[wasm_bindgen(js_name = "zkMahjongKeyToPubkey")]
pub fn to_pubkey(sk: JsValue) -> Result<JsValue, JsValue> {
    let key: SecretKey = serde_wasm_bindgen::from_value(sk)?;
    let pk = key.public_key();
    Ok(serde_wasm_bindgen::to_value(&pk).unwrap())
}

#[wasm_bindgen(js_name = "zkMahjongPubkeyAggregate")]
pub fn aggregate_pubkey(pks: JsValue) -> Result<JsValue, JsValue> {
    let keys: Vec<PublicKey> = serde_wasm_bindgen::from_value(pks)?;
    let agg_pk = PublicKey::aggregate(keys);
    Ok(serde_wasm_bindgen::to_value(&agg_pk).unwrap())
}

#[wasm_bindgen(js_name = "zkMahjongUnmaskMessage")]
pub fn unmask_message(
    sk: JsValue,
    m: JsValue,
) -> Result<JsValue, JsValue> {
    let sk: SecretKey = serde_wasm_bindgen::from_value(sk)?;
    let msg: MaskedMessage = serde_wasm_bindgen::from_value(m)?;
    Ok(serde_wasm_bindgen::to_value(&msg.unmask(&sk).c1).unwrap())
}