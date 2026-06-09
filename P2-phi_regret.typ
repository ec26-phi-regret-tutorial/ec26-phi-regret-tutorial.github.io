#import "meta/gabri_notes.typ": *

#show: gabri_notes.with(lec_num: 2, date: none, title: "Beyond Normal Form")

#show "TreeSwap": `TreeSwap`



In this section, we will show how to construct $Phi$-regret minimizers for strategy sets $cX$ that are more complex than the simplex. To do so, we first define the sets of deviations that we work with.

#definition[#citep(<Zhang25:Learning>)][
  Given a feature map $q : cX -> RR^(k)$, the set of deviations $Phi^q$ is the set of all maps $phi_matM: cX -> cX$ that can be expressed as the matrix-vector product $matM(phi) q(vx)$ for some matrix $matM in RR^(d times k)$ and vector $vc in RR^d$.
] <def:low-degree>

We will assume without loss of generality that $Phi^q$ contains the identity map.

#example[
  If $q(vx) = (1, vx) in RR^(d+1)$, then $Phi^q$ is the set of all linear maps $phi : cX -> cX$. More generally, if $q(vx) = (1, vx)^(times.o ell) in RR^k$ where $k = binom(d, <= ell)$ is the set of all monomials in $vx$ up to degree $ell$, then $Phi^q$ is the set of all degree-$ell$ polynomials in $vx$.
]
In general, we will describe algorithms that have complexity polynomial in the dimension $k$. Note that, in the special case of degree-$ell$ polynomials, these algorithms will therefore run in time polynomial in $d^ell$.

In these settings, we run into two fundamental issues when trying directly to generalize the GGM construction. Recall that the GGM construction requires two ingredients. We will describe why both ingredients are problematic.

1. _Fixed-point oracle_: Finding a fixed point of a continuous function $phi : cX -> cX$ is $PPAD$-complete, and therefore is unlikely to be possible efficiently. Even more generally, if $phi$ contains _discontinuous_ maps, $phi$ may not even admit any approximate fixed points!

2. _External regret minimizer for $Phi^q$_: External regret minimization implies the ability to (approximately) perform linear optimization over $Phi^q$. But linear optimization over $Phi^q$ can be hard, in two ways:

  #set enum(numbering: "a.")

  + If $Phi^q$ contains nonlinear functions, then that this optimization problem is $NP$-hard even when $cX$ is the hypercube $[0, 1]^d$ #citep(<Zhang24:Efficient>).

  + Even if $Phi^q$ is the set of linear endomorphisms, if $cX$ itself is only available via an oracle (e.g., a separation oracle), then optimizing a linear function over $Phi^q$ can in general be hard  #citep(<Daskalakis25:Efficient>).

#remark[
  When $cX$ is represented by an explicit set of linear constraints, i.e., $cX = { vx : matA vx <= vb }$, the set of linear endomorphisms can also be expressed as an explicit set of linear constraints #citep(<Zhang25:Expected>). This is true, for instance, for _extensive-form games_. Thus, in this particular case, linear regret minimization can be accomplished without all the machinery of this section.
]

#remark[
  For extensive-form games in particular, the set of linear endomorphisms has a clean game-theoretic interpretation #citep(<Zhang23:Mediator>). We will not detail it here, though.
]

We will show how to circumvent both of the above issues. We start with the fixed point computation.

= Expected fixed points

Given that computing a fixed point is generally hard (when $phi$ is continuous but nonlinear) or even impossible (when $phi$ is discontinuous), we need a different notion of fixed point. Fortunately, by examining the proof of the Gordon--Greenwald--Marks framework, it turns out that a weaker condition suffices.

#definition[Expected fixed point #citep(<Zhang24:Efficient>)][
  Given a function $phi : cX -> cX$, an $eps$-approximate _expected fixed point_ of $phi$ is a distribution#footnote[Throughout this document, to avoid measure-theoretic issues, we only consider finite-support distributions.] $mu in Delta(cX)$ such that $ norm(EE_(vx ~ mu) [ phi(vx) - vx ])_2 <= eps. $
]

