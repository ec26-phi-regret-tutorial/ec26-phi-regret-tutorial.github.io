#import "meta/gabri_notes.typ": *

#show: gabri_notes.with(
  lec_num: 6,
  date: none,
  strtitle: "Profile Swap Regret, Manipulability, Response-Based Approachability",
  title: [Profile Swap Regret, Manipulability, and\ Response-Based Approachability],
)

In this chapter, we introduce the notion of _profile swap regret_, which lies in between linear swap regret and full swap regret. A key property of profile swap regret is that it guarantees _non-manipulability_---in a sense that will be formalized soon---against a dynamic optimizer. What's more, there is an efficient algorithm for minimizing profile swap regret. This notion was recently introduced by~#citet(<Arunachaleswaran25:Profile>), who also provided the first efficient algorithm. In what follows, we present the approach of #citet(<Anagnostides26:Swap>), which in turn relies on the response-based approachability algorithm of #citet(<Bernstein15:Approachability>).#footnote[We point the interested reader to https://sites.google.com/view/strategic-learning-ec25 for an adjacent workshop at EC '25. ]

= Setup

We operate in the usual online linear optimization setting. The learner picks strategies from a convex and compact strategy set $cX subset RR^d$. At every round $t in [T]$, the learner first selects a strategy $vx^((t)) in cX$; the adversary selects a utility vector $vu^((t)) in cal(U) subset.eq RR^d$; then the learner receives utility $ip(vx^((t)), vu^((t)))$ and observes $vu^((t))$ as feedback. We make the standard normalization assumption $abs(ip(vx, vu)) <= 1$ for all $vx in cX, vu in cal(U).$ In what follows, for a utility vector $vu in cal(U)$, we define the best-response map $b(vu) in argmax_(vx in cX) ip(vx, vu),$ with ties broken arbitrarily.


*Linear swap regret.* We first recall the benchmark of linear swap regret. An affine map $phi: cX -> RR^d$ is an _endomorphism_ of $cX$ if $phi(cX) subset.eq cX$. We denote by $"End"(cX)$ the set of affine endomorphisms. We write each $phi in "End"(cX)$ as $phi(vx) = matM vx + va.$ The _linear swap regret_ of the learner after $T$ rounds is

$
  "LinearSwapReg"^((T))
  :=
  max_(phi in "End"(cX))
  sum_(t=1)^T ip(phi(vx^((t))), vu^((t)))
  -
  sum_(t=1)^T ip(vx^((t)), vu^((t))).
$


This is a natural relaxation of (full) swap regret: instead of allowing arbitrary maps, the performance of the learner is compared against affine endomorphisms. In this case, the algorithm of #citet(<Gordon08:No>) reduces linear swap regret to external regret over the $d(d+1)$-dimensional set of deviations. This implies an online algorithm with linear swap regret growing as $O(d sqrt(T))$, which is information-theoretically optimal~#citep(<Anagnostides26:Swap>). On the other hand, the algorithm of~#citet(<Gordon08:No>) is computationally inefficient~#citep(<Daskalakis25:Efficient>). This lecture will cover an approachability framework that circumvents this computational barrier.

*Correlated strategy profiles.* A basic object in the approachability formulation is the _correlated strategy profile (CSP)_. Specifically, for a realized pair $(vx^((t)), vu^((t)))$, we define $kappa^((t)) := (vu^((t)) times.o vx^((t)), vu^((t))) in RR^(d times (d+1)).$ The average CSP is

$ overline(kappa)^((T)) := 1/T sum_(t=1)^T kappa^((t)). $ <eq:kappa-bar>


For a CSP $kappa = (matK, vq)$ and an affine map $phi(vx)=matM vx + va$, we use the shorthand notation $ip((matM, va), kappa) := ip(matM, matK) + ip(va, vq).$ With this notation at hand, linear swap regret can be rewritten as

$
  "LinearSwapReg"^((T))
  =
  T max_(phi=(matM,va) in "End"(cX))
  ip((matM, va) - (matI, 0), overline(kappa)^((T))).
$ <eq:linear-swap-csp>

Using this reformulation, we will shortly see how to reduce minimizing linear swap regret to a suitable approachability problem.

= Profile swap regret and non-manipulability

We now define the notion of _profile swap regret_ and discuss its non-manipulability properties. To do so, we define the instantaneous regret as $R(vx, vu) := max_(vx^* in cX) ip(vx^* - vx, vu).$ Equivalently, we can write $R(vx, vu) = ip(b(vu) - vx, vu).$ A valid decomposition of $overline(kappa)^((T))$ can be written as
$ overline(kappa)^((T)) = sum_(j=1)^m lambda_j (vu_j times.o vx_j), $
where $lambda_j >= 0$, $sum_(j=1)^m lambda_j = 1$, $vx_j in cX$, and $vu_j in cal(U)$.

#definition("Profile swap regret")[
  The _profile swap regret_ of a CSP $overline(kappa)^((T))$ is
  $
    "ProfileSwapReg"^((T))(overline(kappa)^((T)))
    :=
    T min_((lambda_j, vx_j, vu_j)_j)
    sum_j lambda_j R(vx_j, vu_j),
  $ <eq:profile-swap-reg>
  where the minimum is over all valid convex decompositions of $overline(kappa)^((T))$.
]

