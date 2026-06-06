#import "@preview/polylux:0.4.0": *
#import "@preview/cetz:0.3.2"
#import "@preview/cetz-plot:0.1.1"
#import "boxes.typ": *

#let gabri_slides(doc) = {
  // Make the paper dimensions fit for a presentation and the text larger
  set page(paper: "presentation-4-3", margin: 1cm)
  set text(size: 22pt, font: "PT Sans")
  // show math.equation: set text(font: "NewComputerModernSans08")
  set block(above: 10mm, below: 10mm)
  set list(tight: false, spacing: auto)
  // set list(marker: [---])
  show heading: it => {
    if it.level == 2 {
      text(fill: blue)[#it]
      v(-7mm)
      line(stroke: .6mm, length: 100%)
      v(5mm)
    } else {
      it
    }
  }
  doc
}

#let slide = slide;

#let example(body) = {
  colorbox(body, title: "Example", color: "blue", radius: 3mm, width: auto)
}

#let definition(body) = {
  colorbox(body, title: "Definition", color: "blue", radius: 3mm, width: auto)
}

#let theorem(body) = {
  colorbox(body, title: "Theorem", color: "blue", radius: 3mm, width: auto)
}

#let warning(body) = {
  colorbox(body, title: "Warning", color: "orange", radius: 3mm, width: auto)
}

#let spoiler(body) = {
  colorbox(body, title: "Spoiler", color: "teal", radius: 3mm, width: auto)
}

#let argmin = math.op($arg#h(1mm)min$, limits: true)

#let dt(s) = {
  (
    [#s]
      + if s.last() == "1" and (not s.ends-with(" 1") and not s.ends-with("11")) {
        [#super[st]]
      } else if s.last() == "2" and not s.ends-with("12") {
        [#super[nd]]
      } else if s.last() == "3" and not s.ends-with("13") {
        [#super[rd]]
      } else {
        [#super[th]]
      }
  )
}

#let proofdir(marker, body) = list(indent: 0mm, marker: marker, box(width: 100%, body))

// Math notation
#let boxeq(inset: 2mm, bl: 2mm, body, punct: "") = (
  $
    #box(
  baseline: bl,
  stroke: .2mm,
  inset: ("y": inset, "x": 2mm),
  $display(#body)$,

)" "#punct
  $
)
#let qquad = $quad quad$
#let nor(pt, domain: $Omega$) = $cal(N)_(#h(-.2em)domain)(pt)$
#let span = $op("span")$
#let colspan = $op("colspan")$
#let ip(a, b) = $lr(angle.l #a, #b angle.r)$
#let infconv = math.op(
  box(
    baseline: .8mm,
    text(size: 7.5pt, stack(dir: ttb, $+$, v(-.4mm) + sym.or)),
  ),
)
#let opt(dir, var, obj, ..constraints) = {
  assert(dir == math.min or dir == math.max)
  let data = (($limits(dir)_(var)$, $&obj$),)
  for (i, cntnt) in constraints.pos().enumerate(start: 0) {
    if i == 0 {
      data.push(("s.t.", $&$ + cntnt))
    } else {
      data.push(("", $&$ + cntnt))
    }
  }
  math.mat(delim: none, ..data)
}
#let P = text(font: "New Computer Modern Sans", "P")
#let NP = text(font: "New Computer Modern Sans", "NP")
#let coNP = text(font: "New Computer Modern Sans", "co-NP")
#let cone = math.op("cone")
#let cK = text(font: "Latin Modern Math", "𝒦")
#let div(a, b) = $#text(font: "New Computer Modern", "D")_phi (#a mid(||) #b)$
// [#math.cal("N")#h(-.8mm)#math.cal("P")]
// #let coNP = [co-#NP]
#let nablat = math.op($tilde(nabla)#h(-1mm)$)
