#import "meta/gabri_notes.typ": *

#show: gabri_notes.with(lec_num: 1, date: none, title: "Ellipsoid and semi-separation oracle")

#import "figures/phi-fixed-point.typ": body as phi_fixed_point

#figure(caption: [#todo[Add caption]], scale(80%, phi_fixed_point))

= The ellipsoid against hope algorithm: solving the correlated equilibrium LP in multi-player games <sec:eah>

The techniques we have covered based on regret minimization lead to a running time complexity growing polynomially in $1/epsilon$, where $epsilon > 0$ is the approximation quality of the equilibrium---be it a Nash equilibrium in zero-sum games or a (coarse) correlated equilibrium in general-sum games. Specifically, if we employ algorithms with ($Phi$-)regret bounded by $sqrt(T)$ as a function of the time horizon, we need $1/epsilon^2$ iterations to reach an equilibrium. Can we do better in the regime where $epsilon << 1$? The other main approach we have introduced for equilibrium computation is based on linear programming, which has the advantage of finding an _exact_ equilibrium. But, as we saw in the last lecture, the LP describing (C)CEs has a number of variables that scales _exponentially_ with the number of players.

We will now introduce _ellipsoid against hope_ (EAH), a polynomial-time algorithm for computing (C)CEs even in multi-player games. It was first introduced by #citet(<Papadimitriou08:Computing>); in what follows, we mostly follow the generalized version of the algorithm due to #citet(<Farina24:Polynomial>). To be clear, this algorithm works in the centralized model, and it's not compatible with the framework of online learning, although similarities do exist between the two approaches as we shall see.

Our goal is to compute an _$epsilon$-$Phi$-equilibrium_ of a multilinear game; that is, a correlated distribution $mu in Delta(cX_1 times dots.c times cal(cX)_n)$ such that for any player $i in [n]$ and deviation function $phi_i in Phi_i$ mapping $cX_i -> cX_i$,

$
  EE_((vx_1, dots, vx_n) tilde mu) u_i (vx_1, dots, vx_n) >= EE_((vx_1, dots, vx_n) tilde mu) u_i (phi_i (vx_i), vx_(-i)) - epsilon.
$ <eq:phi-equil>

*The ellipsoid against hope framework.* From a more abstract point of view, EAH deals with optimization problems of the form#footnote[We caution that $cX$ here corresponds to $cX_1 times dots.c times cX_n$ in the context of @eq:phi-equil, even though we usually take $cX$ to be the strategy set of a single player in the context of regret minimization.]

$ "find " mu in Delta(cX) " such that " EE_(vx tilde mu) ip(vy, G(vx)) >= 0 quad forall vy in cY, $ <eq:EAH>

where $cX subset.eq RR^d, cY subset.eq RR^k$, and $G : cX -> RR^k$ is a function that can be evaluated efficiently. The crux in the optimization problem @eq:EAH lies in the fact that $mu$ resides in a high-dimensional space; a canonical case to think about is when $cX = cA_1 times dots.c times cA_n$ in a normal-form game, so that even describing a distribution $mu$ could require specifying $product_(i=1)^n |cA_i| - 1$ coordinates. We assume that $cY$, which corresponds to the set of deviations, admits a _separation oracle_; under mild geometric assumptions, it's equivalent to posit merely a _membership oracle_, which returns whether a point $vy in RR^k$ belongs to $cY$ or not. A special case is when $cY$ is a polytope described with a polynomial number of constraints, as is the case for swap regret in normal-form games---each player's set of deviations amounts to the set of stochastic matrices. The key assumption in the EAH framework #citep(<Farina24:Polynomial>) is the admission of a _good-enough-response (GER)_ oracle, which, _given_ any $vy in cY$, returns a point $vx in cX$ such that $ip(vy, G(vx)) >= 0$.

The EAH algorithm enables solving (@eq:EAH) with just a separation oracle for $cY$ and a GER oracle. The basic idea is to consider an $epsilon$-approximate version of the dual of (@eq:EAH),

$ "find " vy in cY " such that " ip(vy, G(x)) <= - epsilon quad forall vx in cX. $ <eq:dual>

On account of the fact that a GER oracle exists, (@eq:dual) is guaranteed to be infeasible. Even so, EAH proceeds by executing the ellipsoid algorithm on that infeasible program---this is where the apt name "ellipsoid against hope" comes from. In every step $t$ of the algorithm, we have a candidate $vy^((t)) in cY$, and use the GER oracle to produce a point $x^((t)) in cX$ that refutes $vy^((t))$; in fact, an entire halfspace in $RR^k$. This goes on until the volume of the ellipsoid has shrank to a small enough amount. It then follows that

$ forall vy in cY exists t in [T] " such that " ip(vy, G(vx^((t)))) > - epsilon. $

Thus,

$ min_(vy in cY) max_(mu in Delta([T])) sum_(t=1)^T mu^((t)) ip(vy, G(vx^((t)))) > - epsilon. $

By the minimax theorem, we conclude that

$
  max_(mu in Delta([T])) min_(vy in cY) sum_(t=1)^T mu^((t)) ip(vy, G(vx^((t)))) > - epsilon.
$ <eq:compressed-zerosum>

In other words, there is a convex combination of $vx^((1)), dots, vx^((T))$ that refutes any possible $y in cY$. That convex combination, which is a _certificate of dual infeasibility_, will thus be an approximate solution to (@eq:EAH). The key point is that the resulting zero-sum game in (@eq:compressed-zerosum) is much smaller than the one we started with, and can be solved with standard LP techniques.

#theorem[#citep(<Farina24:Polynomial>)][
  Assuming the existence of a separation oracle for $cY$ and a GER oracle, EAH runs in time $"poly"(d, k, log(1/epsilon))$ and returns an $epsilon$-approximate solution to (@eq:EAH).
]