The definition is admittedly somewhat cumbersome, but as we shall see it unlocks certain powerful properties. This is a good time to discuss the notion of non-manipulability due to~#citet(<Deng19:Strategizing>).

== Manipulability against a dynamic optimizer

We consider a two-player game between a _learner_ and an _optimizer_. The learner follows some learning algorithm, whereas the optimizer may choose its actions strategically over time after observing how the learner updates. We let $u_ell$ be the learner's utility function and $cal(X)$ its strategy set. Similarly, we let $u_o$ be the optimizer's utility function and $cal(Y)$ its strategy set.


The key relevant benchmark in this setting is the optimizer's _Stackelberg value_, which is the best utility the optimizer can guarantee by committing to a fixed strategy $vy in cY$, while the learner best-responds to that strategy (with ties broken in favor of the optimizer):

$
  "Stack"(u_o)
  :=
  max_(vy in cY)
  max_(vx in "BR"(vy))
  u_o (vx, vy),
$

where $"BR"(vy) := argmax_(vx in cX) u_ell (vx, vy).$ The Stackelberg value is what the optimizer can guarantee by means of a _static_ commitment. Informally speaking, a learning algorithm is said to be manipulable if the optimizer can do strictly better than its Stackelberg value by using a _dynamic_ strategy: instead of playing a fixed strategy $vy$, the optimizer chooses a sequence $vy^((1)), vy^((2)), dots$ so as to steer the learner's future play toward more favorable outcomes. (It's interesting to note that this steering problem can be computationally intractable~#citep(<Assos24:Maximizing>).) Equivalently, even if the optimizer fully understands the learner's algorithm and chooses its strategies dynamically to influence future play, it cannot asymptotically obtain more than what it could already obtain by committing to a fixed strategy.

A beautiful connection crystallized by #citet(<Arunachaleswaran25:Profile>) reassures us that a learning algorithm that has vanishing profile swap regret cannot be manipulated by a dynamic optimizer.

#proposition[
  If a learning algorithm guarantees $"ProfileSwapReg"^((T)) = o(T)$, it is non-manipulable.
] <prop:profile-nomanip>


= Reducing to an approachability problem

Having introduced and motivated the notion of profile swap regret, we now reduce it to an approachability instance. We define $cal(K)
:=
conv { (vu times.o vx, vu) : vu in cal(U), vx in cX }$ and $cal(S)
:=
conv { (vu times.o b(vu), vu) : vu in cal(U) }.$ We measure approachability loss by

$ "AppLoss"^((T)) := min_(vs in cal(S)) norm(overline(kappa)^((T)) - vs)_F. $ <eq:app-loss>

We first point out that minimizing linear swap regret reduces to this approachability instance.

#lemma[#citep(<Anagnostides26:Swap>)][
  For any time $T in NN$,
  $
    "LinearSwapReg"^((T))
    <=
    2 T "AppLoss"^((T))
    dot.c
    max_(phi in "End"(cX)) norm(phi)_F.
  $ <eq:lin-swap-app-loss>
] <lem:linear-swap-approach>

#proof[
  Let $vs in cal(S)$ be closest to $overline(kappa)^((T))$. Since $vs$ is, by definition, a convex combination of points of the form $(vu times.o b(vu), vu)$ and $b(vu)$ maximizes $ip(dot.c, vu)$ over $cX$, every endomorphism $phi in "End"(cX)$ satisfies
  $
    ip(phi - (matI, 0), vs) <= 0.
  $
  Therefore, using (@eq:linear-swap-csp),
  $
    "LinearSwapReg"^((T))
    <=
    T max_(phi in "End"(cX))
    ip(phi-(matI,0), overline(kappa)^((T)) - vs).
  $
  The claim now follows from Cauchy-Schwarz.
]

What's more, the same target set also captures profile swap regret. The following lemma follows essentially by the definition of profile swap regret.

#lemma[#citep(<Anagnostides26:Swap>)][
  The profile swap distance of $overline(kappa)^((T))$ is equal to $"AppLoss"^((T))$.
] <lem:profile-swap-distance>

The profile swap distance of a CSP $overline(kappa)^((T))$ is the infimum Euclidean distance of $overline(kappa)^((T))$ from a CSP with zero profile swap regret. As a result, we conclude that minimizing the approachability loss simultaneously circumscribes both linear swap regret and profile swap regret.

= Response-based approachability

The key observation now is that the induced approachability problem can be solved efficiently through the response-based approachability framework of~#citet(<Bernstein15:Approachability>). This was recently leveraged by~#citet(<Anagnostides26:Swap>).

