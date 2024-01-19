pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/babyjub.circom";
include "../node_modules/circomlib/circuits/bitify.circom";
include "../node_modules/circomlib/circuits/escalarmulany.circom";
include "../node_modules/circomlib/circuits/escalarmulfix.circom";

include "./babyjubjub.circom";

// c0 = r * G + ic0
// c1 = r * pk + ic1
template ElGamalMask(NUM_BITS, BASE8, N) {
    signal input in_pk[2];
    signal input in_r[N];
    signal input in_c0[N][2];
    signal input in_c1[N][2];

    signal output out_c0[N][2];
    signal output out_c1[N][2];

    // check inputs are on the curve
    component point[2 * N];
    for (var i = 0; i < N; i++) {
        point[i] = BabyCheck();
        point[i].x <== in_c0[i][0];
        point[i].y <== in_c0[i][1];
        point[i + N] = BabyCheck();
        point[i + N].x <== in_c1[i][0];
        point[i + N].y <== in_c1[i][1];
    }

    component bitify[N];
    for (var i = 0; i < N; i++) {
        bitify[i] = Num2Bits(NUM_BITS);
        bitify[i].in <== in_r[i];
    }

    component mul_add_fix[N];
    component mul_add_any[N];
    for (var i = 0; i < N; i++) {
        // c0 = r * G + ic0
        mul_add_fix[i] = BabyMulAddFix(NUM_BITS, BASE8);
        mul_add_fix[i].in <== in_c0[i];
        mul_add_fix[i].r_bits <== bitify[i].out;
        out_c0[i] <== mul_add_fix[i].out;

        // c1 = r * pk + ic1
        mul_add_any[i] = BabyMulAddAny(NUM_BITS);
        mul_add_any[i].in <== in_c1[i];
        mul_add_any[i].r_bits <== bitify[i].out;
        mul_add_any[i].p <== in_pk;
        out_c1[i] <== mul_add_any[i].out;
    }
}

// out = c1 - sk * c0
template ElGamalUnmask(NUM_BITS, BASE8, N) {
    signal input sk;
    signal input in_c0[N][2];
    signal input in_c1[N][2];

    signal output pk[2];
    signal output out[N][2];

    component bitify = Num2Bits(NUM_BITS);
    bitify.in <== sk;

    // pk = sk * G
    component pk_check = BabyPkCheck(NUM_BITS, BASE8);
    pk_check.sk_bits <== bitify.out;
    pk <== pk_check.pk;

    component point[N][2];
    component mul[N];
    component add[N];
    for (var i = 0; i < N; i++) {
        // check inputs are on the curve
        point[i][0] = BabyCheck();
        point[i][0].x <== in_c0[i][0];
        point[i][0].y <== in_c0[i][1];
        point[i][1] = BabyCheck();
        point[i][1].x <== in_c1[i][0];
        point[i][1].y <== in_c1[i][1];

        // mul.out = sk * c0
        mul[i] = EscalarMulAny(NUM_BITS);
        mul[i].e <== bitify.out;
        mul[i].p <== in_c0[i];

        // add.out = c1 - sk * c0
        add[i] = BabyAdd();
        add[i].x1 <== 0 - mul[i].out[0];
        add[i].y1 <== mul[i].out[1];
        add[i].x2 <== in_c1[i][0];
        add[i].y2 <== in_c1[i][1];

        out[i][0] <== add[i].xout;
        out[i][1] <== add[i].yout;
    }
}