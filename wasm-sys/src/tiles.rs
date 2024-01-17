use wasm_bindgen::prelude::*;
use zk_mahjong_core::{babyjubjub::PublicKey, tile::{get_richi_tiles, lookup_tile, shuffle_encrypt_deck}, bn128::Fr, elgamal::MaskedMessage};


#[wasm_bindgen(js_name = "genInitTileSet")]
pub fn gen_init_tile_set() -> Result<JsValue, JsValue> {
    Ok(serde_wasm_bindgen::to_value(&get_richi_tiles()).unwrap())
}

#[wasm_bindgen(js_name = "shuffleEncryptDeck")]
pub fn _shuffle_encrypt_deck(agg_pk: JsValue, tiles: JsValue) -> Result<JsValue, JsValue> {
    let agg_pk: PublicKey = serde_wasm_bindgen::from_value(agg_pk)?;
    let tiles: Vec<MaskedMessage> = serde_wasm_bindgen::from_value(tiles)?;
    Ok(
        serde_wasm_bindgen::to_value(&shuffle_encrypt_deck(&agg_pk, &tiles)).unwrap()
    )
}

#[wasm_bindgen(js_name = "lookupTile")]
pub fn _lookup_tile(tile: JsValue) -> Result<JsValue, JsValue> {
    let tile: Fr = serde_wasm_bindgen::from_value(tile)?;
    Ok(
        serde_wasm_bindgen::to_value(&lookup_tile(&tile).map(|t| t.idx)).unwrap()
    )
}