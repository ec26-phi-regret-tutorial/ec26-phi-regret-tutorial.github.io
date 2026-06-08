#import "../meta/gabri_notes.typ": *

#set page(width: auto, height: auto, margin: 6pt, fill: none)
#set text(font: "Georgia", size: 10.2pt)

#let u = 0.02636
#let p(x, y) = (x * u, -y * u, 0)
#let figure-html = sys.inputs.at("html", default: "false") == "true"

#let blue = rgb("#4a90e2")
#let yellow = rgb("#ffe263")
#let orange = rgb("#f5a623")

#let hairline = 0.75pt
#let heavy = 1.5pt
#let no-stroke = (paint: black.transparentize(100%), thickness: 0pt)
#let blue-stroke = (paint: blue, thickness: hairline)
#let blue-stroke-half = (paint: blue.transparentize(50%), thickness: hairline)
#let blue-heavy = (paint: blue, thickness: heavy)
#let dashed = (paint: black, thickness: hairline, dash: "dashed")
#let dotted-blue = (paint: blue, thickness: hairline, dash: "dotted")
#let dotted-orange = (paint: black, thickness: hairline, dash: "dotted")
#let arrow-blue = (end: ">", fill: blue.transparentize(50%), stroke: no-stroke, length: .20cm, width: .10cm)
#let arrow-black = (end: ">", fill: black, stroke: no-stroke, length: .28cm, width: .14cm)

#let path(cmds, stroke: no-stroke, fill: none, close: false, fill-rule: "non-zero") = {
  (
    ctx => {
      let segments = ()
      let here = none

      for cmd in cmds {
        let kind = cmd.first()

        if kind == "M" {
          here = p(cmd.at(1), cmd.at(2))
        } else if kind == "L" {
          let next = p(cmd.at(1), cmd.at(2))
          segments.push(cetz.path-util.line-segment((here, next)))
          here = next
        } else if kind == "C" {
          let c1 = p(cmd.at(1), cmd.at(2))
          let c2 = p(cmd.at(3), cmd.at(4))
          let next = p(cmd.at(5), cmd.at(6))
          segments.push(cetz.path-util.cubic-segment(here, next, c1, c2))
          here = next
        }
      }

      let drawable = cetz.drawable.path(
        segments,
        stroke: stroke,
        fill: fill,
        close: close,
        fill-rule: fill-rule,
      )
      (
        ctx: ctx,
        drawables: cetz.drawable.apply-transform(ctx.transform, drawable),
      )
    },
  )
}

#let label(x, y, body, fill: black) = {
  import cetz.draw: content
  content(p(x, y), text(fill: fill, body), anchor: "north-west")
}

#let blob(x, y) = (
  ("M", x, y),
  ("C", x + 11.48, y - 21.76, x + 29.34, y - 2.82, x + 44.86, y - 11.24),
  ("C", x + 60.37, y - 19.65, x + 71.77, y - 44.06, x + 88.42, y - 27.70),
  ("C", x + 105.07, y - 11.33, x + 90.58, y + 15.04, x + 103.61, y + 25.55),
  ("C", x + 116.64, y + 36.06, x + 124.07, y + 60.05, x + 100.61, y + 71.95),
  ("C", x + 77.15, y + 83.85, x + 75.10, y + 70.44, x + 63.93, y + 68.60),
  ("C", x + 52.77, y + 66.75, x + 43.61, y + 75.92, x + 36.61, y + 78.35),
  ("C", x + 29.61, y + 80.78, x + 20.61, y + 79.95, x + 11.41, y + 72.35),
  ("C", x + 2.21, y + 64.75, x + 9.55, y + 59.73, x + 6.21, y + 42.75),
  ("C", x + 2.87, y + 25.77, x - 11.47, y + 21.75, x, y),
)

