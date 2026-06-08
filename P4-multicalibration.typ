#import "meta/gabri_notes.typ": *
#import "figures/mcroute.typ" as fig_mcroute

#show: gabri_notes.with(lec_num: 4, date: none, title: "Phi-Regret and Multicalibration")

The black-box reduction from $Phi$-regret to online learning due to #citet(<Gordon08:No>) centers around the algorithmic primitive of fixed points. This chapter introduces a different route to the same destination that passes through forecasting #citep(<FarinaPerdomo26:Efficient>).

At a high level, the moral of this section is the following, also depicted in @fig:mcroute.

#info-box[
  One can construct a $Phi$-regret minimizer starting from a powerful enough _multicalibrated forecaster_ of the upcoming utility, and best respond to such forecast. In turn, the task of constructing such a forecaster admits a reduction to online learning in the style of Gordon-Greenwald-Marks, with expected variational inequalities replacing expected fixed points as the nonlinear optimization primitive.
]

As we will show towards the end of the chapter, this forecasting-based reduction comes with algorithmic benefits that forego the need for complicated semiseparation (#todo[Ref Chapter 3]). We note however that it is not known whether a forecasting-based approach can yield fast _offline_ algorithms for $Phi$-equilibrium computation (_cf._ #todo[Ref Chapter 2]).

// In particular, we will seek to establish the following diagram.

// More specifically, we present two reductions. First, multicalibration can be obtained from ordinary external regret minimization over a class of tests $cH$, provided that we can solve an _expected variational inequality_ (EVI). Second, $Phi$-regret can be obtained from a multicalibrated forecaster by best responding to the forecasts. Taken together, these reductions give a path from external regret to $Phi$-regret that does not require optimizing over the geometry of valid endomorphisms, nor computing their fixed points.


#figure(
  caption: [The two forecasting reductions and their relation to the Gordon-Greenwald-Marks framework.],
  html-figure-asset[
    #fig_mcroute.body(
      ggm: [Gordon, Greenwald, Marks [GGM08]],
      mc_from_regret: [Section 4.1],
      regret_from_mc: [Section 4.2],
    )
  ],
) <fig:mcroute>

= A Gordon-Greenwald-Marks result for multicalibration <sec:mcfromregret>

We start with online multicalibration. In every round $t$, Nature reveals a context $vc^((t)) in cC$. The learner outputs a distribution $D^((t))$ over forecasts $vp^((t)) in cU subset.eq RR^d$, and Nature then reveals the true utility vector $vu^((t)) in cU$. Given test $h : cC times cU -> RR^d$, the calibration error against $h$ is

$
  "MC-Err"^((T)) (h) :=
  sum_(t=1)^T EE_(vp^((t)) tilde D^((t))) [
    ip(h(vc^((t)), vp^((t))), vu^((t)) - vp^((t)))
  ].
$ <eq:mcerr>

The forecasts are $cH$-multicalibrated if $"MC-Err"^((T)) (h) = o(T)$ for every $h in cH$.

As we show in @theorem:mcfromregret, multicalibration admits a black-box reduction to online learning in the style of #citet(<Gordon08:No>). In the case of multicalibration, the nonlinear primitive is _not_ fixed points, but rather _expected variational inequalities_, as defined next.#footnote[Expected variational inequalities have appeared in the literature under different names #todo[finish]]

#definition("Expected variational inequality")[
  Let $S : cU -> RR^d$ be an operator and let $epsilon > 0$. An _$epsilon$-solution_ to the expected variational inequality induced by $S$ is a distribution $D$ over $cU$ such that
  $ EE_(vp tilde D) [ ip(S(vp), vu - vp) ] <= epsilon quad forall vu in cU. $ <eq:evi>
]

One way to parse (@eq:evi) is as a randomized self-consistency condition. The distribution $D$ may place mass on several forecasts; nevertheless, after averaging over that randomness, no possible utility vector $vu in cU$ has positive correlation with the residual direction $S(vp)$ by more than $epsilon$. This is exactly what will make the regret analysis telescope. Efficient algorithms for EVIs are known for general compact convex sets under mild oracle access assumptions; for our purposes, the important point is that EVIs are a standalone optimization primitive, just as fixed points were in the Gordon-Greenwald-Marks theorem from Chapter 1.

