use wasm_bindgen::prelude::*;
use zk_mahjong_core::{babyjubjub::PublicKey, tile::{get_richi_tile_set, lookup_tile}, bn128::Fr};


#[wasm_bindgen(js_name = "genInitTileSet")]
pub fn gen_init_tile_set(agg_pk: JsValue) -> Result<JsValue, JsValue> {
    let agg_pk: PublicKey = serde_wasm_bindgen::from_value(agg_pk)?;
    let tile_set = get_richi_tile_set(&agg_pk);
    Ok(serde_wasm_bindgen::to_value(&tile_set).unwrap())
}

#[wasm_bindgen(js_name = "lookupTile")]
pub fn _lookup_tile(tile: JsValue) -> Result<JsValue, JsValue> {
    let tile: Fr = serde_wasm_bindgen::from_value(tile)?;
    Ok(
        serde_wasm_bindgen::to_value(&lookup_tile(&tile).map(|t| t.idx)).unwrap()
    )
}