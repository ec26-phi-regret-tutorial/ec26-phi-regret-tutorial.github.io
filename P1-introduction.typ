#import "meta/gabri_notes.typ": *
#import "figures/chicken.typ" as fig_chicken
#import "figures/swap-vs-external.typ" as fig_swap_vs_external

#show: gabri_notes.with(lec_num: 1, date: none, title: "Introduction")

#v(4mm)
This introductory chapter covers basic background on regret minimization and connections to game-theoretic equilibrium concepts. In particular, we begin by introducing and motivating the notion of _$Phi$-regret_ and its associated solution concept in games---_$Phi$-equilibrium_. In the second part, we will introduce the canonical algorithmic template for minimizing _$Phi$-regret_ due to~#citet(<Gordon08:No>).

= Online learning and regret

We consider a _learner_ who makes a sequence of decisions over $T$ rounds. The learner interacts repeatedly with an _environment_. In each round $t in [T]$, the learner specifies a mixed strategy $vx^((t)) in cX$, where $cX$ is a convex and compact set. (A canonical case arises when $cX$ is a probability simplex over a finite set of actions, but our focus here is on the general setting.) The environment then selects a _utility vector_ $vu^((t))$, so that the utility obtained by the learner at that round is $ip(vx^((t)), vu^((t)))$. In the full-feedback setting, which is the focus of these notes, $vu^((t))$ is revealed to the learner after the end of the round. An online algorithm produces a sequence of strategies based on the feedback observed up to that point.

What's a sensible way of measuring the performance of the learner in this online environment? There are different notions of _hindsight rationality_. Perhaps the most common performance benchmark is _external regret_, defined as

$ "Reg"^((T)) := max_(vx in cX) { sum_(t=1)^T ip(vx, vu^((t))) } - sum_(t=1)^T ip(vx^((t)), vu^((t))). $ <eq-regret>

The second term in the right-hand side of (@eq-regret) is the cumulative utility obtained by the learner through the $T$ rounds, whereas the first term is the optimal utility that could have been obtained in hindsight _through a fixed strategy_. We will soon introduce more powerful notions of hindsight rationality beyond external regret.

= Games and solution concepts

We will often need to analyze what happens when multiple no-regret players repeatedly interact in a game. To do so, we begin by introducing the canonical normal-form representation of games. While any finite game can be cast in normal form, that representation is often inefficient. This will motivate introducing more compact game representations, as we shall do in the sequel.

Formally, we have a set of $n$ players. In a normal-form game, each player $i in [n]$ has a finite set of available actions $cA_i$; we will use the shorthand notation $m_i := |cA_i|$ for the number of actions. Every player $i in [n]$ has a _utility function_ $u_i$ mapping a joint action profile $(a_1, ..., a_n) in cA_1 times dots.c times cA_n$ to a real value $u_i (a_1, ..., a_n)$. Players can randomize by specifying a probability distribution over their available actions, so that the strategy set of each player is the probability simplex $cal(X)_i = Delta(cA_i)$. Under a joint strategy $(x_1, ..., x_n) in Delta(cA_1) times dots.c times Delta(cA_n)$, the _expected utility_ of player $i in [n]$ reads

