#import "@preview/cetz:0.3.4"
#import "@preview/cetz-plot:0.1.1"
#import "linalg.typ": *
#import "lovelace.typ": *
// #import "@preview/lovelace:0.3.1": *
#import "@preview/equate:0.3.3": equate

#import "citations.typ": *
#import "notation.typ": *

#let _arrow(from, to, ..kw) = {
  cetz.draw.mark(to, (2 * to.at(0) - from.at(0), 2 * to.at(1) - from.at(1)), ..kw)
}

#let is_web = sys.inputs.at("web", default: "false") == "true"
#let eps = math.epsilon.alt
#let thmcounters = state("thmcounters", (:))

#let pseudocode-list = pseudocode-list.with(
  indentation: 1.10em,
  line-gap: 0.80em,
  hooks: .3mm,
  stroke: .2mm + gray,
  booktabs-stroke: .4mm + black,
  max-width: true,
)

#let email(addr) = {
  let w = .3
  let h = .2
  box(
    cetz.canvas({
      import cetz.draw: *
      rect((0, 0), (w, h), stroke: .2mm)
      line((0, h), (w / 2, h / 2.5), (w, h), stroke: .2mm)
    }),
  )
  [~]
  link("mailto:" + addr, raw(addr))
}

#let bpar(body) = {
  [#sym.square.filled #strong(body + ".")~~]
}
#let sf = text.with(font: "Frutiger")
#let swallow = it => place(hide(it))
#let crossrefs_source(file) = {
  read("../" + file)
    .replace(regex("#import \"meta/"), "#import \"../meta/")
    .replace(regex("#import \"figures/"), "#import \"../figures/")
    .replace(regex("#show: gabri_notes\\.with"), "#show: crossref_notes.with")
    .replace(regex("#footnote\["), "#swallow[")
}
#let crossrefs-active = state("crossrefs-active", false)
#let crossrefs(file) = if sys.inputs.at("combined", default: "false") == "false" {
  crossrefs-active.update(true)
  swallow({
    show footnote: it => {}
    show footnote.entry: it => {}
    eval(crossrefs_source(file), mode: "markup")
  })
}
#let lecture-bib = state("lecture-bib", ())
#let lecnum = counter("lecnum")

#let crossref_notes(
  body,
  lec_num: none,
  date: none,
  title: none,
  strtitle: none,
  show_outline: false,
  extrathanks: none,
) = {
  counter(heading).update(0)
  set math.equation(numbering: "(1)")
  show: equate.with(breakable: true, sub-numbering: false, number-mode: "label")
  set cite(style: "alphanum.csl")
  set math.equation(supplement: none)
  set heading(
    numbering: (..nums) => {
      str(lec_num) + "." + nums.pos().map(str).join(".")
    },
  )
  thmcounters.update((lecture: lec_num))
  let footnote = it => {}
  if str(lec_num).starts-with(regex("\d")) {
    lecnum.update(int(lec_num))
  }
  body
}

#let lecture_outline = lec_num => {
  locate(loc => {
    for elem in query(heading, loc) {
      if elem.at("numbering") != none {
        let numbering-fn = elem.at("numbering")
        let numbers = counter(heading).at(elem.location())
        let numbering = numbering(numbering-fn, ..numbers)
        let space = "    " * (numbers.len() - 1)
        if numbering.starts-with(str(lec_num) + ".") {
          (
            [#link(elem.location(), space + numbering + "   " + elem.body) #box(
                width: 1fr,
                repeat[~.~],
              )#h(3mm)#box[#align(right)[#elem.location().page()]]]
              + linebreak()
          )
        }
      }
    }
  })
}

