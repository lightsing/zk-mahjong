pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/babyjub.circom";
include "../node_modules/circomlib/circuits/escalarmulany.circom";
include "../node_modules/circomlib/circuits/escalarmulfix.circom";

// out = r * G + in
template BabyMulAddFix(NUM_BITS, BASE8) {
    signal input in[2];
    signal input r_bits[254];
    signal output out[2];

    component mul = EscalarMulFix(NUM_BITS, BASE8);
    component add = BabyAdd();
    mul.e <== r_bits;
    add.x1 <== mul.out[0];
    add.y1 <== mul.out[1];
    add.x2 <== in[0];
    add.y2 <== in[1];
    out[0] <== add.xout;
    out[1] <== add.yout;
}

// out = r * P + in
template BabyMulAddAny(NUM_BITS) {
    signal input in[2];
    signal input r_bits[254];
    signal input p[2];
    signal output out[2];

    component mul = EscalarMulAny(NUM_BITS);
    component add = BabyAdd();
    mul.e <== r_bits;
    mul.p <== p;
    add.x1 <== mul.out[0];
    add.y1 <== mul.out[1];
    add.x2 <== in[0];
    add.y2 <== in[1];
    out[0] <== add.xout;
    out[1] <== add.yout;
}

template BabyPkCheck(NUM_BITS, BASE8) {
    signal input sk_bits[NUM_BITS];
    signal output pk[2];

    component mul = EscalarMulFix(NUM_BITS, BASE8);
    mul.e <== sk_bits;
    pk <== mul.out;
}

template BabyAggPk(N) {
    signal input in[N][2];
    signal output out[2];

    component check[N];
    for (var i = 0; i < N; i++) {
        check[i] = BabyCheck();
        check[i].x <== in[i][0];
        check[i].y <== in[i][1];
    }

    component add[N];

    add[0] = BabyAdd();
    add[0].x1 <== in[0][0];
    add[0].y1 <== in[0][1];
    add[0].x2 <== in[1][0];
    add[0].y2 <== in[1][1];

    for (var i = 1; i < N - 1; i++) {
        add[i] = BabyAdd();
        add[i].x1 <== add[i - 1].xout;
        add[i].y1 <== add[i - 1].yout;
        add[i].x2 <== in[i + 1][0];
        add[i].y2 <== in[i + 1][1];
    }

    out[0] <== add[N - 2].xout;
    out[1] <== add[N - 2].yout;
}