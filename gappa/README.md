# Gappa proof: rounding error of a truncated trigonometric series

Models evaluating one coordinate's periodic correction in a truncated
VSOP87 (planetary) / ELP2000-82B (lunar) style theory:

$$ s = \sum_{i=1}^{6} a_i \cos(x_i) \quad \text{(f64, Horner-order summation)} $$

against the exact real-valued sum, isolating the cosine implementation's
own error ($d_i$) as a free parameter so the proof doesn't need to resolve
argument-range reduction.

## Results

Writing the error as $e = s - s_{exact}$:

| script              | cos error model                                            | proved bound on $\lvert e \rvert$ |
|---------------------|-------------------------------------------------------------|------------------------------------|
| series_sum_caseA.g  | correctly rounded (<=0.5 ulp, e.g. metallic-rs/CORE-MATH)  | $5.63 \times 10^{-17}$ rad ($\approx 2^{-53.98}$) |
| series_sum_caseB.g  | typical libm (<=1 ulp)                                     | $7.00 \times 10^{-17}$ rad ($\approx 2^{-53.67}$) |

Both run: `nix shell nixpkgs#gappa --command gappa <file>.g`

## Takeaway

A 6-term truncated series carries a *modeling* error on the order of
1 arcminute ($\approx 2.9 \times 10^{-4}$ rad) by construction (that's
the truncation threshold `cosinekitty/astronomy` and `pracstro` both
target). The proved f64 *rounding* error here is ~13 orders of
magnitude smaller than that, regardless of whether the cosine
implementation is correctly rounded or just typical 1-ulp libm
quality. So: floating-point rounding is not the accuracy bottleneck
for this class of algorithm. The choice of how many series terms to
keep is what actually sets the accuracy budget, not the arithmetic.
Correctly-rounded trig (metallic-rs) buys a small, provable
rounding-error reduction, but it does not change the accuracy class
of the overall ephemeris -- so it's worth using where free (fixed-cost
link), but not worth re-deriving the lunar theory truncation around it.

## Technique note

The naive goal

$$ \{\, \text{hyps} \rightarrow \lvert s - s_{exact} \rvert \in\ ? \,\} $$

with no hints gives a useless bound ($\approx 0.247$ rad -- larger
than the signal). Gappa's automatic interval propagation can't
correlate `s` and `s_exact` through six shared variables each passing
through its own rounding operator. The fix: decompose into one
rewrite hint per term ($e_i \rightarrow (t_i - a_i c_i) + a_i d_i$)
plus one for the summation error, so each hint only needs Gappa's
single-rounding-operator theorems (product rounding, sum rounding)
rather than reasoning about the whole expression at once. This
mirrors Gappa's own worked example ($x(1-x)$), which needs an
algebraic rewrite hint for the same reason.
