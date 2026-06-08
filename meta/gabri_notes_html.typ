#import "@preview/cetz:0.3.4"
#import "@preview/cetz-plot:0.1.1"
#import "linalg.typ": *
#import "lovelace.typ": *
// #import "@preview/lovelace:0.3.1": *
#import "equate_html.typ": equate, share-align

#import "notation.typ": *

#let eps = math.epsilon.alt
#let thmcounters = state("thmcounters", (:))

#let _html-pseudo-is-not-empty(it) = {
  (
    type(it) != content
      or not (
        it.fields() == (:)
          or (it.has("children") and it.children == ())
          or (
            it.has("children") and it.children.all(c => not _html-pseudo-is-not-empty(c))
          )
          or (it.has("text") and it.text.match(regex("^\\s*$")) != none)
      )
  )
}

#let _html-pseudo-unwrap-singleton(a) = {
  while type(a) == array and a.len() == 1 {
    a = a.first()
  }
  a
}

#let _html-pseudo-transform-list(it, numbered) = {
  if not it.has("children") {
    if numbered {
      return (it,)
    } else {
      return (no-number(it),)
    }
  }

  let transformed = ()
  let non-item-child = []
  let non-item-label = none
  let items = ()

  for child in it.children {
    let f = child.func()
    if f in (enum.item, list.item) {
      items += _html-pseudo-transform-list(child.body, f == enum.item)
    } else if (
      child.func() == metadata
        and child.value.at(
          "identifier",
          default: "",
        )
          == "lovelace line label"
        and "label" in child.value
    ) {
      non-item-label = child.value.label
    } else {
      non-item-child += child
    }
  }

  if _html-pseudo-is-not-empty(non-item-child) {
    if numbered {
      transformed.push(with-line-label(non-item-label, non-item-child))
    } else {
      transformed.push(no-number(non-item-child))
    }
  }
  if items.len() > 0 {
    transformed.push(indent(..items))
  }
  transformed
}

#let _html-pseudo-render-lines(children, level: 0, closing-guides: ()) = {
  for idx in range(children.len()) {
    let child = children.at(idx)
    let is-last = idx == children.len() - 1
    if type(child) == dictionary {
      let end-guides = ()
      if is-last {
        end-guides = closing-guides
        if level > 0 {
          end-guides.push(level)
        }
      }
      html.elem("div", attrs: (
        class: "pseudo-line",
        style: "--indent:" + str(level),
      ))[
        #for i in range(level) {
          let guide = i + 1
          let class = "pseudo-guide"
          if guide in end-guides {
            class += " pseudo-guide-end"
          }
          html.elem("span", attrs: (
            class: class,
            style: "--guide:" + str(guide),
          ))[]
        }
        #html.elem("div", attrs: (class: "pseudo-text"))[#child.body]
      ]
    } else if type(child) == array {
      let child-closing-guides = ()
      if is-last {
        child-closing-guides = closing-guides
        if level > 0 {
          child-closing-guides.push(level)
        }
      }
      _html-pseudo-render-lines(
        child,
        level: level + 1,
        closing-guides: child-closing-guides,
      )
    }
  }
}

