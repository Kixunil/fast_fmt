Fast fmt
========

Faster, more flexible and more correct alternative to `core::fmt` (AKA `std::fmt`)

Warning
-------

This is WIP. Some APIs may change, some may lack documentation, others may be broken. Information in this README (especially benchmarks) may be misleading. Contributions are highly appreciated!

I don't promise to work on this much for a while!

Why is this faster?
-------------------

* Lack of trait objects allows compiler to optimize better.
* Use of `size_hint` allows writers to e.g. pre-allocate large enough buffer.
* Use of never type for errors coming from `Write` allows to optimize-out error checks.

Why more flexible?
------------------

Instead of multiple traits like `Display`, `Debug`, ... this crate defines a single `Fmt<S>` which allows you to implement multiple different strategies, even your own. One possible use case is to implement `Fmt<Localizer>` to enable localization of your application.

Why more correct?
-----------------

Instead of returning `Err(())` on failed writes it returns appropriate types. It can even be `Void` to represent writers that can never fail (e.g. `std::string::String`).

How fast is it in practice?
---------------------------

The crate provides a very simple benchmark:

```
test bench::bench_core_fmt ... bench:         122 ns/iter (+/- 24)
test bench::bench_fast_fmt ... bench:          26 ns/iter (+/- 1)
```

It's consistently more than four times faster!

What to improve?
----------------

Roughly sorted by priority.

- [ ] Documentation
- [ ] Macros - ideally provide the same experience as `core` does. 
- [ ] More strategies
- [ ] More impls (especially `Fmt` for primitives)
- [ ] Bridge with `core::fmt`
- [ ] Bridge with `T: Iterator<char> + Clone`?
- [ ] Integrate with `genio` and provide encoders for different encodings.
- [ ] Support for trait objects if someone wants them
- [ ] Transformers (e.g. char escaping)
- [ ] Asynchronous formatting maybe?
- [ ] PR against `core`
- [ ] Deprecate `core::fmt`

Last two are jokes.
