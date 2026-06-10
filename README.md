# Learning and Computation of Phi-Equilibria

Tutorial notes and website source for the **ACM EC'26 Tutorial on Learning and
Computation of Phi-Equilibria**.

These notes introduce Phi-regret, its connection to Phi-equilibria in games, and
algorithmic tools for learning and computing these equilibria: fixed-point
reductions, semi-separation, ellipsoid-against-hope methods, multicalibration,
TreeSwap, and profile swap regret.

## Website

- **Tutorial website:** <https://ec26-phi-regret-tutorial.github.io/>
- **Start reading:** <https://ec26-phi-regret-tutorial.github.io/P1-introduction.html>

## Authors

- [Ioannis Anagnostides](https://www.andrew.cmu.edu/user/ianagnos/)  
  Carnegie Mellon University
- [Gabriele Farina](https://www.mit.edu/~gfarina/)  
  Massachusetts Institute of Technology
- [Brian Hu Zhang](https://brianhzhang.github.io)  
  Massachusetts Institute of Technology

## Overview

No-regret learning in games sits at the intersection of game theory and online
learning. This tutorial focuses on Phi-regret and Phi-equilibria, starting from
classical regret and equilibrium notions and building toward recent algorithmic
developments for richer strategy spaces and deviation classes.

The public site is generated from Typst source files in this repository. The
generated HTML, figures, fonts, and PDFs are committed under [`docs/`](docs/), so
the repository can be served directly with GitHub Pages.

## Tutorial Notes

| # | Chapter | Links |
|---:|---|---|
| 1 | Introduction | [HTML](https://ec26-phi-regret-tutorial.github.io/P1-introduction.html) / [PDF](https://ec26-phi-regret-tutorial.github.io/pdf/P1-introduction.pdf) |
| 2 | Beyond Normal Form | [HTML](https://ec26-phi-regret-tutorial.github.io/P2-phi_regret.html) / [PDF](https://ec26-phi-regret-tutorial.github.io/pdf/P2-phi_regret.pdf) |
| 3 | Ellipsoid Against Hope | [HTML](https://ec26-phi-regret-tutorial.github.io/P3-ellipsoid.html) / [PDF](https://ec26-phi-regret-tutorial.github.io/pdf/P3-ellipsoid.pdf) |
| 4 | Phi-Regret and Multicalibration | [HTML](https://ec26-phi-regret-tutorial.github.io/P4-multicalibration.html) / [PDF](https://ec26-phi-regret-tutorial.github.io/pdf/P4-multicalibration.pdf) |
| 5 | TreeSwap: Efficient Swap Regret Minimization | [HTML](https://ec26-phi-regret-tutorial.github.io/P5-treeswap.html) / [PDF](https://ec26-phi-regret-tutorial.github.io/pdf/P5-treeswap.pdf) |
| 6 | Profile Swap Regret, Manipulability, and Response-Based Approachability | [HTML](https://ec26-phi-regret-tutorial.github.io/P6-profile.html) / [PDF](https://ec26-phi-regret-tutorial.github.io/pdf/P6-profile.pdf) |

## Citation

The citation metadata is maintained in [`html-export.yaml`](html-export.yaml).
When citing a chapter, use:

```bibtex
@misc{anagnostides-farina-zhang-2026-phi-equilibria,
  author       = {Anagnostides, Ioannis and Farina, Gabriele and Zhang, Brian Hu},
  title        = {Learning and Computation of Phi-Equilibria},
  howpublished = {ACM EC 2026 Tutorial Notes},
  year         = {2026},
  url          = {https://ec26-phi-regret-tutorial.github.io/}
}
```