#let drawing = cetz.canvas({
  import cetz.draw: *

  line(p(192.9, 202.7), p(208, 248.17), stroke: blue-stroke-half, mark: arrow-blue)

  path(
    (
      ("M", 514.55, 78.85),
      ("C", 534.05, 87.85, 513.4, 78.5, 533.8, 88.2),
      ("C", 532.3, 110.6, 533.3, 99.1, 531.8, 116.85),
      ("C", 520.3, 105.85, 515.8, 91.1, 514.55, 78.85),
    ),
    fill: blue.transparentize(60%),
    close: true,
  )

  path(
    (
      ("M", 502.58, 305.79),
      ("C", 508.63, 308.5, 504.38, 306.5, 510.87, 309.67),
      ("C", 513.88, 317.5, 518.38, 327.25, 523.5, 340.25),
      ("C", 525.38, 350, 526.63, 358, 529, 370.6),
      ("C", 503.63, 345.5, 502.63, 321, 502.58, 305.79),
    ),
    stroke: blue-heavy,
    fill: blue.transparentize(60%),
    close: true,
  )

  path(blob(429.99, 331.85), stroke: dashed, fill: black.transparentize(96%), close: true)

  line(
    p(510.87, 309.67),
    p(523.5, 340.25),
    p(528.5, 367.75),
    p(534.6, 406.8),
    p(540.2, 324.2),
    close: true,
    stroke: dotted-orange,
    fill: orange.transparentize(70%),
  )
  line(
    p(481.43, 295.14),
    p(458.99, 310.64),
    p(470.6, 289.8),
    close: true,
    stroke: dotted-orange,
    fill: orange.transparentize(70%),
  )

  path(blob(34.79, 226.25), stroke: dashed, fill: black.transparentize(96%), close: true)
  circle(p(94.28, 251.51), radius: 17.2 * u, stroke: hairline + black, fill: yellow)
  bezier(p(36.4, 254.2), p(66.4, 295.2), p(28.4, 272.2), p(35.4, 294.2), stroke: heavy + black)

  path(
    (
      ("M", 237.12, 253.55),
      ("C", 225.28, 263.85, 209.82, 270.08, 192.9, 270.08),
      ("C", 155.68, 270.08, 125.52, 239.92, 125.52, 202.7),
      ("C", 125.52, 176.82, 140.1, 154.35, 161.5, 143.06),
    ),
    stroke: blue-stroke,
  )
  line(p(192.9, 202.7), p(174.6, 267.4), stroke: blue-stroke-half, mark: arrow-blue)

  path(blob(423.59, 96.1), stroke: dashed, fill: black.transparentize(96%), close: true)
  circle(p(483.08, 121.11), radius: 17.2 * u, stroke: hairline + black, fill: yellow)
  bezier(p(425.2, 123.8), p(455.2, 164.8), p(417.2, 141.8), p(424.2, 163.8), stroke: heavy + black)

  circle(p(489.48, 357.11), radius: 17.2 * u, stroke: hairline + black, fill: yellow)
  bezier(p(431.6, 359.8), p(461.6, 400.8), p(423.6, 377.8), p(430.6, 399.8), stroke: heavy + black)

  line(p(588.1, 308.3), p(507.4, 279), stroke: blue-stroke-half, mark: arrow-blue)
  circle(p(588.1, 308.3), radius: 2.5 * u, stroke: no-stroke, fill: black)

  bezier(
    p(257.4, 215.8),
    p(391.8, 111.8),
    p(302.54, 217.38),
    p(321.03, 117.04),
    stroke: hairline + black,
    mark: arrow-black,
  )
  bezier(p(259, 223.8), p(395.8, 343), p(304.14, 225.38), p(324.98, 343.8), stroke: hairline + black, mark: arrow-black)

  path(
    (
      ("M", 125.75, 209.25),
      ("C", 145.25, 218.25, 124.6, 208.9, 145, 218.6),
      ("C", 143.5, 241, 144.5, 229.5, 143, 247.25),
      ("C", 131.5, 236.25, 127, 221.5, 125.75, 209.25),
    ),
    stroke: blue-heavy,
    fill: blue.transparentize(60%),
    close: true,
  )

  line(p(36.4, 254.2), p(75.4, 184.2), p(145, 218.6), p(139.4, 301.2), p(66.4, 295.2), stroke: heavy + black)

  circle(p(518.95, 92.05), radius: 2.5 * u, stroke: no-stroke, fill: blue)
  circle(p(192.9, 202.7), radius: 2.5 * u, stroke: no-stroke, fill: black)

  path(
    (
      ("M", 632.32, 359.15),
      ("C", 620.48, 369.45, 605.02, 375.68, 588.1, 375.68),
      ("C", 550.88, 375.68, 520.72, 345.52, 520.72, 308.3),
      ("C", 520.72, 286.48, 531.09, 267.08, 547.17, 254.77),
    ),
    stroke: dotted-blue,
  )

  circle(p(192.9, 202.7), radius: 47.93 * u, stroke: dotted-blue, fill: none)

  path(
    (
      ("M", 632.55, 381.99),
      ("C", 619.58, 389.84, 604.37, 394.35, 588.1, 394.35),
      ("C", 540.58, 394.35, 502.05, 355.82, 502.05, 308.3),
      ("C", 502.05, 285.12, 511.22, 264.07, 526.13, 248.6),
    ),
    stroke: blue-stroke,
  )

  line(
    p(431.6, 359.8),
    p(458.99, 310.64),
    p(481.43, 295.14),
    p(492.26, 300.49),
    p(510.87, 309.67),
    p(523.5, 340.25),
    p(528.5, 367.75),
    p(534.6, 406.8),
    p(461.6, 400.8),
    stroke: heavy + black,
  )

  line(p(137.25, 229), p(116.4, 321.4), stroke: blue-stroke)
  line(p(425.2, 123.8), p(464.2, 53.8), p(533.8, 88.2), p(528.2, 170.8), p(455.2, 164.8), stroke: heavy + black)

  path(
    (
      ("M", 626.34, 337.21),
      ("C", 617.59, 348.76, 603.71, 356.23, 588.1, 356.23),
      ("C", 561.63, 356.23, 540.17, 334.77, 540.17, 308.3),
      ("C", 540.17, 286.1, 555.27, 267.42, 575.76, 261.97),
    ),
    stroke: dotted-blue,
  )

  let phi = math.phi.alt
  label(185.8, 184.4, $phi$)
  label(582.2, 289.2, $phi$)
  label(62.5, 240, $Phi$)
  label(451, 106, $Phi$)
  label(456.5, 352.5, $Phi$)
  label(9.5, 213, $Phi_"FP"$)
  label(408, 74.5, $Phi_"FP"$)
  label(411, 312, $Phi_"FP"$)
  label(55, 178.5, $tilde(Phi)$)
  label(445.5, 41.5, $tilde(Phi)$)
  label(446, 291, $tilde(Phi)$)
  label(181.2, 249, $q$, fill: blue)
  label(518.73, 267.4, $q + delta$, fill: blue)
  label(503.78, 88.47, $phi'$, fill: blue)
  label(562.2, 92.6, [Terminate])
  label(201.4, 208.8, $q - delta$, fill: blue)
  label(95.2, 322.4, $cal(F := B)(phi, q) inter tilde(Phi)$, fill: blue)
})

#let body = if figure-html {
  image("../public/figures/phi-fixed-point.svg")
} else {
  drawing
}

#body