#pseudocode-list(
  booktabs: true,
  title: [*Algorithm* #citep(<Bernstein15:Approachability>): Response-based approachability],
)[
  - *Input:* Horizon $T$, sets $cX, cal(U)$, best-response map $b : cal(U) -> cX$
  - Initialize $matU^((0)) := 0 in RR^(d times (d+1))$
  - *for each* round $t = 1, dots, T$ *do*
    - Compute maximin strategies $(vx^((t)), vu_*^((t)))$ for
      $
        max_(vx in cX) min_(vu in cal(U))
        ip(matU^((t-1)), (vu times.o vx, vu))
      $
    - Set $vs^((t)) := (vu_*^((t)) times.o b(vu_*^((t))), vu_*^((t)))$
    - Play $vx^((t))$ and observe $vu^((t))$
    - Set $kappa^((t)) := (vu^((t)) times.o vx^((t)), vu^((t)))$
    - Update $matU^((t)) := matU^((t-1)) + kappa^((t)) - vs^((t))$
]


The algorithm proceeds by maintaining the accumulated displacement

$ matU^((t)) := sum_(tau=1)^t (kappa^((tau)) - vs^((tau))) in RR^(d times (d+1)). $

At every round $t$, it solves the bilinear zero-sum game

$
  max_(vx in cX) min_(vu in cal(U))
  ip(matU^((t-1)), (vu times.o vx, vu)).
$ <eq:bs-game>

Let $(vx^((t)), vu_*^((t)))$ be a pair of minimax strategies for this game. The algorithm then sets

$ vs^((t)) := (vu_*^((t)) times.o b(vu_*^((t))), vu_*^((t))) in cal(S). $

Finally, it plays the strategy $vx^((t))$, whereupon it observes the utility $vu^((t))$, forms the induced $kappa^((t))$, and updates the cumulative displacement $matU^((t))$.
//
//This response-based approachability algorithm is summarized below.
All steps can be efficiently implemented through oracle access to $cal(X)$.


The proof of correctness crucially relies on the minimax theorem, as we formalize below.

#lemma[#citep(<Bernstein15:Approachability>)][
  For every utility $vu^((t)) in cal(U)$, we have
  $
    ip(matU^((t-1)), kappa^((t)) - vs^((t))) >= 0.
  $ <eq:bs-invariant>
] <lem:bs-invariant>

#proof[
  Let
  $
    g_t (vx, vu) := ip(matU^((t-1)), (vu times.o vx, vu)).
  $
  By the minimax theorem,
  $
    max_(vx in cX) min_(vu in cal(U)) g_t (vx, vu)
    =
    min_(vu in cal(U)) max_(vx in cX) g_t (vx, vu).
  $
  Thus,
  $
    g_t (vx^((t)), vu^((t)))
    >=
    min_(vu in cal(U)) max_(vx in cX) g_t (vx, vu).
  $
  Since $vu_*^((t))$ is minimax optimal and $b(vu_*^((t)))$ is a feasible strategy in $cX$,
  $
    min_(vu in cal(U)) max_(vx in cX) g_t (vx, vu)
    >=
    g_t (b(vu_*^((t))), vu_*^((t))).
  $
  Combining the two inequalities gives
  $
    ip(matU^((t-1)), kappa^((t)))
    >=
    ip(matU^((t-1)), vs^((t))),
  $
  which is exactly (@eq:bs-invariant).
]

The invariant implies approachability by the usual Pythagorean argument.

#lemma("Pythagorean lemma")[
  Let $vz^((1)), dots, vz^((T))$ be vectors with $norm(vz^((t)))_2 <= B$ for all $t$. If
  $
    ip(sum_(tau=1)^(t-1) vz^((tau)), vz^((t))) <= 0
  $
  for every $t$, then
  $
    norm(sum_(t=1)^T vz^((t)))_2 <= B sqrt(T).
  $
] <lem:pythagorean>

Applying @lem:pythagorean with $vz^((t)) = vs^((t)) - kappa^((t))$ gives

$
  "AppLoss"^((T))
  <=
  norm(1/T sum_(t=1)^T (kappa^((t)) - vs^((t))))_F
  <=
  (2 B) / sqrt(T),
$ <eq:app-loss-rate>

where $B := max_(vx in cX, vu in cal(U)) norm((vu times.o vx, vu))_F.$ We thus arrive at the following theorem.

#theorem[#citep(<Anagnostides26:Swap>)][
  For any sequence of utilities $vu^((1)), dots, vu^((T)) in cal(U)$, the response-based approachability algorithm guarantees
  $
    "LinearSwapReg"^((T))
    <=
    4 sqrt(T)
    (max_(vx in cX, vu in cal(U)) norm(vu)_2 sqrt(norm(vx)_2^2 + 1))
    (max_(phi in "End"(cX)) norm(phi)_F),
  $ <eq:BS-linear-bound>
  and
  $
    "ProfileSwapDist"^((T))
    <=
    2 / sqrt(T)
    (max_(vx in cX, vu in cal(U)) norm(vu)_2 sqrt(norm(vx)_2^2 + 1)).
  $ <eq:BS-profile-bound>
] <thm:BS-bound>


Through suitable preconditioning, it can be shown that this algorithm guarantees optimal linear swap regret~#citep(<Anagnostides26:Swap>). We further remark that this response-based framework can be extended to nonlinear deviations; we refer the interested reader to~#citet(<Anagnostides26:Swap>).

#lec_bibliography("../meta/refs.bib")