$
  u_i (vx_1, ..., vx_n) &:= EE_((a_1, ..., a_n) tilde (vx_1, ..., vx_n)) [ u_i (a_1, ..., a_n) ] \
  &= sum_((a_1, ..., a_n) in cA_1 times dots.c times cA_n) quad product_(i' = 1)^n vx_(i') [a_(i')] u_i (a_1, ..., a_n).
$

Each player is trying to maximize its expected utility.

== Correlated and coarse correlated equilibria

A major criticism of the Nash equilibrium is that, even though one always exists---as guaranteed by the famous theorem of John Nash~#citep(<Nash50:Equilibrium>)---it is computationally intractable to find one #citep(<Daskalakis08:Complexity>)---let alone a welfare-optimal one #citep(<Gilboa89:Nash>). As result, we shouldn't expect simple, computationally bounded learning algorithms to converge to Nash equilibria; this raises the question: what are no-regret dynamics converging to in general-sum games?

It turns out that no-regret learning is inherently tied to _coarse correlated equilibria_ #citep(<Moulin78:Strategically>). Let's begin by recalling the basic definition and start building some intuition about this solution concept; for now, we restrict our attention to normal-form games.

#definition("Coarse correlated equilibrium")[
  A correlateddistribution $mu in Delta(cA_1 times dots.c times cA_n)$ is an _$epsilon$-coarse correlated equilibrium (CCE)_ if for any player $i in [n]$ and deviation $a_i' in cA_i$,
  $
    EE_((a_1, ..., a_n) tilde mu) [ u_i (a_1, ..., a_n) ] >= EE_((a_1, ..., a_n) tilde mu) [ u_i (a_i', a_(-i)) ] - epsilon.
  $ <def:CCE>
]

This definition mirrors Nash equilibria, but with a critical difference: the underlying distribution $mu$ can be _correlated_; by contrast, in a Nash equilibrium $mu$ has to be a _product distribution_, reflecting the fact that players randomize independently. To explain this, let's consider the following two distributions with respect to some $2 times 2$ bimatrix game (meaning that each of the two players has two available actions):

$
  mu = mat(1/2, 0; 0, 1/2), quad
  mu' = mat(1/6, 1/6; 1/3, 1/3).
$

Both are distributions over $cA_1 times cA_2 = {"1", "2"} times {"1", "2"}$, but only $mu'$ is a product distribution. Indeed, if Player 1---the row player---plays $(1/3, 2/3)$ and Player 2 plays $(1/2, 1/2)$, the induced distribution over the $4$ outcomes matches $mu'$. In contrast, no pair of strategies gives rise to $mu$.

A Nash equilibrium is always a CCE; a Nash equilibrium is basically an _uncorrelated_ (coarse) correlated equilibrium. But the set of CCEs can unlock new outcomes. Before we examine a concrete example, we also introduce the stronger notion of a _correlated equilibrium_, famously put forward by #citet(<Aumann74:Subjectivity>).

#definition("Correlated equilibrium")[
  A correlated distribution
  $mu in Delta(cA_1 times dots.c times cA_n)$ is an $epsilon$-_correlated equilibrium (CE)_ if for any player
  $i in [n]$
  and deviation function
  $phi_i : cA_i -> cA_i$,
  we have
  $
    EE_((a_1, dots, a_n) tilde mu)[
      u_(i)(a_1, dots, a_n)
    ]
    >=
    EE_((a_1, dots, a_n) tilde mu)[
      u_i (phi_i (a_i), a_(-i))
    ] - epsilon.
  $
  <def:CE>
]

Both correlated and coarse correlated equilibria can be interpreted through the use of a trusted third party---a _mediator_ or _correlation device_---who samples a joint action profile $(a_1, ..., a_n)$ from the correlated distribution $mu$ and then provides the corresponding action $a_i$ to each player $i in [n]$ as a _recommendation_. From this point of view, a distribution is a CCE or a CE if no player has an incentive to deviate from the recommendation, but for CEs the set of possible deviations is richer: in a CE, a player can decide whether to deviate _after_ observing the recommendation, while in a CCE the decision has to be made in advance. This makes a CCE harder to justify in some applications, as one would need some binding mechanism to ensure the player would not be able to deviate after observing the recommendation.

We now go over a concrete example to further elucidate these concepts.

#example[
  We consider the "game of chicken." This is a $2 times 2$ game---played between two drivers who are rapidly approaching an intersection from different streets---whose utilities are tabulated in @fig:chicken. Each player can either play "Stop" or "Go." If both players elect to "Go" a crash ensues---a bad outcome for both players. If a player chooses to "Stop" it gets no utility from the game, whereas if it proceeds while the other player chooses to "Stop" it gets a utility of $1$ for safely crossing the intersection.

  This game has exactly three Nash equilibria: i) (Go, Stop), ii) (Stop, Go), and iii) $((5/6, 1/6), (5/6, 1/6))$, meaning that both players play "Stop" with probability $5/6$. From these three outcomes, the first two are _not_ equitable in that they favor one player over the other. The third outcome is even worse: it leads to a crash with some positive probability.

  (C)CEs address these issues by unlocking new outcomes. In particular, let's consider the correlated distribution $1/2 ("Go", "Stop") + 1/2 ("Stop", "Go")$. It's easy to verify that this is a CE, and thus a CCE. Under that distribution, both players get in expectation a utility of $1/2$. Focusing CEs, there is a natural interpretation of this outcome through a _traffic light_, which provides a signal to each player. If Player 1 is recommended "Stop," it means that Player 2 will play "Go" with probability $1$, so stopping is in Player 1's interest. On the other hand, if Player 1 is recommended "Go," it means that Player 2 will play "Stop" with probability $1$, so crossing is safe for Player 1. That is, in a CE, the signal a player observes updates that player's beliefs concerning the behavior of the other players
  #figure(align(center)[#fig_chicken.body], caption: [The game of chicken.]) <fig:chicken>
]

