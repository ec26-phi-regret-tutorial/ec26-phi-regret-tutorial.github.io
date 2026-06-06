#import "meta/gabri_notes.typ": *

#show: gabri_notes.with(lec_num: 3, date: none, title: "Beyond Normal Form")

#show "TreeSwap": `TreeSwap`

#todo(
  "my eventual intention is to merge the semi-separation chapter into this chapter. there is a lot of overlapping content -brian",
)

In this section, we will show how to construct $Phi$-regret minimizers for strategy sets $cX$ that are more complex than the simplex. To do so, we first define the sets of deviations that we work with.

#definition[#citep(<Zhang25:Learning>)][
  Given a feature map $q : cX -> RR^(k)$, the set of deviations $Phi^q$ is the set of all maps $phi: cX -> cX$ that can be expressed as the matrix-vector product $matK(phi) q(vx)$ for some matrix $matK in RR^(d times k)$ and vector $vc in RR^d$.
] <def:low-degree>

#example[
  If $q(vx) = (1, vx) in RR^(d+1)$, then $Phi^q$ is the set of all linear maps $phi : cX -> cX$. More generally, if $q(vx) = vx^(times.o <= ell) in RR^k$ where $k = binom(d, <= ell)$ is the set of all monomials in $vx$ up to degree $ell$, then $Phi^q$ is the set of all degree-$ell$ polynomials in $vx$.
]
In general, we will describe algorithms that have complexity polynomial in the dimension $k$. Note that, in the special case of degree-$ell$ polynomials, these algorithms will therefore run in time polynomial in $d^ell$.

In these settings, we run into two fundamental issues when trying directly to generalize the GGM construction. Recall that the GGM construction requires two ingredients. We will describe why both ingredients are problematic. Intuitively, both problems stem from the same root cause:

1. _Fixed-point oracle_: Finding a fixed point of a continuous function $phi : cX -> cX$ is $PPAD$-complete, and therefore is unlikely to be possible efficiently. Even more generally, if $phi$ contains _discontinuous_ maps, $phi$ may not even admit any approximate fixed points!

2. _External regret minimizer for $Phi^q$_: External regret minimization implies the ability to (approximately) perform linear optimization over $Phi^q$. But linear optimization over $Phi^q$ can be hard, in two ways:
  #set enum(numbering: "a.")

  + If $Phi^q$ contains nonlinear functions, then that this optimization problem is $NP$-hard even when $cX$ is the hypercube $[0, 1]^d$ #cite(<Zhang24:Efficient>).

  + Even if $Phi^q$ is the set of linear endomorphisms, if $cX$ itself is only available via an oracle (e.g., a separation oracle), then optimizing a linear function over $Phi^q$ can in general be hard  #cite(<Daskalakis25:Efficient>). #todo("why is the citation not showing?")

#remark[
  When $cX$ is represented by an explicit set of linear constraints, i.e., $cX = { vx : matA vx <= vb }$, the set of linear endomorphisms can also be expressed as an explicit set of linear constraints #cite(<Zhang25:Expected>). This is true, for instance, for _extensive-form games_. Thus, in this particular case, linear regret minimization can be accomplished without all the machinery of this section.
]

#remark[
  For extensive-form games in particular, the set of linear endomorphisms has a clean game-theoretic interpretation #cite(<Zhang23:Mediator>). We will not detail it here, though.
]

We will show how to circumvent both of the above issues. We start with the fixed point computation.

= Expected fixed points

Given that computing a fixed point is generally hard (when $phi$ is continuous but nonlinear) or even impossible (when $phi$ is discontinuous), we need a different notion of fixed point. Fortunately, by examining the proof of the Gordon--Greenwald--Marks framework, it turns out that a weaker condition suffices.

#definition[Expected fixed point #citep(<Zhang24:Efficient>)][
  Given a function $phi : cX -> cX$, an $eps$-approximate _expected fixed point_ of $phi$ is a distribution#footnote[Throughout this document, to avoid measure-theoretic issues, we only consider finite-support distributions.] $mu in Delta(cX)$ such that $ norm(EE_(vx ~ mu) [ phi(vx) - vx ])_1 <= eps. $
]

As we will see shortly, expected fixed points fix both problems with fixed point computation: an (approximate) expected fixed point always exists, and moreover, there is an efficient algorithm for computing one!

#todo("continue from here -brian")


// #colbreak()

#lec_bibliography("../meta/refs.bib")
