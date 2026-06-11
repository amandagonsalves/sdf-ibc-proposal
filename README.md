# stellar-ibc-proposal

Public project documentation for **Stellar IBC Eureka** — trust-minimized IBC v2
(Eureka) for the Stellar network.

📖 **Read it online** at this repository's GitHub Pages URL
(`https://<owner>.github.io/<repo>/`).

> **Source availability.** The implementation is currently in a private
> repository while it is under active development, and will be **open-sourced**
> once it stabilizes. This repository is the public documentation of the
> project's design, rationale, and roadmap in the meantime.

## What's here

A self-contained [Jekyll](https://jekyllrb.com/) site with a custom dark theme
(no external theme gem), published via GitHub Pages. All links and assets use
relative paths, so the site works under any repository name without
configuration.

| Page | Source |
|---|---|
| Home / overview | [`index.md`](index.md) |
| Strategy — the *why* | [`strategy.md`](strategy.md) |
| Architecture — the *how*, with sequence diagrams | [`architecture.md`](architecture.md) |
| Roadmap — deliverables from devnet to mainnet | [`roadmap.md`](roadmap.md) |

Layout and styling live in [`_layouts/`](_layouts), [`assets/css/style.css`](assets/css/style.css),
and [`assets/js/site.js`](assets/js/site.js) (mermaid rendering + mobile nav).

## Running locally

```sh
bundle install
bundle exec jekyll serve
# open http://localhost:4000/
```

## Publishing

This site builds with GitHub's default Pages builder. In the repository settings,
under **Settings → Pages → Build and deployment**, set the **Source** to
**Deploy from a branch** (branch `main`, folder `/ (root)`). Every push to `main`
rebuilds and deploys automatically — no workflow or theme gem required.

## License

Licensed under the [Apache License 2.0](LICENSE).
