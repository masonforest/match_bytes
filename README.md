`matches_bytes`
===============

`matches_bytes` is a library for matching byte patterns in slices of bytes. `matches_bytes` is inspired by Erlang's [bit sytax](https://www.erlang.org/doc/reference_manual/expressions.html#bit_syntax).

Example
-------

````rust
    let bytes = [0, 0, 0, 1, 0, 0, 0, 0];
    match_bytes!([prefix: u32 / be, rest @ ..] = &bytes);

    assert_eq!(prefix, 1);
    assert_eq!(*rest, [0, 0, 0, 0]);
````