#example[
  Our next example clarifies the difference between CCEs and CEs. We consider a $4 times 4$ bimatrix game described with the payoff matrices
  #set math.equation(numbering: "(1)")
  $
    M_R = mat(
      2, 0, 0, 0;
      0, 2, 0, 0;
      3, 0, 0, 0;
      0, 3, 0, 0
    )
    quad "and" quad
    M_C = mat(
      2, 0, 3, 0;
      0, 2, 0, 3;
      0, 0, 0, 0;
      0, 0, 0, 0
    )
  $ <eq-gameclique>
  #set math.equation(numbering: none)
  for the row and column player, respectively. We label each player's actions as "1," "2," "3," and "4." We claim that the distribution $mu = 1/2 ("1", "1") + 1/2 ("2", "2")$ is an exact CCE of this game, whereas the CE gap of $mu$ is large---namely, $1$. Specifically, the swap deviation $phi$ that results in a large deviation gain is $1 |-> 3$ and $2 |-> 4$; the mapping for the rest of the actions is moot, as they are never played under $mu$. While each player obtains a utility of $2$ under $mu$, deviating per $phi$ gives a utility of $3$. At the same time, $mu$ is a CCE as it is robust with respect to constant deviations.
]

== Computational properties

We have seen that CCEs and CEs give rise to new outcomes that are not attainable under independent randomization. Moreover, correlated equilibrium concepts have better computational properties than Nash equilibria. Specifically, a key property is that the set of (C)CEs is convex and can be described through a linear program.

#proposition[
  There is a linear program with $product_(i=1)^n |cA_i|$ variables and $sum_(i=1)^n |cA_i| (|cA_i| - 1)$ constraints whose solution is an exact correlated equilibrium of the game. <prop:LP>
]

While the number of swap deviations of each player $i in [n]$ is $|cA_i|^(|cA_i|)$, it's enough to consider only a certain subset of swap deviations---ones that only change a _single_ action; such deviations are called _internal_---with size $|cA_i| (|cA_i| - 1)$; the simple proof is left as an exercise. This means that, for normal-form games with a constant number of players, a correlated equilibrium can be computed in polynomial time.

One caveat of this characterization is that the size of the LP grows exponentially with the number of players; the basic reason why this happens is that a correlated distribution in multi-player games is an exponential objective---one needs to specify the value of $product_(i=1)^n |cA_i| - 1$ coordinates. As we shall see in the sequel, there is an ingenious algorithm for addressing this issue. For now, it is reassuring to know that one can compute a CE in normal-form games with a constant number of players. It is worth noting that one can also incorporate any linear objective function into the linear program, such as the _social welfare_---the sum of the players' utilities.

== Connection to no-regret learning <sec:noregret>