Now suppose that we have an external-regret minimizer $R_cH$ whose decision set is the class of tests $cH$. In each round, it chooses a test $h^((t)) in cH$; after the utility vector is revealed, it receives the linear utility

$
  g^((t))(h) :=
  EE_(vp^((t)) tilde D^((t))) [
    ip(h(vc^((t)), vp^((t))), vu^((t)) - vp^((t)))
  ].
$ <eq:testutility>

The reduction is as follows.

#pseudocode-list(
  booktabs: true,
  title: [*Algorithm*: Multicalibration from external regret],
)[
  - *Input:* An external regret minimizer $R_cH$ for the test set $cH$
  - *function* `NextForecast`($vc^((t))$):
    - Set $h^((t)) := R_cH.$`NextStrategy`$()$
    - Let $D^((t))$ solve the EVI with $S^((t))(vp) := h^((t))(vc^((t)), vp)$:
      $
        EE_(vp^((t)) tilde D^((t))) [
          ip(h^((t))(vc^((t)), vp^((t))), vu - vp^((t)))
        ] <= epsilon^((t)) quad forall vu in cU
      $
    - *return* $D^((t))$
  - *function* `ObserveUtility`($vu^((t))$):
    - Feed the linear utility $g^((t))$ in (@eq:testutility) to $R_cH$
]

#theorem[#citep(<FarinaPerdomo26:Efficient>)][
  Let $"Reg"_cH^((T))(h)$ be the external regret of $R_cH$ with respect to the sequence of utilities $g^((1)), dots, g^((T))$, and let $"EVI"^((T)) := sum_(t=1)^T epsilon^((t))$. Then the forecasts produced by the algorithm above satisfy
  $ "MC-Err"^((T)) (h) <= "Reg"_cH^((T))(h) + "EVI"^((T)) quad forall h in cH. $
] <theorem:mcfromregret>

#proof[
  The EVI condition is invoked at the realized utility vector $vu^((t))$, so
  $ g^((t))(h^((t))) <= epsilon^((t)). $
  Therefore, for any comparator test $h in cH$,
  $
    "MC-Err"^((T)) (h) & = sum_(t=1)^T g^((t))(h) \
                       & <= sum_(t=1)^T (g^((t))(h) - g^((t))(h^((t)))) + sum_(t=1)^T epsilon^((t)) \
                       & = "Reg"_cH^((T))(h) + "EVI"^((T)).
  $
  Thus, sublinear regret over $cH$, together with $sum^((T)) epsilon^((t)) = o(T)$, gives sublinear multicalibration error.
]

*Necessity of EVIs.* In a precise sense, EVIs are also _necessary_. Suppose we already had an efficient $cH$-multicalibrated forecaster. Fix a context $vc in cC$ and a test $h in cH$, and consider the EVI operator $S(vp) = h(vc, vp)$. To solve this EVI, run the forecaster for $T$ rounds with the same context $vc^((t)) = vc$. After it outputs $D^((t))$, choose Nature's response as

$ vu^((t)) in arg max_(vu in cU) ip(vu, EE_(vp tilde D^((t))) [S(vp)]). $ <eq:eviadversary>

For any fixed $vu^star in cU$, the choice in (@eq:eviadversary) gives

$
  EE_(vp tilde D^((t))) [ip(S(vp), vu^((t)) - vp)]
  >=
  EE_(vp tilde D^((t))) [ip(S(vp), vu^star - vp)].
$

Averaging the distributions $D^((1)), dots, D^((T))$ uniformly therefore produces a distribution $D$ with

$
  EE_(vp tilde D) [ip(S(vp), vu^star - vp)]
  <= ("MC-Err"^((T)) (h)) / T
  quad forall vu^star in cU.
$

As $T$ grows, multicalibration drives the right-hand side to $0$. So an online multicalibrated forecaster gives a black-box EVI solver.

= From Multicalibration to Phi-regret minimization <sec:regretfrommc>

We now turn the arrow around and use forecasting to make decisions. Let $cX subset.eq RR^d$ be a compact convex action set, and let $cU subset.eq RR^d$ be a compact convex set of possible utility vectors. A deviation class is a family $Phi subset.eq {phi : cX -> cX}$. If the learner outputs distributions $mu^((t))$ over $cX$, its $Phi$-regret against a deviation $phi in Phi$ is

