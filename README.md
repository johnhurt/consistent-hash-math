# consistent-hash-math

An interactive derivation of the formulas that describe the distribution of
work in systems that use [consistent hashing](https://en.wikipedia.org/wiki/Consistent_hashing)
to share work. It is part technical paper, part demo and part blog post, so
there is a lot of math, some WebAssembly, but also some jokes. The goal is
for it to be complete and compelling, but also approachable (and
interesting?) for any reader regardless of background.

It is a companion piece to a [blog post written for
Cloudflare](https://blog.cloudflare.com/saving-100-tb-of-ram-with-math) about
how this math was used to safely reclaim 100+ TB of RAM on the edge.

## TL;DR

The error in how evenly work is distributed with consistent hashing for $N$
servers with $k$ hashes each is

$$
    \text{Err}_k = \sqrt{\frac{N-1}{kN+1}}
$$

which is very close to the asymptotic $\mathcal{O}\left(\sqrt{1/k}\right)$
bound found in the literature. The full derivation also yields the general
formula for any server holding $k$ hashes out of $H$ total

$$
    \text{Err}_k = \sqrt{\frac{H-k}{k(H+1)}}
$$

along the way. All it takes to get there is some high-school math and a
little creativity.

## Repository layout

* `www/contents.md` - the article itself: the derivation, written in
  markdown with embedded LaTeX (rendered with KaTeX) and interactive demos
* `src/` - the Rust source for those interactive demos, compiled to
  WebAssembly with [`wasm-pack`](https://github.com/rustwasm/wasm-pack) and
  [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen)
* `www/` - the webpack site that renders the article and wires up the demos
* `release.sh` - builds the wasm package and the site for deployment

## 🚴 Usage

### 🛠️ Build with `wasm-pack build`

```
wasm-pack build
```

### 🔬 Test in Headless Browsers with `wasm-pack test`

```
wasm-pack test --headless --firefox
```

### 🏃 Run the site locally

```
cd www
npm install
npm start
```

### 📦 Build the site for deployment

```
./release.sh
```

## 🔋 Batteries Included

* [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) for
  communicating between WebAssembly and JavaScript.
* [`quill`](https://crates.io/crates/quill) and
  [`plotters-canvas`](https://crates.io/crates/plotters-canvas) for
  rendering charts and diagrams for the demos.
* [`console_error_panic_hook`](https://github.com/rustwasm/console_error_panic_hook)
  for logging panic messages to the developer console.
* `LICENSE_APACHE` and `LICENSE_MIT`: most Rust projects are licensed this
  way, so these are included for you

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE_APACHE](LICENSE_APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE_MIT](LICENSE_MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
