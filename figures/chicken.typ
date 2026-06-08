#set page(width: auto, height: auto, margin: 2pt, fill: none)
#set text(font: "Georgia", size: 10.2pt)

#let body = table(
  columns: 3,
  align: center + horizon,
  stroke: none,
  [], [Stop], [Go],
  [Stop], [0, 0], [0, 1],
  [Go], [1, 0], [-5, -5],
)

#body
