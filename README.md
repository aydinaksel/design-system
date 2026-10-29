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

Add the crate as a git dependency and forward your app's features:

```toml
[dependencies]
design-system = { git = "https://github.com/aydinaksel/design-system", tag = "v0.1.1" }

[features]
hydrate = ["design-system/hydrate"]
ssr = ["design-system/ssr"]
```

Tailwind only generates classes it finds in the files it scans, so your
stylesheet has to import the one the crate ships. It sits wherever Cargo
checked the dependency out, which you can ask Cargo for:

```sh
manifest_path=$(cargo metadata --format-version 1 \
  | jq -r '.packages[] | select(.name == "design-system") | .manifest_path')
printf '@import "%s/style/design-system.css";\n' "$(dirname "$manifest_path")" \
  > style/design-system.css
```

Generate that file before each build, keep it out of version control, and
import it after Tailwind:

```css
@import "tailwindcss";
@import "./design-system.css";
```

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
