Simple [marching squares](https://en.wikipedia.org/wiki/Marching_squares) implementation for `image` pixels. `core` module works with `no_std`.

# Examples

Standard library is not required for the algorithm to run:

```rust
const CAPACITY: usize = 600;
let cells = [marching_pixels::core::Cell::EMPTY; CAPACITY];

marching_pixels::core::set(
    &mut cells, 
    100, 
    100, 
    ::core::iter::repeat(true).take(100 * 100)
);

let (vertices, x_indices, y_indices) = marching_pixels::core::get(&cells, 100);
```

When `alloc` feature is enabled, some sugar code makes it easier to run the algorithm:

```rust
let mut algorithm = marching_pixels::Algorithm::with_capacity(100, 100);
let (vertices, indices) = algorithm.search(marching_pixels::Args {
    width: 100,
    height: 100,
    pixels: std::iter::repeat(true).take(100 * 100),
});
```