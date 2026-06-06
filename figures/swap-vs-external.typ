#set page(width: auto, height: auto, margin: 2pt, fill: none)
#set text(font: "New Computer Modern", size: 10.2pt)

#let body = table(
  columns: 10,
  align: center + horizon,
  stroke: 0.3mm + gray,
  fill: (col, row) => if calc.rem(col - row + 3, 3) == 2 and col > 0 { luma(80%) } else { none },
  [1], [0], [0], [1], [0], [0], [1], [0], [0], [1],
  [2], [0], [1], [0], [0], [1], [0], [0], [1], [0],
  [3], [1], [0], [0], [1], [0], [0], [1], [0], [0],
)

#body
