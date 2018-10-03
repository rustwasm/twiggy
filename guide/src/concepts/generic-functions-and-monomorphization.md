# Generic Functions and Monomorphization

Generic functions with type parameters in Rust and template functions in C++ can
lead to code bloat if you aren't careful. Every time you instantiate these
generic functions with a concrete set of types, the compiler will *monomorphize*
the function, creating a copy of its body replacing its generic placeholders
with the specific operations that apply to the concrete types. This presents
many opportunities for compiler optimizations based on which particular concrete
types each copy of the function is working with, but these copies add up quickly
in terms of code size.

Example of monomorphization in Rust:

```rust
fn generic_function<T: MyTrait>(t: T) { ... }

// Each of these will generate a new copy of `generic_function`!
generic_function::<MyTraitImpl>(...);
generic_function::<AnotherMyTraitImpl>(...);
generic_function::<MyTraitImplAlso>(...);
```

Example of monomorphization in C++:

```c++
template<typename T>
void generic_function(T t) { ... }

// Each of these will also generate a new copy of `generic_function`!
generic_function<uint32_t>(...);
generic_function<bool>(...);
generic_function<MyClass>(...);
```

If you can afford the runtime cost of dynamic dispatch, then changing these
functions to use trait objects in Rust or virtual methods in C++ can likely save
a significant amounts of code size. With dynamic dispatch, the generic
function's body is not copied, and the generic bits within the function become
indirect function calls.

Example of dynamic dispatch in Rust:

```rust
fn generic_function(t: &MyTrait) { ... }
// or
fn generic_function(t: Box<MyTrait>) { ... }
// etc...

// No more code bloat!
let x = MyTraitImpl::new();
generic_function(&x);
let y = AnotherMyTraitImpl::new();
generic_function(&y);
let z = MyTraitImplAlso::new();
generic_function(&z);
```

Example of dynamic dispatch in C++:

```c++
class GenericBase {
  public:
    virtual void generic_impl() = 0;
};

class MyThing : public GenericBase {
  public
    virtual void generic_impl() override { ... }
};

class AnotherThing : public GenericBase {
  public
    virtual void generic_impl() override { ... }
};

class AlsoThing : public GenericBase {
  public
    virtual void generic_impl() override { ... }
};

void generic(GenericBase& thing) { ... }

// No more code bloat!
MyThing x;
generic(x);
AnotherThing y;
generic(y);
AlsoThing z;
generic(z);
```

`twiggy` can analyze a binary to find which generic functions are being
monomorphized repeatedly, and calculate an estimation of how much code size
could be saved by switching from monomorphization to dynamic dispatch.
