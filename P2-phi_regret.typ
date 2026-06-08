#import "meta/gabri_notes.typ": *

#show: gabri_notes.with(lec_num: 3, date: none, title: "Ellipsoid against hope")



The techniques we have covered so far based on regret minimization lead to a running time complexity growing polynomially in $1\/eps$, where $epsilon > 0$ is the approximation quality of the equilibrium---be it a Nash equilibrium in zero-sum games or a (coarse) correlated equilibrium in general-sum games. Specifically, if we employ algorithms with ($Phi$-)regret bounded by $sqrt(T)$ as a function of the time horizon, we need $1\/eps^2$ iterations to reach an equilibrium. Can we do better in the regime where $epsilon$ is, say, exponentially small? The other main approach we have introduced for equilibrium computation is based on linear programming, which has the advantage of finding an _exact_ equilibrium. But, as we have already seen, the LP describing (C)CEs has a number of variables that scales _exponentially_ with the number of players.

= Ellipsoid against hope <sec:eah>

We will now introduce _ellipsoid against hope_ (EAH), a polynomial-time algorithm for computing $Phi$-equilibria even in games with many players and complex strategy sets. It was first introduced by #citet(<Papadimitriou08:Computing>). In what follows, we mostly follow the generalized version of the algorithm due to #citet(<Farina24:Polynomial>) and #citet(<Zhang25:Learning>). To be clear, this algorithm works in the centralized model, and it's not compatible with the framework of online learning, although as we shall see similarities do exist between the two approaches.

Our goal is to compute an _$epsilon$-$Phi$-equilibrium_ of a multilinear game, that is, a correlated distribution $mu in Delta(cX_1 times dots.c times cal(cX)_n)$ such that for any player $i in [n]$ and deviation function $phi_i in Phi_i$ mapping $cX_i -> cX_i$,

$
  EE_((vx_1, dots, vx_n) tilde mu) u_i (vx_1, dots, vx_n) >= EE_((vx_1, dots, vx_n) tilde mu) u_i (phi_i (vx_i), vx_(-i)) - epsilon.
$ <eq:phi-equil>



From a more abstract point of view, EAH deals with optimization problems of the form#footnote[We caution that $cX$ here corresponds to $cX_1 times dots.c times cX_n$ in the context of (@eq:phi-equil), even though we sometimes take $cX$ to be the strategy set of a single player in the context of regret minimization.]

$ "find " mu in Delta(cX) " such that " EE_(vx tilde mu) ip(vy, G(vx)) >= 0 quad forall vy in cY, $ <eq:EAH>

where $cX subset.eq RR^d, cY subset.eq RR^k$, and $G : cX -> RR^k$ is a function that can be evaluated efficiently. The crux in the optimization problem (@eq:EAH) lies in the fact that $mu$ resides in a high-dimensional space; a canonical case to think about is when $cX = cA_1 times dots.c times cA_n$ in a normal-form game, so that even describing a distribution $mu$ could require specifying $product_(i=1)^n |cA_i| - 1$ coordinates. We assume that $cY$, which corresponds to the set of deviations, admits a _separation oracle_; under mild geometric assumptions, it's equivalent to posit merely a _membership oracle_, which returns whether a point $vy in RR^k$ belongs to $cY$ or not. A special case is when $cY$ is a polytope described with a polynomial number of constraints, as is the case for swap regret in normal-form games---each player's set of deviations amounts to the set of stochastic matrices.

The key assumption in the EAH framework #cite(<Farina24:Polynomial>) is the existence of a _good-enough-response (GER)_ oracle, which, given any $vy in cY$, returns a point $vx in cX$ such that $ip(vy, G(vx)) >= 0$.

The EAH algorithm enables solving (@eq:EAH) with just a separation oracle for $cY$ and a GER oracle. The basic idea is to consider an $epsilon$-approximate version of the dual of (@eq:EAH),

$ "find " vy in cY " such that " ip(vy, G(vx)) <= - epsilon quad forall vx in cX. $ <eq:dual>

