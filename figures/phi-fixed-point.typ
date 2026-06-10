#import "../meta/gabri_notes.typ": *
#import "eggs/common.typ": (
  arrow-black, draw-left, draw-lower, draw-upper, hairline, left-x, left-y, lower-y, pt, upper-y,
)

#set page(width: auto, height: auto, margin: 6pt, fill: none)
#set text(font: "Georgia", size: 10.2pt)

#let figure-html = sys.inputs.at("html", default: "false") == "true"
#let right-panel-x = 320

#let drawing = cetz.canvas({
  import cetz.draw: *

  draw-left(dx: left-x, dy: left-y)
  draw-upper(dx: right-panel-x, dy: upper-y)
  draw-lower(dx: right-panel-x, dy: lower-y)

  bezier(
    pt(257.4, 215.8),
    pt(321.8, 111.8),
    pt(280, 217.38),
    pt(286, 111.8),
    stroke: hairline + black,
    mark: arrow-black,
  )
  bezier(
    pt(259, 223.8),
    pt(325.8, 343),
    pt(281, 225.38),
    pt(290, 343),
    stroke: hairline + black,
    mark: arrow-black,
  )
})

#let body = if figure-html {
  image("../docs/figures/phi-fixed-point.svg")
} else {
  drawing
}

#body