#let gabri_notes(
  body,
  lec_num: none,
  date: none,
  title: none,
  strtitle: none,
  show_outline: false,
  extrathanks: none,
) = {
  context if not crossrefs-active.get() {
    lecture-bib.update(())
  }
  counter(heading).update(0)
  set text(font: "New Computer Modern", size: 10.2pt)
  // set text(font: "Times New Roman", size: 10.2pt)
  set par(justify: true)
  set list(indent: 4.05mm)
  set enum(indent: 4.05mm)
  set math.equation(numbering: "(1)")
  show: equate.with(breakable: true, sub-numbering: false, number-mode: "label")
  show figure.caption: body => (
    context pad(
      left: 2em,
      right: 1em,
      align(left)[
        #h(-1em)*#body.supplement #numbering(body.numbering, ..body.counter.get())*#body.separator#body.body
        // #repr(body.fields())
      ],
    )
  )
  set page(
    margin: if is_web {
      1mm
    } else {
      // (left: 1.25in, right: 1.25in, top: 1.3in, bottom: 1.3in)
      (left: 1.3in, right: 1.3in, top: 1.6in, bottom: 1.6in)
      // (left: 1.35in, right: 1.35in, top: 1.6in, bottom: 1.6in)
    },
    numbering: if is_web {
      none
    } else {
      "1"
    },
    width: if is_web {
      8.27in - 1.3in - 1.3in
    } else {
      8.27in
    },
    height: if is_web {
      auto
    } else {
      11.69in
    },
    footer: if not is_web {
      context box(stroke: none, inset: 0mm)[
        #if str(lec_num).starts-with(regex("\d")) [ Chapter #lec_num #sym.bullet ] #if strtitle != none { strtitle } else { title } #if sys.inputs.at("combined", default: "false") == "false" and false [#sym.bullet EC'26 Learning and Computation of $Phi$-Equilibria] #h(1fr)~~|~ #numbering("1", ..counter(page).get())/#numbering("1", ..counter(page).final())
      ]
    } else { none },
  )

  // set page(
  //   // margin: (left: 1.5in, right: 1.15in, top: 2in, bottom: 2in),
  //   // margin: (left: 1.15in, right: 1.15in, top: 1.75in, bottom: 1.75in),
  //   margin: (left: 1.25in, right: 1.25in, top: 1.3in, bottom: 1.3in),
  //   numbering: "1",
  //   paper: "a4",
  //   // header: context {
  //   //   h(-.6in)
  //   //   if calc.rem(counter(page).get().at(0), 2) == 1 {
  //   //     h(1fr)
  //   //   }
  //   //   sf[#numbering("1", ..counter(page).get())]
  //   // },
  // )
  // set page(margin: 1mm, numbering: none, width: 5.8in, height: auto)
  // set text(font: "PT Sans")
  // set heading(numbering: "1.1  ")
  set cite(style: "alphanum.csl")
  set math.equation(supplement: none)
  show cite: set text(fill: blue.darken(40%))
  show strong: set text(font: "Frutiger", weight: "bold")
  show heading: it => {
    // if it.level == 1 {
    //   v(8mm)
    //   grid(
    //     columns: (30%, 70%),
    //     align: top,
    //     box(baseline: .6mm, line(length: 100%, stroke: black + 1.8mm)), line(length: 100%, stroke: black + .6mm),
    //   )
    // }
    if it.numbering != none {
      v(3mm + 1mm * (2 - it.level))
      // v(1mm)
      [#h(-.4in)#box(width: .3in, fill: gray, height: (3 - it.level) * 1mm + .7mm)#h(.1in)#box(
          width: .6in,
          inset: 0mm,
          stroke: none,
        )[#strong(counter(heading).display())]#strong(it.body)]
      // v(1.5mm - it.level * 1mm)
    } else [
      // #v(2mm * (2 / it.level))
      #h(-.4in)#box(width: .3in, fill: gray, height: 2mm)#h(.1in)#strong(it.body)
      // v(1mm)
    ]
    v(0mm)
  }
  show "https://doi.org/": text(10pt, `https://doi.org/`)
  show link: it => {
    if (
      it.body.func() != raw
        and it.body.has("text")
        and (
          it.body.text.starts-with("http://")
            or it.body.text.starts-with("https://")
            or it.body.text.match(regex("^10.\d{4,9}/[-._;()/:a-zA-Z0-9]+$")) != none
        )
    ) {
      link(it.dest, text(10pt, raw(it.body.text)))
    } else {
      // it.fields()
      it
    }
  }
  show regex("arXiv:\d{4}[.]\d{4,5}(v\d+)?"): it => {
    let id = it.text.replace("arXiv:", "")
    link("https://arxiv.org/abs/" + id, it)
  }
  show "i.e.": emph
  show "e.g.": emph
  set heading(
    numbering: (..nums) => {
      str(lec_num) + "." + nums.pos().map(str).join(".")
    },
  )

  let ref-chapter-prefix = target-chapter => {
    if target-chapter != none and str(target-chapter) != str(lec_num) {
      [Chapter #target-chapter, ]
    }
  }

  show ref: it => {
    if (
      it.element != none
        and it.element.has("kind")
        and it.element.kind
          in (
            "shared",
            "theorem",
            "proposition",
            "corollary",
            "definition",
            "example",
            "remark",
            "lemma",
          )
    ) {
      let counters = thmcounters.at(it.element.location())
      let target-chapter = counters.at("lecture")
      let supplement = if it.supplement == auto {
        it.element.supplement
      } else {
        it.supplement
      }
      let number = str(target-chapter) + "." + str(counters.at(it.element.kind, default: 0) + 1)
      link(
        it.element.location(),
      )[#ref-chapter-prefix(target-chapter)#supplement~#number]
    } else if (
      it.element != none and it.element.func() == heading and it.element.at("numbering", default: none) != none
    ) {
      let numbering-fn = it.element.at("numbering")
      let numbers = counter(heading).at(it.element.location())
      let number = str(numbering(numbering-fn, ..numbers))
      let target-chapter = number.split(".").first()
      let supplement = if it.supplement == auto {
        [Section]
      } else {
        it.supplement
      }
      link(it.element.location())[#ref-chapter-prefix(target-chapter)#supplement~#number]
    } else {
      it
    }
  }
  show figure.where(kind: "shared"): set block(breakable: true)
  show figure.where(kind: "theorem"): set block(breakable: true)
  show figure.where(kind: "proposition"): set block(breakable: true)
  show figure.where(kind: "corollary"): set block(breakable: true)
  show figure.where(kind: "definition"): set block(breakable: true)
  show figure.where(kind: "example"): set block(breakable: true)
  show figure.where(kind: "remark"): set block(breakable: true)
  show figure.where(kind: "lemma"): set block(breakable: true)
  show "TODO": highlight
  show "XXX": highlight
  thmcounters.update((lecture: lec_num))

  if str(lec_num).starts-with(regex("\d")) {
    lecnum.update(int(lec_num))
  }
  [#metadata((lec_num, title)) <lecture>]
  if sys.inputs.at("combined", default: "false") == "true" {
    v(1cm)
    box(stroke: (bottom: 1.5mm + gray), inset: (y: 4mm))[
      #text(size: 16pt)[#strong[Chapter #lec_num]]#v(2mm)
      #text(size: 18pt)[*#title*]
    ]
    // v(3cm)
    v(1.9cm)
  } else {
    box(
      stroke: .5pt,
      inset: 3mm,
      width: 100%,
      radius: 0mm,
    )[
      #place(top + left)[Learning and Computation of $Phi$-Equilibria]
      #place(top + right)[EC'26 Tutorial #date]
      #v(12mm)
      #align(center)[#text(size: 16pt)[#strong[
        #if str(lec_num).starts-with(regex("\d")) [ Chapter #lec_num#v(-2mm) ] else { v(3mm) } *#title*]]]
      #v(3mm)
      Ioannis Anagnostides, Gabriele Farina, and Brian Hu Zhang#footnote(numbering: _ => sym.star.filled)[#email("ianagnos@cs.cmu.edu, {gfarina,zhangbh}@mit.edu"). These tutorial notes have not undergone formal peer review. We are grateful for any feedback or reports of typos.]
      #counter(footnote).update(0)
    ]
    v(1cm)
  }

  // if show_outline {
  //   // outline(fill: repeat([~.~]))
  //   v(3mm)
  //   lecture_outline(lec_num)
  //   line(length: 100%, stroke: gray)
  //   v(1.2cm)
  // } else {
  // }
  body
}

#let citation_register(key) = {
  lecture-bib.update(it => {
    if key not in it {
      it.push(key)
    }
    it
  })
}

#let citep(..keys) = context {
  if crossrefs-active.get() {
    []
  } else {
    let items = keys.pos()
    for key in items {
      citation_register(key)
    }
    [\[]
    for (i, key) in items.enumerate() {
      if i > 0 {
        [; ]
      }
      text(fill: blue.darken(40%), citation_label_text(key, cited_keys: lecture-bib.final()))
    }
    [\]]
  }
}

#let citet(key, ..supplement) = context {
  if crossrefs-active.get() {
    []
  } else {
    citation_register(key)
    text(fill: blue.darken(40%), citation_author_text(key))
    [ \[]
    text(fill: blue.darken(40%), citation_label_text(key, cited_keys: lecture-bib.final(), ..supplement))
    [\]]
  }
}

#let changelog(body) = [
  #v(1cm)
  #line(length: 100%, stroke: gray)
  #set text(luma(40%))
  *Changelog*
  #set text(8pt, font: "Menlo")
  #body
]

#let lec_bibliography = (path, title: auto) => context if not crossrefs-active.get() {
  show cite: set text(black)
  set heading(numbering: none)
  v(3mm)
  if title != none and title != auto {
    [= #title]
    v(2mm)
  } else if title == auto {
    [= Bibliography for this chapter]
    v(2mm)
  }
  context {
    let rows = ()
    for item in lecture-bib.get() {
      rows.push([[#citation_label_text(item, cited_keys: lecture-bib.final())]])
      rows.push(cite(item, form: "full"))
    }
    grid(columns: 2, row-gutter: 3.8mm, column-gutter: 2.3mm, ..rows)
  }
  // [
  //   // #show cite: set text(fill: red)
  //   #cnt
  // ]

  if sys.inputs.at("combined", default: "false") == "false" {
    swallow[#bibliography("refs.bib", title: none)]
  }
}

#let appendix(body) = (
  context {
    counter(heading).update(0)
    let lec_num = str(lecnum.get().at(0))
    set heading(
      numbering: (..nums) => {
        lec_num + "." + numbering("A.1", ..nums)
      },
    )
    body
  }
)
#let brown = rgb(149, 69, 53)
#let comment(body, visual: none) = text(luma(50%))[
  #sym.triangle.r #body
  #if visual != none {
    linebreak()
    align(center)[#visual]
  }
]
#let todo(body) = highlight(fill: red.lighten(50%), body)

#let alertbox(body, kind: "highlight", title: none) = {
  let fill = if kind == "warning" {
    rgb("#fff3dd")
  } else if kind == "info" {
    rgb("#eef6fb")
  } else {
    rgb("#f7f1df")
  }
  let stroke-color = if kind == "warning" {
    rgb("#c47a2c")
  } else if kind == "info" {
    rgb("#4f86a8")
  } else {
    luma(72%)
  }

  block(
    breakable: true,
    fill: fill,
    width: 100%,
    stroke: (
      left: .35mm + stroke-color,
      top: .15mm + luma(82%),
      right: .15mm + luma(82%),
      bottom: .15mm + luma(82%),
    ),
    inset: 3mm,
    radius: .65mm,
  )[
    #if title != none {
      strong(title)
      linebreak()
    }
    #body
  ]
}
#let info-box(body, title: none) = alertbox(body, kind: "info", title: title)
#let warning-box(body, title: none) = alertbox(body, kind: "warning", title: title)
#let highlight-box(body, title: none) = alertbox(body, kind: "highlight", title: title)
#let html-figure-asset(body) = body

#let graybox = block.with(
  breakable: true,
  fill: luma(95%),
  width: 100%,
  stroke: .2mm + luma(80%),
  inset: 3mm,
  radius: .65mm,
)

