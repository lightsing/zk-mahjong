pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/bitify.circom";
include "../node_modules/circomlib/circuits/escalarmulfix.circom";

template BabyPkCheck(NUM_BITS, BASE8) {
    signal input sk;
    signal output pk[2];

    component bitify = Num2Bits(NUM_BITS);
    bitify.in <== sk;
    component mul = EscalarMulFix(NUM_BITS, BASE8);
    mul.e <== bitify.out;
    pk <== mul.out;
}

component main = BabyPkCheck(254, [
    5299619240641551281634865583518297030282874472190772894086521144482721001553,
    16950150798460657717958625567821834550301663161624707787222815936182638968203
]);