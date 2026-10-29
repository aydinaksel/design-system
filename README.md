# Design system

[Leptos](https://leptos.dev) 0.8 components styled with
[Tailwind CSS](https://tailwindcss.com) 4: buttons, dialogs, menus, comboboxes,
form fields, tabs, toasts, and server-driven data tables whose state lives in
the URL. Icons come from [Lucide](https://lucide.dev) through
[lepticons](https://crates.io/crates/lepticons).

| Crate | Purpose |
| --- | --- |
| `design-system` | The components. |
| `table-query` | Serializable table state (search, filters, sorting, paging), shared by the browser and the server functions that answer it. |
| `component-gallery` | A Leptos app that renders every component, for development. |

## Using it

Tailwind only generates classes it finds in the files it scans, so an app has
to import the stylesheet this crate ships, and that needs a path it can rely on.
The simplest is to check this repository out next to the app and depend on it
by path, forwarding the app's features:

```toml
[dependencies]
design-system = { path = "../design-system/crates/design-system" }

[features]
hydrate = ["design-system/hydrate"]
ssr = ["design-system/ssr"]
```

Then import its stylesheet after Tailwind, by the same relative route:

```css
@import "tailwindcss";
@import "../../design-system/crates/design-system/style/design-system.css";
```

For Nix builds, pin this repository as a non-flake input
(`github:aydinaksel/design-system/v0.1.1`) and copy it beside the app's source
before building, so the same relative paths resolve inside the sandbox.

The imported stylesheet adds the crate's sources to Tailwind's scan and
defines the animations the components use.

## Gallery

```sh
nix run .#gallery-watch
```

serves the gallery at <http://localhost:3003> and rebuilds on every change.

## Development

`nix develop` provides the toolchain. `just check` runs rustfmt, leptosfmt,
Clippy for the server and the WebAssembly targets, and the tests.

## License

MIT, see [LICENSE](LICENSE).