As we have alluded to, (coarse) correlated equilibria are closely tied to the framework of regret minimization in online learning.

*$Phi$-regret.* To formalize this connection in its full generality, we will now introduce the important concept of _$Phi$-regret_. It is a measure of the learner's performance parameterized by a family of _strategy deviations $Phi$_. For a set of deviations $Phi$ comprising functions $phi : cX -> cX$, $Phi$-regret is defined as

$
  Phi"Reg"^((T)) := max_(phi in Phi) { sum_(t=1)^T ip(phi(vx^((t))), vu^((t))) } - sum_(t=1)^T ip(vx^((t)), vu^((t))) .
$ <eq-Phiregret>

We covered a moment ago the special case where $Phi$ comprises only _constant deviations_: $Phi_("const") = { phi : exists vx' in cX " such that " phi(vx) = vx' }$; this is indeed the most standard definition of regret in online learning, referred to as _external_ regret. The key point about (@eq-Phiregret) is that the richer the set of deviations $Phi$, the stronger the induced notion of hindsight rationality. The other end of the spectrum where $Phi$ consists of all possible deviations $cX -> cX$ gives rise to _swap regret_. The next example shows that an algorithm can experience large swap regret even when its external regret is small.

#example[
  Let's say the learner picks a distribution over three actions, "1," "2," and "3." Suppose further that the sequence of utilities and selected actions follow the pattern shown in @fig:swap-vs-external, where we can assume $T = 0 mod 3$. In this example, the learner obtains overall a utility of $T/3$. In fact, this matches the optimal strategy in hindsight. So, the external regret of the learner is $0$ in this example. At the same time, consider the swap deviation
  $ phi: a |-> cases("2" & "if" a = "1,", "1" & "if" a = "2,", "3" & "if" a = "3.") $
  Under that deviation, the learner would be able to collect maximal utility, implying that the swap regret of the learner is $Omega(T)$.

  #figure(
    align(center)[#fig_swap_vs_external.body],
    caption: [An example of a learner with large swap regret but zero external regret.],
  ) <fig:swap-vs-external>
]

This example shows that external regret---and the associated solution concept of a CCE---is a weak benchmark. In fact, a CCE can be supported on strictly dominated actions~#citep(<Viossat13:Noregret>)! This motivates considering tighter equilibrium concepts revolving around the notion of $Phi$-regret.

Now, the techniques we have covered so far for minimizing external regret will not be enough to minimize swap regret. For example, multiplicative weights update (MWU) can have linear swap regret #citep(<Cesa-Bianchi06:Prediction>). As a result, we will need new algorithmic ideas to go beyond external regret and coarse correlated equilibria.

Before we proceed, we formalize the connection between minimizing $Phi$-regret and correlated equilibrium concepts. We now extend our scope to general _multilinear games_. Here, each player $i in [n]$ selects a strategy $vx_i in cX_i$ from a convex and compact set $cX_i$, so that for any joint strategy $(vx_1, dots, vx_n) in cX_1 times dots.c times cX_n$, the utility can be expressed as $u_i (vx_1, dots, vx_n) = ip(vx_i, vu_i (vx_(-i)))$ for some utility vector $vu_i$ that does not depend on $vx_i$. This is a useful abstraction for encompassing both normal- and extensive-form games, the latter under the so-called sequence-form representation.

#definition[$Phi$-equilibrium][
  A correlated distribution $mu in Delta(cX_1 times dots.c times cX_n)$ is an _$epsilon$-$Phi$-equilibrium_ if for any player $i in [n]$ and deviation function $phi_i: cX_i -> cX_i$,
  $
    EE_((vx_1, dots, vx_n) tilde mu) u_i (vx_1, dots, vx_n) >= EE_((vx_1, dots, vx_n) tilde mu) u_i (phi_i (vx_i), vx_(-i)) - epsilon.
  $
]