#let pseudocode-list(..config, body) = {
  let named = config.named()
  let title = named.at("title", default: none)
  let transformed = _html-pseudo-unwrap-singleton(_html-pseudo-transform-list(body, false))
  if type(transformed) != array {
    transformed = (transformed,)
  }

  html.elem("section", attrs: (class: "env algorithm"))[
    #if title != none {
      html.elem("div", attrs: (class: "env-title"))[#title]
    }
    #html.elem("div", attrs: (class: "pseudocode"))[
      #_html-pseudo-render-lines(transformed)
    ]
  ]
}

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
#let swallow = it => html.div(hidden: true, it)
#let lecture-bib = state("lecture-bib", ())
#let lecnum = counter("lecnum")
#let html-heading-tag(level) = ("h1", "h2", "h3", "h4", "h5", "h6").at(calc.min(level - 1, 5))
#let html-text(value) = if value == none { "" } else { str(value) }
#let html-math-mode = sys.inputs.at("html-math", default: "svg")
#let math-data-attrs(body, display, katex: none) = {
  let body = if body == none { [] } else { body }
  let attrs = (
    "data-typst-math": repr(body),
    "data-math-display": display,
  )
  if katex != none {
    attrs.insert("data-katex", katex)
  }
  attrs
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
  lecture-bib.update(())
  counter(heading).update(0)
  set text(font: "Georgia", size: 9.5pt)
  // set text(font: "Times New Roman", size: 10.2pt)
  set par(justify: true)
  set list(indent: 4.05mm)
  set enum(indent: 4.05mm)
  set math.equation(numbering: "(1)")
  show: equate.with(breakable: true, sub-numbering: false, number-mode: "label")
  show figure.caption: body => context [
    #html.elem("span", attrs: (class: "figcaption-label"))[
      #body.supplement #numbering(body.numbering, ..body.counter.get()).
    ]
    #body.body
  ]
  set page(
    margin: 1mm,
    numbering: none,
    width: 8.27in - 1.3in - 1.3in,
    height: auto,
    footer: none,
  )

  set cite(style: "alphanum.csl")
  set math.equation(supplement: none)
  show cite: set text(fill: blue.darken(40%))
  show strong: set text(font: "Frutiger", weight: "bold")
  show heading: it => {
    let tag = html-heading-tag(it.level)
    if it.numbering != none {
      let number = html-text(counter(heading).display())
      html.elem(tag, attrs: (
        class: "notes-heading",
        "data-level": str(it.level),
        "data-number": number,
      ))[
        #html.elem("span", attrs: (class: "secno"))[#counter(heading).display()]
        #it.body
      ]
    } else {
      html.elem(tag, attrs: (
        class: "notes-heading notes-heading-unnumbered",
        "data-level": str(it.level),
      ))[
        #it.body
      ]
    }
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
      link(
        it.element.location(),
      )[#if it.supplement == auto { it.element.supplement } else { it.supplement } #counters.at("lecture").#{ counters.at(it.element.kind, default: 0) + 1 }]
    } else {
      it
    }
  }
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
  html.elem("div", attrs: (
    class: "notes-meta",
    hidden: "",
    "data-lecture-number": html-text(lec_num),
    "data-title": if strtitle != none { html-text(strtitle) } else { html-text(title) },
  ))[]

  show math.equation.where(block: false): it => {
    html.elem(
      "span",
      attrs: (
        role: "math",
        ..math-data-attrs(it.body, "inline"),
      ),
      html.frame({
        show math.equation: eq => eq
        it
      }),
    )
  }

  let render-caption(caption) = if caption != none {
    html.elem("figcaption")[#caption]
  }

  // for styling, use `where` to assign classes for different types of figure
  show figure: it => {
    if it.kind == math.equation and it.body != none and it.body.func() == metadata {
      html.elem("span", attrs: (class: "equation-anchor", hidden: ""))[]
    } else if it.kind == "shared" {
      html.elem("section", attrs: (class: "env statement"), it.body)
    } else {
      html.elem("figure", attrs: (class: "typst"))[
        #html.elem("div", attrs: (class: "figure-body"))[
          #html.frame({
            show math.equation: eq => eq
            it.body
          })
        ]
        #render-caption(it.caption)
      ]
    }
  }

  body
}

#let citep(key) = {
  text(fill: blue.darken(40%), {
    set cite(style: "alphanum-intext.csl")
    cite(key)
  })
  lecture-bib.update(it => {
    if key not in it {
      it.push(key)
    }
    it
  })
}
#let citet(key, ..supplement) = {
  text(fill: blue.darken(40%), {
    set cite(style: "alphanum-intext.csl")
    cite(key, form: "prose", ..supplement)
  })
  lecture-bib.update(it => {
    if key not in it {
      it.push(key)
    }
    it
  })
}

#let changelog(body) = [
  #v(1cm)
  #line(length: 100%, stroke: gray)
  #set text(luma(40%))
  *Changelog*
  #set text(8pt, font: "Menlo")
  #body
]

#let lec_bibliography = (path, title: auto) => {
  show cite: set text(black)
  set heading(numbering: none)
  let bib-title = if title != none and title != auto {
    title
  } else if title == auto {
    [Bibliography for this chapter]
  } else {
    none
  }
  html.elem("section", attrs: (class: "bibliography", id: "bibliography"))[
    #if bib-title != none {
      html.elem("h1", attrs: (
        class: "notes-heading notes-heading-unnumbered",
        "data-level": "1",
      ))[#bib-title]
    }
    #context {
      html.elem("table", attrs: (class: "bibliography-table"))[
        #for item in lecture-bib.get() [
          #html.elem("tr", attrs: (class: "bibliography-row"))[
            #html.elem("td", attrs: (class: "bib-key"))[#cite(item)]
            #html.elem("td", attrs: (class: "bib-entry"))[#cite(item, form: "full")]
          ]
        ]
      ]
    }
  ]
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
#let comment(body, visual: none) = {
  html.elem("aside", attrs: (class: "special-comment"))[
    #html.elem("div", attrs: (class: "special-comment-text"))[#body]
    #if visual != none {
      html.elem("div", attrs: (class: "special-comment-figure"))[
        #html.frame({
          show math.equation: eq => eq
          visual
        })
      ]
    }
  ]
}
#let todo(body) = html.elem("mark", attrs: (class: "todo"), body)

