#import "meta/gabri_notes.typ": *
#show: gabri_notes.with(lec_num: 5, date: none, title: "TreeSwap: Efficient Swap Regret Minimization")

#show "TreeSwap": `TreeSwap`

So far, we have seen that no-regret learning, and efficient equilibrium computation, are essentially possible whenever the set of deviations $Phi$ is "efficiently representable", in particular,  whenever _external_ regret minimization is possible over $Phi$. What about beyond? In particular, we have not yet addressed the _strongest_ possible notion of $Phi$-regret, namely, the case where $Phi$ contains _all_ functions $phi: cX -> cX$---known as _swap regret_.

The fact that algorithms for $Phi$-regret minimization grow increasingly complex as $Phi$ itself grows in complexity may lead one to believe that efficient swap regret minimization is hopeless. For example, the set of all functions $phi : cX -> cX$ has _infinite_ dimension, so the techniques we have covered so far do not work. If $cX$ is a polytope, we could run the Blum-Mansour algorithm over its vertices---this would indeed succeed, but it has the drawback that the number of vertices of $cX$ is generally not polynomial in the dimension $d$ of $cX$, and hence the algorithm is not efficient. Hence, for a long time, the possibility of swap regret minimization---and correspondingly, correlated equilibrium computation beyond normal-form games---remained a major open question.

The question has since essentially been fully resolved. In a simultaneous major breakthrough, #citet(<Peng24:Fast>) and #citet(<Dagan24:From>), using essentially an identical algorithm now known as TreeSwap, showed that, if there is a regret minimizer on $cX$ that achieves _external_ regret $eps$ after $M$ rounds, then there is a regret minimizer on $cX$ that achieves $eps T$ _swap_ regret after $M^(1\/eps)$ rounds, and therefore efficient swap regret minimization _ks_ in fact possible, at least when $eps$ is a constant. They also showed nearly-matching lower bounds for normal-form games, which #citet(<Daskalakis24:Lower>) later extended to extensive-form games as well, thereby precluding the possibility of $"poly"(d, 1\/eps)$-time algorithms for swap regret beyond normal-form games. In this chapter, we will go over the upper bound.

= The TreeSwap Algorithm

We now describe the TreeSwap algorithm, following #citet(<Peng24:Fast>) and #citet(<Dagan24:From>). Let $cal(R)$ be a no-regret algorithm on $cX$ whose regret is bounded by $eps T$ after $M$ rounds. Of course, $cal(R)$ will not necessarily have small swap regret. So, how can we hope to bound swap regret?

One thing to try might be the following. Instead of having $cal(R)$ update its strategy on _every_ iteration, it will be lazy, and only update its strategy every $M$ iterations, using the average utility on those $M$ iterations. That is, at time $M$, it receives utility $1/M sum_(t=1)^M vu^((t))$, at time $2M$ it receives utility $1/M sum_(t=M+1)^(2M) vu^((t))$, and so on. Then, between those $M$ iterations, that is, while $cal(R)$ is playing the same strategy, we initialize _another copy of the same regret minimizer_, which we will call $cal(R)'$, which updates its strategy on _every_ iteration.

The purpose of $cal(R)'$ is to bound the value of the most profitable deviation from every single strategy that $cal(R)$ plays. Therefore, $cal(R)'$ can ensure that no strategy played by $cal(R)$ incurs large swap regret. Of course, $cal(R)'$ itself will have high swap regret... but this can be solved by introducing yet more layers!

The TreeSwap algorithm uses $K$ layers of regret minimizers $cal(R)_0, dots, cal(R)_(K-1)$, each responsible for ensuring that there is no profitable swap deviation for the regret minimizers in the next layer. $cal(R)_0$ updates on every iteration. Each regret minimizer $cal(R)_k$ updates $M$ times slower than the previous regret minimizer $cal(R)_(k-1)$, and resets every $M$ updates. Hence, this algorithm runs for $M^K$ steps. The strategy output by TreeSwap is the uniform distribution (not to be confused with the average) over the strategies $\(vx^((t))_0, dots, vx^((t))_(K-1))$ output by the $K$ regret minimizers. For simplicity of notation, we will  zero-index timesteps and use the notation $[n] = {0, ..., n-1}$.

