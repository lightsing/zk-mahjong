use super::Point;
use crate::bn128::Fr;

pub const D: Fr = Fr::from_raw([
    0x2735f484aff261f5,
    0x70ba1b579a2e0f63,
    0xff41c9a91e2caa8c,
    0x7704a8e8fe6025f,
]);
pub const A: Fr = Fr::from_raw([
    0x95accf61fff261e0,
    0x24780d659df7d378,
    0xe0ac11b07e906ae8,
    0xf35db2216d3def3,
]);
pub const BASE_POINT: Point = Point {
    x: Fr::from_raw([
        0x0a8fc7bc1a89fa86,
        0xa7d9d786e9e48627,
        0xee6158b465bea369,
        0x14a0ff6d2f874519,
    ]),
    y: Fr::from_raw([
        0xb83342d20d0201aa,
        0x2ffef2f7cdcfeac7,
        0xbfa79a9425a6e625,
        0x0dfb859dc3a44b70,
    ]),
};
pub const ORDER: Fr = Fr::from_raw([
    0xa23e286ed4e1f3f3,
    0x62a18081708c31ae,
    0x6021d6f042c466cf,
    0x1b4c14b255111d4b,
]);
pub const SUBORDER: Fr = Fr::from_raw([
    0x9eb4fe8a509c3e7f,
    0x2574a13d7a256c90,
    0x7f36667019496414,
    0x21a8339e176127c3,
]);
pub const Q_SHR_1: Fr = Fr::from_raw([0xcba5e0bbd0000003, 0x789bb8d96d2c51b3, 0x28f0d12384840917, 0x112ceb58a394e07d]);