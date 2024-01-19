pragma circom 2.0.0;

// Given a vector A (N) and a vector B (N), compute A * B.
template VectorMultiplier(N) {
    signal input A[N];
    signal input B[N];
    signal output out[N];
    
    for (var i = 0; i < N; i++) {
        out[i] <== A[i] * B[i];
    }
}

// Sum all elements in a vector A (N).
template VectorSum(N) {
    signal input in[N];
    signal output out;

    var acc = 0;
    for (var i = 0; i < N; i++) {
        acc = acc + in[i];
    }

    out <== acc;
}

// Given matrix A (M rows x N cols) and matrix B (N rows x P cols), compute A * B.
template MatrixMultiplier(M, N, P) {
    signal input A[M][N]; // [ 00..0N 10..1N 20..2N ... M0..MN ]
    signal input B[N][P]; // [ 00..0P 10..1P 20..2P ... N0..NP ]
    signal output out[M][P];

    component vm[M][P];
    component vs[M][P];

    for (var i = 0; i < M; i++) {
        for (var j = 0; j < P; j++) {
            // multiply row i of A with column j of B, N elements each
            vm[i][j] = VectorMultiplier(N);
            for (var k = 0; k < N; k++) {
                vm[i][j].A[k] <== A[i][k];
                vm[i][j].B[k] <== B[k][j];
            }
            // sum the result
            vs[i][j] = VectorSum(N);
            vs[i][j].in <== vm[i][j].out;
            out[i][j] <== vs[i][j].out;
        }
    }
}

// Given matrix A (N rows x N cols), constrain it to be a permutation matrix.
// A permutation matrix is a square binary matrix that has exactly one entry 1 in each row and each column and 0s elsewhere.
// The order of the input is row-major.
template ConstrainPermutationMatrix(N) {
    signal input in[N][N];

    component lvs[N]; // line vector sum
    component cvs[N]; // column vector sum

    for (var i = 0; i < N; i++) {
        lvs[i] = VectorSum(N);
        cvs[i] = VectorSum(N);
        for (var j = 0; j < N; j++) {
            // each cell is either 0 or 1
            in[i][j] * (in[i][j] - 1) === 0;
            // sum of each row and column
            lvs[i].in[j] <== in[i][j];
            cvs[i].in[j] <== in[j][i];
        }
        // sum is 1
        lvs[i].out === 1;
        cvs[i].out === 1;
    }
}