#pseudocode-list(
  booktabs: true,
  title: [*Algorithm*: TreeSwap  #h(1fr) <alg:treeswap>],
)[
  - *Input:* Regret minimizers $cal(R)_1, dots, cal(R)_K$
  - *for* timesteps $t in [T] := [M^K]$ *do*
    - *for* layers $k in [K]$ *do*
      - *if* $M^(k+1) | t$ *then* reset $cal(R)_k$
      - *else if* $M^k | t$ *then* pass to $cal(R)_k$ the average of the last $M^k$ utilities, $ 1/(M^k)sum_(tau=t - M^k)^(t-1) vu^((tau)) $
      - $vx^((t))_k <-$ current strategy of $cal(R)_k$
    - play mixed strategy $"Unif"\(vx^((t))_0, dots, vx^((t))_(K-1)) in Delta(cX)$
    - receive utility $vu^((t)) in RR^d$
]
#theorem[
  The regret of TreeSwap is bounded by $T dot.c (eps + 1\/K)$. In particular, taking $K = 1\/eps$ and $M = "poly"(d, 1\/eps)$ (as is achieved by any reasonable external-regret-minimizing algorithm), TreeSwap achieves swap regret $eps T$ after $M^K = d^(tilde(O)(1\/epsilon))$ iterations.
] <theorem:treeswap>

#remark[
  @theorem:treeswap only applies when $T$ is a power of $M$. Handling the case where $T$ is not a power of $M$ requires a bit of care and results in a looser bound of $T dot.c (eps + 3\/K)$, but the main ideas are captured by the above result and its proof. #citep(<Dagan24:From>)
]

#remark[
  We are mainly interested in the implications of @theorem:treeswap for its implications for swap regret in high-dimensional settings. However, it is worth noting that @theorem:treeswap achieves a new result even in the normal-form setting: its regret bound for normal form, when instantiated with multiplicative weights, is $log(N)^(tilde(O)(1\/eps))$, which, for large $N$ and $eps$, is better than the bound of Blum-Mansour.
]

We now sketch a proof of @theorem:treeswap.

#proofsketch[
  In this proof, timesteps are zero-indexed, and $[n] := {0, ..., n-1}$.
  Let $phi : cX -> cX$ be any function.
  Then the (time-averaged) regret against $phi$ is given by
  $
    1/T "Reg"(T, phi) &= 1/K sum_(k=0)^(K-1) EE_(t in [T]) ip(vu^((t)), phi \(vx^((t))_k\)) - ip(vu^((t)), vx^((t))_k) \
    &= 1/K sum_(k=0)^(K-1) EE_(t in [T]) ip(vu^((t)), phi \(vx^((t))_k\)) - 1/K sum_(k=1)^(K) EE_(t in [T]) ip(vu^((t)), vx^((t))_(k-1)) \
    &= 1/K sum_(k=1)^(K-1) underbrace(EE_(t in [T]) ip(vu^((t)), phi \(vx^((t))_k\) - vx^((t))_(k-1)), <= eps) + 1/K underbrace(EE_(t in [T])ip(vu^((t)), phi \(vx^((t))_0\) - vx^((t))_(K-1)), <= 1)\
    & <= eps + 1/K
  $
  where, in the second-to-last line, the bound on the first term comes from the regret bound of each phase between resets (of length $M$ updates), noting that $phi \(vx^((t))_k\)$ is constant within any given phase and thus the regret against $phi \(vx^((t))_k\)$̧ is bounded by the external regret during the phase.
]


// #colbreak()
#lec_bibliography("../meta/refs.bib")