#let alertbox(body, kind: "highlight", title: none) = {
  html.elem("aside", attrs: (
    class: "callout callout-" + kind,
    "data-callout": kind,
  ))[
    #if title != none {
      html.elem("p", attrs: (class: "callout-title"))[#title]
    }
    #html.elem("div", attrs: (class: "callout-body"))[#body]
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
        let counter_name = "shared" // name
        thmcounters.update(x => {
          x.insert(counter_name, x.at(counter_name, default: 0) + 1)
          x
        })
        html.elem("p", attrs: (class: "env-heading"))[
          #html.elem("span", attrs: (class: "env-title env-title-numbered"))[
            #html.elem("strong", attrs: (class: "env-kind"))[#Name]
            #context {
              let counters = thmcounters.get()
              html.elem("strong", attrs: (class: "env-number"))[
                #str(counters.at("lecture"))#html.elem("span", attrs: (class: "env-number-separator"))[.]#str(
                  counters.at(counter_name),
                )
              ]
            }
            #if args.pos().len() > 0 {
              html.elem("span", attrs: (class: "env-extra"))[(#args.pos().first())]
            }
            #html.elem("strong", attrs: (class: "env-punct"))[.]
          ]
        ]
        html.elem("div", attrs: (class: "env-body"))[
          #body
        ]
      },
    )
  }
}

#let proof-factory(name) = {
  let Name = name.replace(
    regex("[A-Za-z]+('[A-Za-z]+)?"),
    word => upper(word.text.first()) + lower(word.text.slice(1)),
  )

  (..args, body) => {
    html.elem("section", attrs: (class: "env proof"))[
      #html.elem("p", attrs: (class: "env-heading"))[
        #html.elem("span", attrs: (class: "env-title"))[
          #if args.pos().len() > 0 [
            _#Name #args.pos().first();._
          ] else [
            _#Name._
          ]
        ]
      ]
      #html.elem("div", attrs: (class: "env-body"))[
        #body
      ]
    ]
  }
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
#let P = text(font: "Georgia", "P")
#let PPAD = text(font: "Georgia", "PPAD")
#let NP = text(font: "Georgia", "NP")
#let coNP = text(font: "Georgia", "co-NP")
#let cone = math.op("cone")
#let nablat = math.op($tilde(nabla)#h(-1mm)$)
#let div(a, b, dgf: $phi$) = $#text(font: "Georgia", "D") _#dgf (#a mid(||) #b)$
#let divt(a, b) = $#text(font: "Georgia", "D") _(phi_t) (#a mid(||) #b)$
#let circled(body) = box(
  baseline: .6mm,
  circle(
    radius: 1.6mm,
    stroke: .15mm + luma(50%),
    inset: .3mm,
    text(size: 7pt, font: "Georgia", body),
  ),
)
#let dom = math.op("dom")
#let diag = math.op("diag")

// HTML math export currently sees Typst `cal`, `bold`, and `bb` mainly as
// `styled(child: ..., ..)`, which loses the original style command. Use
// Unicode math alphabet symbols in the shared notation macros so the KaTeX
// postprocessor receives an unambiguous representation.
#let BB = $𝔹$
#let CC = $ℂ$
#let NN = $ℕ$
#let QQ = $ℚ$
#let RR = $ℝ$
#let EE = math.op($𝔼$, limits: true)

#let matA = $𝐀$
#let matI = $𝐈$
#let matK = $𝐊$
#let matM = $𝐌$
#let matU = $𝐔$

#let va = $𝐚$
#let vb = $𝐛$
#let vc = $𝐜$
#let vp = $𝐩$
#let vq = $𝐪$
#let vs = $𝐬$
#let vu = $𝐮$
#let vx = $𝐱$
#let vy = $𝐲$
#let vz = $𝐳$

#let cA = $𝓐$
#let cC = $𝓒$
#let cH = $𝓗$
#let cK = $𝓚$
#let cS = $𝓢$
#let cU = $𝓤$
#let cX = $𝓧$
#let cY = $𝓨$
// [#math.cal("N")#h(-.8mm)#math.cal("P")]
// #let coNP = [co-#NP]