#let thm-factory(name) = {
  let Name = name.replace(
    regex("[A-Za-z]+('[A-Za-z]+)?"),
    word => upper(word.text.first()) + lower(word.text.slice(1)),
  )

  return (
    ..args,
    body,
  ) => {
    figure(
      kind: "shared",
      outlined: false,
      caption: none,
      supplement: Name,
      // breakable: true,
      {
        set align(left)
        let counter_name = "shared" // name
        thmcounters.update(x => {
          x.insert(counter_name, x.at(counter_name, default: 0) + 1)
          x
        })
        graybox({
          context {
            let counters = thmcounters.get()
            strong(Name)
            strong(" " + str(counters.at("lecture")))
            strong("." + str(counters.at(counter_name)))
          }
          if args.pos().len() > 0 {
            " (" + args.pos().first() + ")"
          }
          strong(".")
          [ #body]
        })
      },
    )
  }
}

#let proof-factory(name) = {
  let Name = name.replace(
    regex("[A-Za-z]+('[A-Za-z]+)?"),
    word => upper(word.text.first()) + lower(word.text.slice(1)),
  )

  (..args, body) => block(
    breakable: true,
    fill: none,
    width: 100%,
    stroke: (left: .3mm + luma(60%), right: none),
    radius: 0mm,
    inset: (left: 4mm, y: 1mm),
    {
      if args.pos().len() > 0 [
        _#Name #args.pos().first();._
      ] else [
        _#Name._
      ]
      body
      h(1fr) + $square$
    },
  )
}