#theorem[
  Suppose that each player $i in [n]$ incurs $Phi_i$-regret $Phi"reg"_i^((T))$ under the sequence of utilities $(vu_i (vx^((t))_(-i)))_(t=1)^T$. Then the average correlated distribution of play $mu := 1/T sum_(t=1)^T vx_1^((t)) times.o dots.c times.o vx_n^((t))$ is an $epsilon$-$Phi$-equilibrium with $epsilon = 1/T max_(1 <= i <= n) Phi"Reg"_i^((T))$.
] <theorem:ce-reg>

Above, $vx_1^((t)) times.o dots.c times.o vx_n^((t))$ is the product distribution induced by $(vx_1^((t)), dots, vx_n^((t)))$; $times.o$ denotes the tensor product. The distribution $mu$ produced by @theorem:ce-reg is a _mixture_ of $T$ product distributions. Correlation arises by playing multiple iterations of the game. As a special case, @theorem:ce-reg implies that players minimizing swap regret converge---in terms of the average correlated distribution of play---to correlated equilibria, whereas external regret is associated with _coarse_ correlated equilibria.

#proof[
  For any player $i in [n]$, we have
  $
    Phi"Reg"^((T)) &= max_(phi_i in Phi_i) { sum_(t=1)^T ip(phi(vx_i^((t))), vu_i^((t))) } - sum_(t=1)^T ip(vx_i^((t)), vu_i^((t))) \
    &= max_(phi_i in Phi_i) { sum_(t=1)^T u_i (phi_i (vx^((t))_i), vx^((t))_(-i)) } - sum_(t=1)^T u_i (vx_1^((t)), dots, vx_n^((t))),
  $ <align:cont>
  by multilinearity. Let $mu = 1/T sum_(t=1)^T times.o.big_(i=1)^n vx_i^((t))$. Continuing from (@align:cont),
  $
    1/T Phi"Reg"^((T)) = max_(phi_i in Phi_i) EE_((vx_1, dots, vx_n) tilde mu) u_i (phi_i (vx_i), vx_(-i)) - EE_((vx_1, dots, vx_n) tilde mu) u_i (vx_1, dots, vx_n).
  $
]

= A framework for minimizing $Phi$-regret <sec:Gordon>

Having connected $Phi$-regret with $Phi$-equilibria, we now introduce the elegant framework of #citet(<Gordon08:No>) to minimize $Phi$-regret in the online learning setting; by virtue of @theorem:ce-reg, one can then compute a $Phi$-equilibrium by having each player employ such algorithms.

*Reducing $Phi$-regret to external regret.* The key idea in the construction of #citet(<Gordon08:No>) is that one can reduce minimizing $Phi$-regret to minimizing external regret. The framework of #citet(<Gordon08:No>) provides a general template based on two basic subroutines.

1. A _fixed-point oracle_: for any deviation $phi in Phi$, it outputs a fixed point $vx = phi(vx)$. <item:fp>
2. An online algorithm $R_(Phi)$ minimizing _external regret_ with respect to _the set $Phi$_. <item:phireg>

Regarding the fixed-point oracle, we will assume that $Phi$ consists of continuous functions mapping $cX$ to $cX$, so that the _existence_ of a fixed point is guaranteed by Brouwer's fixed-point theorem; whether such a fixed point can be computed efficiently is a different story. (As we shall see in the sequel, computing approximate fixed points of general functions is known to be equivalent to finding Nash equilibria #citep(<Daskalakis08:Complexity>), which defeats our purpose.) For now, we can assume that $Phi$ is structured enough so that it admits an efficient fixed-point oracle; for example, this is the case when $Phi$ contains only linear deviations. A final point is that it will be enough if one has instead an _approximate_ fixed-point oracle, in that $norm(vx - phi(vx)) <= epsilon$.

Assuming access to a fixed-point oracle, the reduction of #citet(<Gordon08:No>) reduces $Phi$-regret to external regret, but with an important catch: the algorithm minimizing external regret needs to _operate over the set of deviations $Phi$_. This is a significantly more complex set than the one we started with, and will be the critical step in establishing efficient $Phi$-regret minimizers.

