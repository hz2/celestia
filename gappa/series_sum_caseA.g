# amplitudes mirrored in Rust as celestia::series::GAPPA_PROOF_AMPLITUDES
# -- keep both in sync by hand, Gappa can't import a Rust constant.
@fl64 = float<ieee_64,ne>;

a1 = 1.0975e-1;
a2 = 1.274e-2;
a3 = 6.58e-4;
a4 = 2.14e-4;
a5 = 1.11e-4;
a6 = 4.6e-5;

c1 = cosx1 + d1;
c2 = cosx2 + d2;
c3 = cosx3 + d3;
c4 = cosx4 + d4;
c5 = cosx5 + d5;
c6 = cosx6 + d6;

t1 = fl64(a1 * c1);
t2 = fl64(a2 * c2);
t3 = fl64(a3 * c3);
t4 = fl64(a4 * c4);
t5 = fl64(a5 * c5);
t6 = fl64(a6 * c6);

s1 = fl64(t1 + t2);
s2 = fl64(s1 + t3);
s3 = fl64(s2 + t4);
s4 = fl64(s3 + t5);
s  = fl64(s4 + t6);

s_exact = a1 * cosx1 + a2 * cosx2 + a3 * cosx3
        + a4 * cosx4 + a5 * cosx5 + a6 * cosx6;

# per-term error: floating-point rounding of the product, plus the
# cosine-implementation error carried through the multiply.
e1 = t1 - a1 * cosx1;
e2 = t2 - a2 * cosx2;
e3 = t3 - a3 * cosx3;
e4 = t4 - a4 * cosx4;
e5 = t5 - a5 * cosx5;
e6 = t6 - a6 * cosx6;

# summation rounding error, chained through the running partial sums.
esum = s - (t1 + t2 + t3 + t4 + t5 + t6);

err = s - s_exact;

{
  cosx1 in [-1,1] /\ cosx2 in [-1,1] /\ cosx3 in [-1,1] /\
  cosx4 in [-1,1] /\ cosx5 in [-1,1] /\ cosx6 in [-1,1] /\
  d1 in [-1b-53,1b-53] /\ d2 in [-1b-53,1b-53] /\ d3 in [-1b-53,1b-53] /\
  d4 in [-1b-53,1b-53] /\ d5 in [-1b-53,1b-53] /\ d6 in [-1b-53,1b-53]
  ->
  |err| in ?
}

err -> e1 + e2 + e3 + e4 + e5 + e6 + esum;
e1 -> (t1 - a1 * c1) + a1 * d1;
e2 -> (t2 - a2 * c2) + a2 * d2;
e3 -> (t3 - a3 * c3) + a3 * d3;
e4 -> (t4 - a4 * c4) + a4 * d4;
e5 -> (t5 - a5 * c5) + a5 * d5;
e6 -> (t6 - a6 * c6) + a6 * d6;