#let restate = label => context {
  let content = query(label).at(0).body.child.children.at(1)
  assert(content.body.children.at(1) == strong([.]))
  graybox({
    strong(ref(label))
    [~(Restated)*.*]
    for field in content.body.children.slice(2) {
      field
    }
  })
}

#let theorem = thm-factory("theorem")
#let proposition = thm-factory("proposition")
#let lemma = thm-factory("lemma")
#let corollary = thm-factory("corollary")
#let exercise = thm-factory("exercise")
#let definition = thm-factory("definition")
#let example = thm-factory("example")
#let remark = thm-factory("remark")
#let proof = proof-factory("proof")
#let proofsketch = proof-factory("Proof Sketch")
#let solution = proof-factory("solution")

#let argmin = math.op($arg#h(1mm)min$, limits: true)
#let argmax = math.op($arg#h(1mm)max$, limits: true)

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

#let manualcite(..lbls) = {
  let rows = ()
  for lbl in lbls.pos() {
    rows.push(cite(lbl))
    rows.push(cite(lbl, form: "full"))
  }
  grid(columns: (1cm, auto), row-gutter: 3.8mm, column-gutter: 2.3mm, ..rows)
}

// #let proofdir(marker, body) = list(indent: 0mm, marker: marker, block(width: 100%, breakable: true, body))
#let proofdir(marker, body) = [#marker~~#body]

