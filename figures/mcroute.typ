#import "../meta/gabri_notes.typ": *
#import "@preview/fletcher:0.5.8" as fletcher: diagram, edge, node

#set page(width: auto, height: auto, margin: (x: 10pt, y: 0pt), fill: none)
#set text(font: "Georgia", size: 10.2pt)

#let figsym = text.with(font: "STIX Two Math")
#let fH = figsym[ℋ]
#let fPhi = figsym[Φ]
#let fU = figsym[𝒰]
#let fX = figsym[𝒳]

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
  node((0, 0), box(width: 3cm, par(justify: false)[External regret minimizer for #fH]), name: <EH>),
  node((1, 0), box(width: 3.6cm, par(justify: false)[External regret minimizer for #fPhi]), name: <EP>),
  node((0, 1), box(width: 3cm, par(justify: false)[#box[#fH]-multicalibrated forecaster for #fU]), name: <MC>),
  node(
    (1, 1),
    box(width: 3.6cm, par(justify: false)[#box[#fPhi]-regret minimizer for #fX with utilities in #fU]),
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
