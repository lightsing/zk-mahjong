pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/babyjub.circom";

include "./babyjubjub.circom";
include "./elgamal_template.circom";
include "./matrix.circom";

template ShuffleEncrypt(NUM_PLAYERS, NUM_CARDS) {
    signal input agg_pk[2]; // Aggregated public keys
    signal input in_c0[NUM_CARDS][2]; // Points of the cards
    signal input in_c1[NUM_CARDS][2]; // Points of the cards
    signal input in_r[NUM_CARDS]; // Randomness for the encryption
    signal input permutation[NUM_CARDS][NUM_CARDS]; // Permutation of the cards

    // use as intermediate, avoid large public inputs
    signal out_c0[NUM_CARDS][2]; // Points of the cards
    signal out_c1[NUM_CARDS][2]; // Points of the cards

    // check that the agg_pk is valid point
    component baby_check = BabyCheck();
    baby_check.x <== agg_pk[0];
    baby_check.y <== agg_pk[1];

    var BASE8[2] = [
        5299619240641551281634865583518297030282874472190772894086521144482721001553,
        16950150798460657717958625567821834550301663161624707787222815936182638968203
    ];

    component mask = ElGamalMask(254, BASE8, NUM_CARDS);
    mask.in_pk <== agg_pk;
    mask.in_r <== in_r;
    mask.in_c0 <== in_c0;
    mask.in_c1 <== in_c1;

    component matrix_mul_c0 = MatrixMultiplier(NUM_CARDS, NUM_CARDS, 2);
    matrix_mul_c0.A <== permutation;
    matrix_mul_c0.B <== mask.out_c0;
    out_c0 <== matrix_mul_c0.out;

    component matrix_mul_c1 = MatrixMultiplier(NUM_CARDS, NUM_CARDS, 2);
    matrix_mul_c1.A <== permutation;
    matrix_mul_c1.B <== mask.out_c1;
    out_c1 <== matrix_mul_c1.out;
}

component main {public [agg_pk]} = ShuffleEncrypt(4, 136);
