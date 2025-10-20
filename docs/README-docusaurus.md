# Cloak Documentation Site

This directory hosts the Docusaurus configuration for the Cloak documentation portal.

## Prerequisites

- Node.js 18+
- pnpm, npm, or yarn (choose one package manager)

## Install

```bash
cd docs
npm install
```

> You can use `pnpm install` or `yarn install` if preferred.

## Local Development

```bash
npm run start
```

This launches the documentation site at `http://localhost:3000/` with hot reload enabled.

## Build

```bash
npm run build
```

Produces the static site output in `docs/build/`.

## Preview Production Build

```bash
npm run serve
```

Serves the previously built site from `docs/build/`.

## Directory Layout

- `docusaurus.config.ts` – site configuration referencing Markdown docs in `./docs`
- `sidebars.ts` – navigation structure covering every Cloak component
- `src/pages/index.mdx` – landing page for the documentation portal
- `src/css/custom.css` – theme customisations
- `static/` – static assets (logo, etc.)
- `docs/` – all documentation markdown files

## Next Steps

1. Ensure Node dependencies are installed (`npm install`).
2. Review the docs sidebar configuration and adjust ordering if desired.
3. Deploy the generated static site to your preferred hosting (GitHub Pages, Vercel, etc.).
