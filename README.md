# stellar-ibc-proposal

Public project documentation for **Stellar IBC Eureka** — trust-minimized IBC v2
(Eureka) for the Stellar network.

📖 **Read it online:** <https://amandagonsalves.github.io/stellar-ibc-proposal/>

## What's here

This repository is a [Jekyll](https://jekyllrb.com/) site built with the
[Just the Docs](https://just-the-docs.com/) theme and published via GitHub Pages.

| Page | Source |
|---|---|
| Home / overview | [`index.md`](index.md) |
| Strategy — the *why* | [`strategy.md`](strategy.md) |
| Architecture — the *how*, with sequence diagrams | [`architecture.md`](architecture.md) |
| Roadmap — deliverables from devnet to mainnet | [`roadmap.md`](roadmap.md) |

## Running locally

```sh
bundle install
bundle exec jekyll serve
# open http://localhost:4000/stellar-ibc-proposal/
```

## Publishing

A push to `main` triggers the [`Deploy Jekyll site to Pages`](.github/workflows/pages.yml)
workflow, which builds the site and deploys it to GitHub Pages.

**One-time setup:** in the repository settings, under **Settings → Pages →
Build and deployment**, set the **Source** to **GitHub Actions**.

## License

Licensed under the [Apache License 2.0](LICENSE).