$
  Phi"Reg"^((T))(phi) :=
  sum_(t=1)^T EE_(vx^((t)) tilde mu^((t))) [
    ip(phi(vx^((t))) - vx^((t)), vu^((t)))
  ].
$ <eq:phiregp4>

The reduction asks the forecaster to predict the next utility vector. Given a forecast $vp in cU$, define the deterministic best response

$ sigma(vp) in arg max_(vx in cX) ip(vx, vp), $ <eq:bestresponse>

with ties broken by a fixed rule. If the forecaster outputs a distribution $D^((t))$ over forecasts $vp^((t))$, the decision maker plays the pushforward distribution $mu^((t))$ induced by $vx^((t)) = sigma(vp^((t)))$.

What tests should the forecaster be calibrated against? For each deviation $phi in Phi$, define

$
  h_phi (vp) := phi(sigma(vp)) - sigma(vp),
  qquad quad
  cH_Phi := {h_phi : phi in Phi}.
$ <eq:hphi>

This definition is the whole reduction. The test $h_phi$ measures the direction in utility space in which the deviation $phi$ would improve over the best response to the forecast.

#pseudocode-list(
  booktabs: true,
  title: [*Algorithm*: $Phi$-regret from multicalibration],
)[
  - *Input:* A forecaster over $cU$ multicalibrated with respect to $cH_Phi$
  - `NextStrategy`():
    - Query the forecaster and receive a distribution $D^((t))$ over forecasts $vp^((t)) in cU$
    - *return* the pushforward distribution $mu^((t))$ of $vx^((t)) = sigma(vp^((t)))$
  - `ObserveUtility`($vu^((t))$):
    - Feed the realized utility vector $vu^((t))$ to the forecaster
]

#theorem[#citep(<FarinaPerdomo26:Efficient>)][
  If the forecaster in the algorithm above has multicalibration error $"MC-Err"^((T))$ with respect to $cH_Phi$, then the decision maker has
  $ Phi"Reg"^((T))(phi) <= "MC-Err"^((T))(h_phi) quad forall phi in Phi. $
  In particular, sublinear $cH_Phi$-multicalibration implies sublinear $Phi$-regret.
] <theorem:phifrommc>

#proof[
  Fix any $phi in Phi$. Since $mu^((t))$ is the pushforward of $D^((t))$ under $vp |-> sigma(vp)$,
  $
    Phi"Reg"^((T))(phi) & = sum_(t=1)^T EE_(vp^((t)) tilde D^((t))) [
                            ip(h_phi (vp^((t))), vu^((t)))
                          ] \
                        & = sum_(t=1)^T EE_(vp^((t)) tilde D^((t))) [
                            ip(h_phi (vp^((t))), vu^((t)) - vp^((t)))
                          ] + sum_(t=1)^T EE_(vp^((t)) tilde D^((t))) [
                            ip(h_phi (vp^((t))), vp^((t)))
                          ].
  $
  The first term is exactly $"MC-Err"^((T))\(h_phi\)$. The second term is nonpositive, because
  $
    ip(h_phi (vp), vp)
    = ip(phi(sigma(vp)), vp) - ip(sigma(vp), vp)
    <= 0
  $
  by the definition of $sigma(vp)$ as a maximizer over $cX$.
]

The theorem is useful because the complexity of the required calibration class scales with the complexity of the deviation class. If $Phi$ contains only constant deviations, then $cH_Phi$ is a class of constant best-response gaps and we recover external regret. If $Phi$ grows toward all maps $cX -> cX$, then $cH_Phi$ becomes a strong calibration class and the result approaches the classical connection between calibration and swap regret.

The contextual version is identical. If contexts $vc^((t))$ are observed before play and deviations have the form $phi : cC times cX -> cX$, then we define

$ h_phi (vc, vp) := phi(vc, sigma(vp)) - sigma(vp). $

Multicalibration with respect to these tests gives contextual $Phi$-regret.

== Putting the two reductions together

