# Call Graph

Consider the following functions:

```rust
pub fn shred() {
    gnar_gnar();
    bluebird();
}

fn gnar_gnar() {
    weather_report();
    pow();
}

fn bluebird() {
    weather_report();
}

fn weather_report() {
    shred();
}

fn pow() {
    fluffy();
    soft();
}

fn fluffy() {}

fn soft() {}

pub fn baker() {
    hood();
}

fn hood() {}
```

If we treat every function as a *vertex* in a graph, and if we add an *edge*
from *A* to *B* if function *A* calls function *B*, then we get the following
*call graph*:

[<img alt="Call Graph" src="./call-graph.svg"/>](./call-graph.svg)