Assuming access to these oracles, the algorithm of #citet(<Gordon08:No>) produces a $Phi$-regret minimizer $R$ as follows.

- In every time $t in [T]$, it obtains the next strategy $phi^((t))$ of $R_(Phi)$. $R$ then produces as the next strategy $vx^((t)) in cX$ any fixed point of $phi^((t))$ through the fixed-point oracle.
- Next, upon observing $vu^((t))$, $R$ feeds to $R_(Phi)$ the utility function $u^((t))_(Phi): phi |-> ip(phi(vx^((t))), vu^((t)))$.


#pseudocode-list(
  booktabs: true,
  title: [*Algorithm* #citep(<Gordon08:No>): $Phi$-regret minimizer#h(1fr) <alg:Gordon>],
)[
  - *Input:* An external regret minimizer $R_Phi$ for the set $Phi$
  - *function* `NextStrategy`():
    - Set $phi^((t)) := R_Phi.$`NextStrategy`$()$
    - *return* a fixed point $vx^((t)) = phi^((t))(vx^((t)))$
  - *function* `ObserveUtility`($vu^((t))$):
    - Set $u_Phi^((t)) : phi |-> ip(phi(vx^((t))), vu^((t)))$
    - $R_Phi.$`ObserveUtility`$(u_Phi^((t)))$
]

#theorem[#citep(<Gordon08:No>)][
  If $"Reg"^((T))$ is the external regret of $R_(Phi)$ and $Phi"Reg"^((T))$ is the $Phi$-regret of $R$, then $"Reg"^((T)) = Phi"Reg"^((T))$.
] <theorem:Gordon>

#proof[
  We have
  $
    Phi"Reg"^((T)) &= max_(phi in Phi) { sum_(t=1)^T ip(phi(vx^((t))), vu^((t))) } - sum_(t=1)^T ip(vx^((t)), vu^((t))) \
    &= max_(phi in Phi) { sum_(t=1)^T ip(phi(vx^((t))), vu^((t))) } - sum_(t=1)^T ip(phi^((t))(vx^((t))), vu^((t))),
  $ <align:phiregu>
  since $vx^((t)) = phi^((t)) (vx^((t)))$. Continuing from (@align:phiregu),
  $
    Phi"Reg"^((T)) = max_(phi in Phi) { sum_(t=1)^T u^((t))_(Phi)(phi) } - sum_(t=1)^T u^((t))_(Phi)(phi^((t))) = "Reg"^((T)).
  $
]


== The algorithm of Blum and Mansour

Finally, we will see how to make use of the previous framework to minimize swap regret when the underlying strategy set is the probability simplex---corresponding to normal-form games. The resulting algorithm was first developed by #citet(<Blum07:From>) (we also refer to a closely related algorithm due to~#citet(<Stoltz05:Internal>)). As alluded to above, the key to applying the framework of #citet(<Gordon08:No>) is to understand the structure of the set of deviations $Phi$. In this special case, it is enough to consider only _linear functions_ mapping $Delta(cA)$ to $Delta(cA)$, for which there is a simple combinatorial characterization in terms of _(column)-stochastic_ matrices.

#lemma[
  Any linear function $phi : Delta(cA) -> Delta(cA)$ can be equivalently expressed as $vx |-> matM vx$ for some stochastic matrix $matM$.
] <lemma:stochastic>

In proof, since $phi$ is linear it can be expressed as $vx |-> matM vx$ for some matrix $matM$. Now, every column of $matM$ is equal to the output of $phi$ for the probability distribution that places all the probability in the corresponding action profile. Since $phi$ maps $Delta(cA)$ to $Delta(cA)$, it follows that every column of $matM$ is a probability distribution.

Armed with the characterization of @lemma:stochastic, we now see how to implement the two oracles required in the framework of #citet(<Gordon08:No>). First, a stochastic matrix induces a Markov chain over $cA$. Any _stationary distribution_ of that Markov chain is a fixed point, and can be computed efficiently as it boils down to solving a linear system. (More broadly, when the underlying set $cX$ is a polytope and $Phi$ comprises linear deviations, computing a fixed point amounts to solving a linear program, which can be done in polynomial time.)

