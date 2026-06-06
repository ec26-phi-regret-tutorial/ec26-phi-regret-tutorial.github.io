#import "../meta/gabri_notes.typ": *
#import "@preview/fletcher:0.5.8" as fletcher: diagram, edge, node

#set page(width: auto, height: auto, margin: (x: 10pt, y: 0pt), fill: none)
#set text(font: "New Computer Modern", size: 10.2pt)

#let body(
  ggm: text(size: 9pt)[Gordon, Greenwald, Marks [GGM08]],
  mc_from_regret: [Section 4.1],
  regret_from_mc: [Section 4.2],
) = pad(top: 1cm, bottom: 2mm, diagram(
  spacing: (50mm, 15mm),
  node-stroke: .22mm,
  node-corner-radius: .25mm,
  edge-stroke: .3mm,
  node-fill: white,
  node((0, 0), box(width: 3cm, par(justify: false)[External regret minimizer for $cH$]), name: <EH>),
  node((1, 0), box(width: 3.6cm, par(justify: false)[External regret minimizer for $Phi$]), name: <EP>),
  node((0, 1), box(width: 3cm, par(justify: false)[$cH$-multicalibrated forecaster for $cU$]), name: <MC>),
  node(
    (1, 1),
    box(width: 3.6cm, par(justify: false)[$Phi$-regret minimizer for $cX$ with utilities in $cU$]),
    name: <PHI>,
  ),
  edge(<EH>, <MC>, "-}>", label: [Expected\ VI], label-side: left),
  edge(<EP>, <PHI>, "-}>", label: [Expected\ fixed point], label-side: left),
  edge(<MC>, <PHI>, "-}>", label: [Best response], label-side: right),

  node(
    enclose: (<EP>, <PHI>),
    fill: luma(95%),
    inset: 3mm,
    stroke: (dash: "dashed"),
    label: pad(
      x: -2cm,
      top: -5cm,
    )[#ggm],
  ),
  node(
    enclose: (<EH>, <MC>),
    fill: blue.transparentize(90%),
    inset: 3mm,
    stroke: (dash: "dashed"),
    label: pad(
      top: -5cm,
    )[#mc_from_regret],
  ),
  node(
    enclose: (<MC>, <PHI>),
    fill: brown.transparentize(90%),
    inset: 1.6mm,
    stroke: (dash: "dashed"),
    label: pad(
      top: -2.1cm,
    )[#regret_from_mc],
  ),
))

#body()