// Math notation
#let boxeq(inset: 2mm, bl: 2mm, body, punct: "") = (
  $
    #box(baseline: bl, stroke: .15mm + luma(20%), inset: ("y": inset, "x": 2mm), $display(#body)$)" "#punct
  $
)
#let qquad = $quad quad$
#let nor(pt, domain: $Omega$) = $cal(N)_(domain)(pt)$
#let span = $op("span")$
#let colspan = $op("colspan")$
#let ip(a, b) = $lr(chevron.l #a, #b chevron.r)$
#let infconv = math.op(
  box(
    baseline: .8mm,
    text(size: 7.5pt, stack(dir: ttb, $+$, v(-.4mm) + sym.or)),
  ),
)
#let opt(dir, var, obj, ..constraints) = {
  // assert(dir == math.min or dir == math.max)
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
#let P = text(font: "New Computer Modern Sans 08", "P")
#let PPAD = text(font: "New Computer Modern Sans 08", "PPAD")
#let NP = text(font: "New Computer Modern Sans 08", "NP")
#let coNP = text(font: "New Computer Modern Sans 08", "co-NP")
#let cone = math.op("cone")
#let cK = $cal(K)$
#let nablat = math.op($tilde(nabla)#h(-1mm)$)
#let div(a, b, dgf: $phi$) = $#text(font: "New Computer Modern", "D") _#dgf (#a mid(||) #b)$
#let divt(a, b) = $#text(font: "New Computer Modern", "D") _(phi_t) (#a mid(||) #b)$
#let circled(body) = box(
  baseline: .6mm,
  circle(
    radius: 1.6mm,
    stroke: .15mm + luma(50%),
    inset: .3mm,
    text(size: 7pt, font: "New Computer Modern Sans 08", body),
  ),
)
#let dom = math.op("dom")
#let diag = math.op("diag")
// [#math.cal("N")#h(-.8mm)#math.cal("P")]
// #let coNP = [co-#NP]
