#let bb = text.with(font: "Stix Two Math")
#let BB = bb($BB$)
#let CC = bb($CC$)
#let NN = bb($NN$)
#let QQ = bb($QQ$)
#let RR = bb($RR$)
#let EE = math.op(bb($E$), limits: true)

#let matA = $bold(A)$
#let vx = $bold(x)$
#let vb = $bold(b)$

#let vu = $bold(u)$
#let vx = $bold(x)$
#let vy = $bold(y)$
#let vp = $bold(p)$
#let vc = $bold(c)$

#let matA = $upright(bold(A))$
#let matI = $upright(bold(I))$
#let matM = $upright(bold(M))$
#let matK = $upright(bold(K))$

#let cC = $cal(C)$
#let cH = $cal(H)$
#let cU = $cal(U)$
#let cA = $cal(A)$
#let cX = $cal(X)$
#let cY = $cal(Y)$

#let eps = $epsilon$

#let comment = body => text(luma(128))[~~~~ $triangle.stroked.small.r$ _ #body _]

#let EE = math.op(math.bb($E$), limits: true)