Let's now see to apply this algorithm to solve (@eq:phi-equil). We assume that each $Phi_i$ contains linear functions of the form $vx_i |-> matM_i vx_i in cX_i subset.eq RR^(m_i)$, where the set of valid matrices $M_i$ is a polytope $cY_i$ with a polynomial number of variables and constraints. (For example, in the example covered in the last lecture pertaining to swap regret in normal-form games, the set of column stochastic matrices is a Cartesian product of probability simplices.) We assume further that $Phi_i$ contains the identity matrix. Then (@eq:phi-equil) can be expressed as

$
  EE_((vx_1, dots, vx_n) tilde mu) ip(matI - matM_i, u_i (vx_(-i)) times.o vx_i) >= - epsilon quad forall i in [n], matM_i in cY_i,
$ <eq:simplified>

since

$
  EE_((vx_1, dots, vx_n) tilde mu) ip(vx_i - matM_i vx_i, vu_i (vx_(-i))) = EE_((vx_1, dots, vx_n) tilde mu) ip(matI - matM_i, vu_i (vx_(-i)) times.o vx_i).
$

This indeed adheres to (@eq:EAH). What's left is to prove that (@eq:simplified) admits a GER oracle. To do so, let's consider any $i in [n]$ and deviation $matM_i in cY_i$. Taking $vx_i in cX_i$ to be a _fixed point_ of $matM_i$, we have

$ EE_((vx_1, dots, vx_n) tilde mu) ip(vx_i - matM_i vx_i, u_i (vx_(-i))) = 0, $

as desired.

#theorem[#citep(<Farina24:Polynomial>)][
  If for each player $i in [n]$ in a multilinear game the set of deviations $Phi_i$ admits a separation oracle, there is an algorithm polynomial in $log(1/epsilon)$, $n$, and $m = max_(1 <= i <= n) m_i$ that outputs an $epsilon$-$Phi$-equilibrium.
]

Overall, we find that EAH and the framework of $Phi$-regret have several conceptual similarities. Namely, they both operate over the set of deviations, returning in each round a fixed point of the corresponding deviation. Further, the correlated distribution they output is given as a mixture of product distributions. This ensures that the correlated distribution admits an efficient, compact representation. The final similarity is that they both require a separation oracle for the set of deviations, which brings us to the final topic of this lecture.

= Semi-separation oracle <sec:semi-separation>

The main requirement in both EAH and the framework of #citet(<Gordon08:No>) is a separation oracle for the set of deviations. Indeed, focusing on minimizing external regret, a separation oracle is enough to efficiently implement an algorithm such as "FTRL"; conversely, any regret minimizer should be able to perform separation; otherwise, there would be a fixed utility exerting high regret.

However, even when $Phi$ contains only linear functions mapping $cX -> cX$, it can be NP-hard to implement a membership oracle, as shown recently by #citet(<Daskalakis25:Efficient>). Does this mean that minimizing $Phi$-regret or applying the EAH framework is intractable? Not so. While so far we have insisted on having _both_ a separation oracle for the set of deviations _and_ an expected fixed-point oracle (that is, a good-enough-response oracle), it turns out that it's enough to implement their _disjunction_; this was coined a _semi-separation_ oracle by #citet(<Daskalakis25:Efficient>) (@def:semiseparation).

We will introduce this notion in the general setting where the set $Phi$ comprises nonlinear functions expressed in the following form.

#definition[#citep(<Zhang25:Learning>)][
  Given a map $psi : cX -> RR^(k')$, the set of deviations $Phi^psi$ is the set of all maps $phi: cX -> cX$ that can be expressed as the matrix-vector product $matK(phi) psi(vx) + vc(phi)$ for some matrix $matK in RR^(d times k')$ and vector $vc in RR^d$. We denote by $k = d times k' + d$ the dimension of $(matK, vc)$.
] <def:low-degree>

One can think of $psi$ as a _feature mapping_; a natural example captured by @def:low-degree is _low-degree polynomials_, where $psi$ contains all $<= ell$-wise products of entries in $x$. Minimizing regret with respect to the set describing all valid $(matK, vc)$ is daunting, but it turns out that this is not necessary. One can instead solve the following simpler problem; the reason why this is sufficient is not covered in these notes.

#definition("Semi-separation")[
  In the _semi-separation_ problem we are given a function $phi : cX -> RR^d$ and we have to compute
  - _either_ an $epsilon$-expected fixed point $mu in Delta(cX)$ of $phi$,
  - _or_ a point $vx in cX$ such that $phi(vx) in.not cX$.
] <def:semiseparation>

Implementing the semi-separation oracle is actually not hard once we have an algorithm for computing expected fixed points. One can basically run the same algorithm: if during the execution of the algorithm we identify a point such that $phi(vx) in.not cX$, we can terminate as we have a certificate that $phi$ is not a valid mapping. Otherwise, the algorithm will be able to find an expected fixed point of $phi$, _even though $phi$ may not necessarily be a valid transformation_. In other words, the basic idea in the recent framework of #citet(<Daskalakis25:Efficient>) is to make allowance for functions that do not necessarily map $cX$ to $cX$. To conclude, we state the main implication for minimizing $Phi^(psi)$-regret per @def:low-degree.

#theorem[
  There is an algorithm, based on EAH, that computes an $epsilon$-$Phi^psi$-equilibrium of any multilinear $n$-player game and has complexity $"poly"(n, m, k, log(1/epsilon))$. Furthermore, in the online learning setting, there is an efficient algorithm with regret bounded as $"poly"(m, k) sqrt(T)$ after $T$ rounds.
]



// #colbreak()
#lec_bibliography("../meta/refs.bib")