Combining @theorem:mcfromregret and @theorem:phifrommc gives the promised route from external regret to $Phi$-regret. Set $cH = cH_Phi$. Run the multicalibration algorithm as the forecaster inside the best-response algorithm. Then, for every $phi in Phi$,

$
  Phi"Reg"^((T))(phi)
  <= "MC-Err"^((T))\(h_phi\)
  <= "Reg"_cH^((T))\(h_phi\) + "EVI"^((T)).
$ <eq:combinedroute>

Thus the burden of $Phi$-regret minimization shifts to two primitives:

+ no-regret learning over the induced test class $cH_Phi$; and
+ solving EVIs over the forecast domain $cU$.

The key distinction from the Gordon-Greenwald-Marks path is that we do _not_ need to optimize over valid deviations directly. We only need a regret minimizer over tests that contain the maps $h_phi$. This extra flexibility is what makes the approach especially clean for large structured deviation classes.

== Linear deviations

As an illustration, consider linear deviations $phi(vx) = matM vx$ over a convex compact set $cX subset.eq RR^d$. The classical GGM approach asks us to understand the geometry of all matrices $matM$ satisfying $matM cX subset.eq cX$, and then compute fixed points of the matrices selected by the regret minimizer. That geometry can be difficult.

The multicalibration route permits a relaxation. From (@eq:hphi),

$ h_phi(vp) = (matM - matI) sigma(vp). $

If every valid linear endomorphism has spectral norm at most $S$, then each test above lies in a Frobenius ball of radius on the order of $sqrt(d)(S + 1)$. Instead of learning over valid endomorphisms, we can learn over the larger class

$ cH := { vp |-> matA sigma(vp) : norm(matA)_F <= rho }, $

where $rho$ is chosen large enough to contain all tests $h_phi$. This larger class is easy: external regret over a Euclidean ball is handled by projected gradient descent. Indeed, the utility sent to the matrix learner at time $t$ is linear:

$
  g^((t))(matA)
  =
  EE_(vp^((t)) tilde D^((t))) [
    ip(matA sigma(vp^((t))), vu^((t)) - vp^((t)))
  ]
  =
  ip(
    matA, EE_(vp^((t)) tilde D^((t))) [
      (vu^((t)) - vp^((t))) sigma(vp^((t)))^top
    ]
  )_(F).
$

Consequently, if $norm(vx)_2 <= B$ for $vx in cX$ and $norm(vu)_2 <= L$ for $vu in cU$, the standard projected-gradient bound gives $O(B L S sqrt(d T))$ linear-swap regret, up to constants. This improves the dimension dependence of the recent semi-separation-based route of #citet(<Daskalakis25:Efficient>), while avoiding semi-separation altogether.

== RKHS deviations

The same idea extends beyond finite-dimensional linear classes. Suppose that the deviation functions $phi : cC times cX -> cX$ lie in a vector-valued reproducing kernel Hilbert space with matrix-valued kernel

$ Gamma((vc, vx), (vc', vx')) in RR^(d times d). $

For the contextual reduction, the relevant tests are

$ h_phi (vc, vp) = phi(vc, sigma(vp)) - sigma(vp). $

These tests also lie in an RKHS. Indeed, the first term is represented by composing the deviation kernel with the best-response map, while the second term is represented by the linear kernel on $sigma(vp)$. The resulting kernel is

$
  Gamma'((vc, vp), (vc', vp'))
  :=
  Gamma((vc, sigma(vp)), (vc', sigma(vp')))
  + sigma(vp) sigma(vp')^top.
$ <eq:rkhskernel>

So, if we have an online learner over the RKHS ball associated with $Gamma'$, @theorem:mcfromregret supplies a multicalibrated forecaster, and @theorem:phifrommc turns it into a no-$Phi$-regret algorithm. This recovers low-degree polynomial deviations as a special case and also covers infinite-dimensional kernels, such as Gaussian kernels, where the fixed-point route does not have an obvious finite-dimensional endomorphism geometry to exploit.

The moral is that forecasting separates the two hard-looking parts of $Phi$-regret. Best response converts forecasts to actions; multicalibration certifies that no deviation can systematically exploit the forecast errors; and EVIs provide the self-consistency condition that makes the forecaster black-box reducible to ordinary external regret.

#lec_bibliography("meta/refs.bib")