As we will see shortly, expected fixed points fix both problems with fixed point computation: an (approximate) expected fixed point always exists, and moreover, there is an efficient algorithm for computing one!

#theorem[Expected fixed point computation][
  There is an algorithm with runtime $"poly"(d, 1\/eps)$ that, given query access to $phi : cal(X) -> cal(X)$ and oracle access to $cal(X)$, outputs an $eps$-approximate expected fixed point of $phi$.
] <thm:efp>
The existence of this algorithm proves, as a corollary, that $eps$-approximate expected fixed points always exist for every $eps > 0$. Although we will not need it here, it also turns out to be true that _exact_ expected fixed points also always exist (even when $phi$ is discontinuous) #citep(<Zhang25:Learning>). The "either-or" nature of the above theorem will become useful shortly.

#proof[
  Let $vx^((1)) in cal(X)$ be arbitrary, and consider the sequence of points $vx^((1)), ..., vx^((K))$ where $vx^((k)) = phi(vx^((k-1)))$ for each $k > 0$. Let $mu$ be the uniform distribution on ${vx^((1)), ..., vx^((K))}$. Then, by a telescoping sum, we have
  $
    EE_(vx ~ mu) [ phi(vx) - vx ] = 1/K sum_(k=1)^K (phi(vx^((k))) - vx^((k)) ) = 1/K sum_(k=1)^K (vx^((k+1)) - vx^((k)) ) = 1/K (vx^((K+1)) - vx^((1)))
  $
  where $vx^((K+1)) := phi(vx^((K)))$ for notational simplicity. But the right-hand side has norm at most $"diam"(cal(X))\/K$, so setting $K = "diam"(cal(X))\/eps$ completes the proof.
]

= Semi-separation

@thm:efp also, conveniently, solves the second problem. The key observation, as it turns out, is that it is not actually necessary to be able to efficiently optimize/separate over $Phi^q$. In fact, it is sufficient to be able to perform the following task, which we call _semi-separation_.

#definition("Semi-separation")[
  In the _semi-separation_ problem we are given a function $phi : cX -> RR^d$ and we have to compute _either_
  - an $epsilon$-expected fixed point $mu in Delta(cX)$ of $phi$, _or_
  - a point $vx in cX$ such that $phi(vx) in.not cX$.
] <def:semiseparation>

The proof of @thm:efp also gives a solution to the semi-separation problem: indeed, while constructing the sequence $vx^((1)), ..., vx^((K))$, if any $vx^((k))$ is not in $cal(X)$, we can immediately output $vx^((k-1))$.

It is instructive to contemplate briefly the implications of this result. A point $vx in cX$ such that $phi(vx) in.not cX$ gives a separating direction $va^top phi(vx) <= b$ separating $phi(vx)$ from $cX$; the same constraint can be used to separate $phi$ from $Phi^q$. Determining whether a given function $phi : cal(X) -> RR^d$ actually has image in $cal(X)$ is a hard problem, and indeed a semi-separation algorithm does not achieve this. Semi-separation only guarantees an expected fixed point _or_ a separation certificate; in particular, it is possible that semi-separation algorithms output an expected fixed point even when $phi in.not Phi^q$.

= Efficient regret minimization via semi-separation <sec:regret-via-semiseparation>

// We now show how to use a semi-separation oracle to construct an efficient regret minimizer over any set of functions $Phi^q$ expressible in the form given by @def:low-degree. #todo("actually do this")

We now briefly sketch how to use a semi-separation oracle to construct an efficient regret minimizer over any set of functions $Phi^q$ expressible in the form given by @def:low-degree.

At a high level, the algorithm relies on an online algorithm called _Shell gradient descent_, developed by~#citet(<Daskalakis25:Efficient>). This is just projected gradient descent, but with the twist that the underlying constraint set is _changing from round to round_. As long as the constraint set at each round contains the constraint set of interest, this approach is sound. Shell gradient descent hinges on a projection oracle, which crucially uses the semi-separation oracle developed above.



#import "figures/phi-fixed-point.typ": body as phi_fixed_point

#figure(caption: [#todo[Add caption]], scale(90%, phi_fixed_point))


// #colbreak()

#lec_bibliography("../meta/refs.bib")
