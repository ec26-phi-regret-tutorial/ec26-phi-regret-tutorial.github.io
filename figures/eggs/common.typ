#import "../../meta/gabri_notes.typ": *

#let u = 0.02636
#let pt(x, y, dx: 0, dy: 0) = ((x + dx) * u, -(y + dy) * u, 0)

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

#let left-x = 0
#let left-y = 140
#let upper-x = 390
#let upper-y = 35
#let lower-x = 390
#let lower-y = 245

#let phi = math.phi.alt

#let path(cmds, dx: 0, dy: 0, stroke: no-stroke, fill: none, close: false, fill-rule: "non-zero") = {
  (
    ctx => {
      let segments = ()
      let here = none

      for cmd in cmds {
        let kind = cmd.first()

        if kind == "M" {
          here = pt(cmd.at(1), cmd.at(2), dx: dx, dy: dy)
        } else if kind == "L" {
          let next = pt(cmd.at(1), cmd.at(2), dx: dx, dy: dy)
          segments.push(cetz.path-util.line-segment((here, next)))
          here = next
        } else if kind == "C" {
          let c1 = pt(cmd.at(1), cmd.at(2), dx: dx, dy: dy)
          let c2 = pt(cmd.at(3), cmd.at(4), dx: dx, dy: dy)
          let next = pt(cmd.at(5), cmd.at(6), dx: dx, dy: dy)
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

#let label(x, y, body, dx: 0, dy: 0, fill: black) = {
  import cetz.draw: content
  content(pt(x, y, dx: dx, dy: dy), text(fill: fill, body), anchor: "north-west")
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

#let draw-left(dx: 0, dy: 0) = {
  import cetz.draw: *
  let ox = dx - left-x
  let oy = dy - left-y

  line(pt(192.9, 202.7, dx: ox, dy: oy), pt(208, 248.17, dx: ox, dy: oy), stroke: blue-stroke-half, mark: arrow-blue)
  path(blob(34.79, 226.25), dx: ox, dy: oy, stroke: dashed, fill: black.transparentize(96%), close: true)
  circle(pt(94.28, 251.51, dx: ox, dy: oy), radius: 17.2 * u, stroke: hairline + black, fill: yellow)
  bezier(
    pt(36.4, 254.2, dx: ox, dy: oy),
    pt(66.4, 295.2, dx: ox, dy: oy),
    pt(28.4, 272.2, dx: ox, dy: oy),
    pt(35.4, 294.2, dx: ox, dy: oy),
    stroke: heavy + black,
  )
  path(
    (
      ("M", 237.12, 253.55),
      ("C", 225.28, 263.85, 209.82, 270.08, 192.9, 270.08),
      ("C", 155.68, 270.08, 125.52, 239.92, 125.52, 202.7),
      ("C", 125.52, 176.82, 140.1, 154.35, 161.5, 143.06),
    ),
    dx: ox,
    dy: oy,
    stroke: blue-stroke,
  )
  line(pt(192.9, 202.7, dx: ox, dy: oy), pt(174.6, 267.4, dx: ox, dy: oy), stroke: blue-stroke-half, mark: arrow-blue)
  path(
    (
      ("M", 125.75, 209.25),
      ("C", 145.25, 218.25, 124.6, 208.9, 145, 218.6),
      ("C", 143.5, 241, 144.5, 229.5, 143, 247.25),
      ("C", 131.5, 236.25, 127, 221.5, 125.75, 209.25),
    ),
    dx: ox,
    dy: oy,
    stroke: blue-heavy,
    fill: blue.transparentize(60%),
    close: true,
  )
  line(
    pt(36.4, 254.2, dx: ox, dy: oy),
    pt(75.4, 184.2, dx: ox, dy: oy),
    pt(145, 218.6, dx: ox, dy: oy),
    pt(139.4, 301.2, dx: ox, dy: oy),
    pt(66.4, 295.2, dx: ox, dy: oy),
    stroke: heavy + black,
  )
  circle(pt(192.9, 202.7, dx: ox, dy: oy), radius: 2.5 * u, stroke: no-stroke, fill: black)
  circle(pt(192.9, 202.7, dx: ox, dy: oy), radius: 47.93 * u, stroke: dotted-blue, fill: none)
  line(pt(137.25, 229, dx: ox, dy: oy), pt(116.4, 321.4, dx: ox, dy: oy), stroke: blue-stroke)

  label(185.8, 184.4, $phi$, dx: ox, dy: oy)
  label(62.5, 240, $Phi$, dx: ox, dy: oy)
  label(9.5, 213, $Phi_"FP"$, dx: ox, dy: oy)
  label(55, 178.5, $tilde(Phi)$, dx: ox, dy: oy)
  label(181.2, 249, $q$, dx: ox, dy: oy, fill: blue)
  label(201.4, 208.8, $q - delta$, dx: ox, dy: oy, fill: blue)
  label(95.2, 322.4, $cal(F := B)(phi, q) inter tilde(Phi)$, dx: ox, dy: oy, fill: blue)
}

#let draw-upper(dx: 0, dy: 0) = {
  import cetz.draw: *
  let ox = dx - upper-x
  let oy = dy - upper-y

  path(
    (
      ("M", 514.55, 78.85),
      ("C", 534.05, 87.85, 513.4, 78.5, 533.8, 88.2),
      ("C", 532.3, 110.6, 533.3, 99.1, 531.8, 116.85),
      ("C", 520.3, 105.85, 515.8, 91.1, 514.55, 78.85),
    ),
    dx: ox,
    dy: oy,
    fill: blue.transparentize(60%),
    close: true,
  )
  path(blob(423.59, 96.1), dx: ox, dy: oy, stroke: dashed, fill: black.transparentize(96%), close: true)
  circle(pt(483.08, 121.11, dx: ox, dy: oy), radius: 17.2 * u, stroke: hairline + black, fill: yellow)
  bezier(
    pt(425.2, 123.8, dx: ox, dy: oy),
    pt(455.2, 164.8, dx: ox, dy: oy),
    pt(417.2, 141.8, dx: ox, dy: oy),
    pt(424.2, 163.8, dx: ox, dy: oy),
    stroke: heavy + black,
  )
  circle(pt(518.95, 92.05, dx: ox, dy: oy), radius: 2.5 * u, stroke: no-stroke, fill: blue)
  line(
    pt(425.2, 123.8, dx: ox, dy: oy),
    pt(464.2, 53.8, dx: ox, dy: oy),
    pt(533.8, 88.2, dx: ox, dy: oy),
    pt(528.2, 170.8, dx: ox, dy: oy),
    pt(455.2, 164.8, dx: ox, dy: oy),
    stroke: heavy + black,
  )

  label(451, 106, $Phi$, dx: ox, dy: oy)
  label(408, 74.5, $Phi_"FP"$, dx: ox, dy: oy)
  label(445.5, 41.5, $tilde(Phi)$, dx: ox, dy: oy)
  label(503.78, 88.47, $phi'$, dx: ox, dy: oy, fill: blue)
  label(562.2, 92.6, [Terminate], dx: ox, dy: oy)
}

#let draw-lower(dx: 0, dy: 0) = {
  import cetz.draw: *
  let ox = dx - lower-x
  let oy = dy - lower-y

  path(
    (
      ("M", 502.58, 305.79),
      ("C", 508.63, 308.5, 504.38, 306.5, 510.87, 309.67),
      ("C", 513.88, 317.5, 518.38, 327.25, 523.5, 340.25),
      ("C", 525.38, 350, 526.63, 358, 529, 370.6),
      ("C", 503.63, 345.5, 502.63, 321, 502.58, 305.79),
    ),
    dx: ox,
    dy: oy,
    stroke: blue-heavy,
    fill: blue.transparentize(60%),
    close: true,
  )
  path(blob(429.99, 331.85), dx: ox, dy: oy, stroke: dashed, fill: black.transparentize(96%), close: true)
  line(
    pt(510.87, 309.67, dx: ox, dy: oy),
    pt(523.5, 340.25, dx: ox, dy: oy),
    pt(528.5, 367.75, dx: ox, dy: oy),
    pt(534.6, 406.8, dx: ox, dy: oy),
    pt(540.2, 324.2, dx: ox, dy: oy),
    close: true,
    stroke: dotted-orange,
    fill: orange.transparentize(70%),
  )
  line(
    pt(481.43, 295.14, dx: ox, dy: oy),
    pt(458.99, 310.64, dx: ox, dy: oy),
    pt(470.6, 289.8, dx: ox, dy: oy),
    close: true,
    stroke: dotted-orange,
    fill: orange.transparentize(70%),
  )
  circle(pt(489.48, 357.11, dx: ox, dy: oy), radius: 17.2 * u, stroke: hairline + black, fill: yellow)
  bezier(
    pt(431.6, 359.8, dx: ox, dy: oy),
    pt(461.6, 400.8, dx: ox, dy: oy),
    pt(423.6, 377.8, dx: ox, dy: oy),
    pt(430.6, 399.8, dx: ox, dy: oy),
    stroke: heavy + black,
  )
  line(pt(588.1, 308.3, dx: ox, dy: oy), pt(507.4, 279, dx: ox, dy: oy), stroke: blue-stroke-half, mark: arrow-blue)
  circle(pt(588.1, 308.3, dx: ox, dy: oy), radius: 2.5 * u, stroke: no-stroke, fill: black)
  path(
    (
      ("M", 632.32, 359.15),
      ("C", 620.48, 369.45, 605.02, 375.68, 588.1, 375.68),
      ("C", 550.88, 375.68, 520.72, 345.52, 520.72, 308.3),
      ("C", 520.72, 286.48, 531.09, 267.08, 547.17, 254.77),
    ),
    dx: ox,
    dy: oy,
    stroke: dotted-blue,
  )
  path(
    (
      ("M", 632.55, 381.99),
      ("C", 619.58, 389.84, 604.37, 394.35, 588.1, 394.35),
      ("C", 540.58, 394.35, 502.05, 355.82, 502.05, 308.3),
      ("C", 502.05, 285.12, 511.22, 264.07, 526.13, 248.6),
    ),
    dx: ox,
    dy: oy,
    stroke: blue-stroke,
  )
  line(
    pt(431.6, 359.8, dx: ox, dy: oy),
    pt(458.99, 310.64, dx: ox, dy: oy),
    pt(481.43, 295.14, dx: ox, dy: oy),
    pt(492.26, 300.49, dx: ox, dy: oy),
    pt(510.87, 309.67, dx: ox, dy: oy),
    pt(523.5, 340.25, dx: ox, dy: oy),
    pt(528.5, 367.75, dx: ox, dy: oy),
    pt(534.6, 406.8, dx: ox, dy: oy),
    pt(461.6, 400.8, dx: ox, dy: oy),
    stroke: heavy + black,
  )
  path(
    (
      ("M", 626.34, 337.21),
      ("C", 617.59, 348.76, 603.71, 356.23, 588.1, 356.23),
      ("C", 561.63, 356.23, 540.17, 334.77, 540.17, 308.3),
      ("C", 540.17, 286.1, 555.27, 267.42, 575.76, 261.97),
    ),
    dx: ox,
    dy: oy,
    stroke: dotted-blue,
  )

  label(582.2, 289.2, $phi$, dx: ox, dy: oy)
  label(456.5, 352.5, $Phi$, dx: ox, dy: oy)
  label(411, 312, $Phi_"FP"$, dx: ox, dy: oy)
  label(446, 291, $tilde(Phi)$, dx: ox, dy: oy)
  label(518.73, 267.4, $q + delta$, dx: ox, dy: oy, fill: blue)
}
