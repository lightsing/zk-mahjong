pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/babyjub.circom";
include "../node_modules/circomlib/circuits/bitify.circom";
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

// c0 = r * G + ic0
// c1 = r * pk + ic1
template ElGamalMask(NUM_BITS, BASE8) {
    signal input in_pk[2];
    signal input in_r;
    signal input in_c0[2];
    signal input in_c1[2];

    signal output out_c0[2];
    signal output out_c1[2];

    component bitify = Num2Bits(NUM_BITS);
    bitify.in <== in_r;
    
    component mul_add_c0 = BabyMulAddFix(NUM_BITS, BASE8);
    mul_add_c0.in <== in_c0;
    mul_add_c0.r_bits <== bitify.out;
    out_c0 <== mul_add_c0.out;

    component mul_add_c1 = BabyMulAddAny(NUM_BITS);
    mul_add_c1.in <== in_c1;
    mul_add_c1.r_bits <== bitify.out;
    mul_add_c1.p <== in_pk;
    out_c1 <== mul_add_c1.out;
}

template BabyPkCheck(NUM_BITS, BASE8) {
    signal input sk_bits[NUM_BITS];
    signal output pk[2];

    component mul = EscalarMulFix(NUM_BITS, BASE8);
    mul.e <== sk_bits;
    pk <== mul.out;
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

    component mul[N];
    component add[N];
    for (var i = 0; i < N; i++) {
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

component main = ElGamalUnmask(254, [
    5299619240641551281634865583518297030282874472190772894086521144482721001553,
    16950150798460657717958625567821834550301663161624707787222815936182638968203
], 1);