On account of the fact that a GER oracle exists, (@eq:dual) is guaranteed to be infeasible. Even so, EAH proceeds by executing the ellipsoid algorithm on that infeasible program---this is where the apt name "ellipsoid against hope" comes from. In every step $t$ of the algorithm, we have a candidate $vy^((t)) in cY$ and use the GER oracle to produce a point $x^((t)) in cX$ that refutes $vy^((t))$; in fact, an entire halfspace in $RR^k$. This goes on until the volume of the ellipsoid has shrank to a small enough amount. It then follows that

$ forall vy in cY exists t in [T] " such that " ip(vy, G(vx^((t)))) > - epsilon. $

Thus,

$ min_(vy in cY) max_(mu in Delta([T])) sum_(t=1)^T mu^((t)) ip(vy, G(vx^((t)))) > - epsilon. $

By the minimax theorem, we conclude that

$
  max_(mu in Delta([T])) min_(vy in cY) sum_(t=1)^T mu^((t)) ip(vy, G(vx^((t)))) > - epsilon.
$ <eq:compressed-zerosum>

In other words, there is a convex combination of $vx^((1)), dots, vx^((T))$ that refutes any possible $y in cY$. That convex combination, which is a _certificate of dual infeasibility_, will thus be an approximate solution to (@eq:EAH). The key point is that the resulting zero-sum game in (@eq:compressed-zerosum) is much smaller than the one we started with, and can be solved with standard techniques.

#theorem[#citep(<Farina24:Polynomial>)][
  Assuming the existence of a separation oracle for $cY$ and a GER oracle, EAH runs in time $"poly"(d, k, log(1\/eps))$ and returns an $epsilon$-approximate solution to (@eq:EAH).
] <thm:eah-general>

= Application to computing correlated equilibria

Let's now see to apply this algorithm to solve (@eq:phi-equil). As before, we assume that each $Phi_i$ contains linear functions of the form $vx_i |-> matM_i q_i (vx_i) in cX_i$ for some feature map $q_i: cX_i -> RR^(k_i)$. For these notes, we will assume that the set of valid matrices $matM_i$ form a convex, compact set $cY_i subset RR^(d_i times k_i)$; this assumption can be relaxed using a similar idea to the semi-separation oracle #cite(<Zhang25:Learning>). we will later briefly explain how to relax this assumption. Then a $Phi$-equilibrium is a distribution $mu$ such that

$
  sum_(i=1)^n EE_((vx_1, dots, vx_n) tilde mu) ip(matI_i - matM_i, u_i (vx_(-i)) times.o q(vx_i)) >= - epsilon quad forall i in [n], matM_i in cY_i,
$ <eq:simplified>

where $matI_i$ is the matrix for which $phi_(matI_i)$ is the identity map. This indeed adheres to (@eq:EAH). //, with the settings

// $
//   cX &= cX_1 times dots.c times cX_n, \
//   cY &= cY_1 times dots.c times cY_n, \
//   G(vx_1, ..., vx_n) &=
// $

// $
//   EE_((vx_1, dots, vx_n) tilde mu) ip(vx_i - matM_i vx_i, vu_i (vx_(-i))) = EE_((vx_1, dots, vx_n) tilde mu) ip(matI - matM_i, vu_i (vx_(-i)) times.o vx_i).
// $

What's left is to prove that (@eq:simplified) admits an efficient GER oracle. To do so, let's consider any $i in [n]$ and deviation $matM_i in cY_i$. Taking $mu_i in Delta(cX_i)$ to be an $eps$-approximate expected fixed point of $phi_(matM_i)$, we have

$ EE_((vx_1, dots, vx_n) tilde mu) ip(vx_i - matM_i q(vx_i), u_i (vx_(-i))) >= - eps dot.c norm(u_i (vx_(-i)))_2, $

But this is not immediately good enough. Recall that our algorithm for expected fixed points, from `@thm:efp` #todo[how to cite a theorem from a different file?], has $"poly"(1\/eps)$ runtime. Since our goal in this section is to construct algorithms with $log(1\/eps)$ runtime, we need to do better. Fortunately, there _is_ a $"poly"(d, log(1\/eps))$-time algorithm for semi-separation with expected fixed points:

#theorem[#citep(<Zhang25:Learning>)][
  There is a $"poly"(d, log(1\/eps))$-time algorithm for semi-separation.
]
#proofsketch[
  We use EAH. Given $phi : cX -> RR^d$, consider the EAH problem with $cY$ set to the unit $ell_2$-ball, and $G(vx) = phi(vx) - vx$. Then, by @thm:eah-general, it suffices to implement a GER oracle. That is, given $vy in RR^d$, we need to find $vx$ with $ip(vy, phi(vx) - vx) >= 0$. But this is easy: we can simply take $vx in argmin_(hat(vx) in cX) ip(vy, hat(vx))$. Then either $phi(vx) in.not cX$, in which case we output $vx$ as a certificate of infeasibility of $phi$, or $phi(vx) in cX$, in which case $ip(vy, phi(vx) - vx) >= 0$ by definition of $vx$.
]
Finally, if $mu_i in Delta(cX_i)$ is an expected fixed point of $phi_i$ for every $i$, then the product distribution $mu_1 times dots.c times mu_n$ is a GER solution satisfying (@eq:simplified). We therefore have:

#theorem[#citep(<Farina24:Polynomial>), #citep(<Zhang25:Learning>) #todo[is it possible to do `#cite[one, two]`, like in latex?]][
  If for each player $i in [n]$ in a multilinear game the set of deviations $Phi_i$ admits a separation oracle, there is an algorithm polynomial in $log(1\/epsilon)$, $n$, $d_1, ..., d_n$, and $k_1, ..., k_n$ that outputs an $epsilon$-$Phi$-equilibrium.
] <thm:eah-eqm>

We conclude this chapter with several remarks about the previous theorem.

1. _Nested EAH._ The algorithm that we derive for @thm:eah-eqm is a doubly-nested ellipsoid against hope algorithm: the outer EAH algorithm computes the equilibrium itself, and on every iteration, EAH is invoked for each player $i$ to compute an expected fixed point for $i$.

2. _Necessity of linearity._
  The results presented in this document assume, and use, the fact that utilities are multilinear. The multilinearity assumption is used twice:
  #set enum(numbering: "a.")

  + in the GER oracle, to calculate expected utilities under a product distribution $mu = mu_1 times dots.c times mu_n$ (which has support that increases exponentially with the number of players $n$); and

  + to ensure that an expected fixed point solution actually guarantees separation: for example, if instead we had arbitrary concave utilities $u(dot.c, vx_(-i))$, then we would need the condition $ EE_(vx ~ mu) ip(grad u(vx), phi(vx) - vx) >= -eps $ for every convex function $u$. Expected fixed points do not guarantee this.

  The first issue is not circumventable, and is generally a barrier to efficient equilibrium computation in concave games with many players.
  The second issue is not circumventable in general, as it would imply the ability to compute fixed points of contraction maps. However, it _is_ circumventable in the special case where $u(dot.c)$ is _quadratic_ concave for each player $i$; therefore, it is possible to compute $Phi$-equilibria in quadratic games with a constant number of players. The required techniques, however, are beyond the scope of this document. We refer the interested reader to our recent preprint on this topic #cite(<Anagnostides26:Complexity>).
3. _Non-separable sets $Phi$_. So far in this section, we have assumed that the sets $cY_i$ admit efficient separation oracles. However, this is not a necessary assumption. As it turns out, like we did for regret minimization, it is sufficient to take $cY_i$ to be a _superset_ of the valid deviations and rely on the semi-separation oracle to declare when a given matrix $matM_i in cY_i$ is invalid. Therefore, @thm:eah-eqm also generalizes to the case where $cY_i$ only admits an efficient _semi_-separation oracle, such as the sets $Phi^q$ discussed in the regret minimization section. For more details, we refer the interested reader to our paper #cite(<Zhang25:Learning>).



// #colbreak()
#lec_bibliography("../meta/refs.bib")