The next step is to minimize regret with respect to the set of stochastic matrices. By definition, the set of stochastic matrices is a product of simplices---one probability distribution for each column:
$ { [ (vx_a)_(a in cA) ] : vx_a in Delta(cA) quad forall a in cA }, $
where, for $vx, vx' in RR^(cA)$, $[ (vx, vx') ]$ denotes the matrix with columns $x$ and $x'$. Minimizing (external) regret over such a set can be accomplished by simply having an independent regret minimizer for each column.

#lemma[
  There is an efficient no-regret algorithm for minimizing external regret over the set of stochastic matrices.
]

The overall construction is given below.
#pseudocode-list(
  booktabs: true,
  title: [*Algorithm*: Swap regret minimizer  #h(1fr) <alg:BM>],
)[
  - *Input:* A regret minimizer $R_a$ for each action $a in cA$
  - `NextStrategy`():
    - *for each* action $a in cA$ *do*
      - $vx^((t))_a := R_a.$`NextStrategy`$()$
    - Set $matM^((t)) := [(vx_a^((t)))_(a in cA)]$ #h(1fr) <line:stack>
    - *return* a fixed point $vx^((t)) = matM^((t)) vx^((t))$ #h(1fr) <line:fp>
  - `ObserveUtility`($vu^((t)) in RR^(cA)$):
    - *for each* action $a in cA$ *do*
      - Set $vu_a^((t)) := vx^((t))[a] vu^((t))$ #h(1fr) <line:utilia>
      - $R_a.$`ObserveUtility`$(vu_a^((t)))$
]

It consists of $|cA|$ separate regret minimizers, $(R_a)_(a in cA)$, each of which operates over $Delta(cA)$. To obtain the next strategy, we create the stochastic matrix $matM^((t))$ in which each column is given by the strategy of the corresponding regret minimizer, and then output any fixed point of $matM^((t))$. To explain the second part of the algorithm, let's first note that the utility observed by $R_(Phi)$, per the construction in @theorem:Gordon, can be cast as $u_(Phi)(phi) = ip(phi(vx^((t))), vu^((t))) = ip(matM vx^((t)), vu^((t))) = ip(matM, vu^((t)) times.o vx^((t)))$, where we used that $phi(vx^((t))) = matM vx^((t))$. In other words, $R_(Phi)$ observes the utility vector $vu^((t)) times.o vx^((t))$. Moreover, one should forward to each $R_a$ its corresponding component, which is $vx^((t))[a] vu^((t))$. If we instantiate each $R_a$ with MWU and invoke @theorem:Gordon, we arrive at the following result.

#theorem[#citep(<Blum07:From>)][
  There is an online algorithm whose swap regret is bounded by $O(sqrt(T |cA| log |cA|))$.
] <theorem:swap>

The naive argument here would only yield a swap regret bound of $O(|cA| sqrt(T log |cA|))$ since each MWU algorithm incurs an external regret bounded by $O(sqrt(T log |cA|))$. However, one can make use of the structure of the utilities to obtain the improved bound claimed in @theorem:swap. In particular, we observe that for any $t in [T]$,
$
  sum_(a in cA) norm(vu_a^((t)))_oo^2 = norm(vu^((t)))_oo^2 sum_(a in cA) (vx^((t))[a])^2 <= norm(vu^((t)))_oo^2.
$
So, using the regret bound of MWU together with @theorem:Gordon,
$
  Phi"Reg"^((T)) <= (|cA| log |cA|) / eta + eta sum_(t=1)^T norm(vu^((t)))_oo^2 <= (|cA| log |cA|) / eta + eta T.
$
Optimizing the learning rate $eta$ gives the claim.





// #colbreak()
#lec_bibliography("../meta/refs.bib")
