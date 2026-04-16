# Deploying the pvm web

The `web/` app is a standard Next.js 16 App Router project. It ships static where possible and uses on-demand rendering for OG images and search.

## Vercel (recommended)

1. **Import the project**
   - Push the branch to GitHub (already done if you're reading this in `feat/web-docs-landing`).
   - In Vercel: **Add New → Project**, import the `pvm` repo.
   - **Root Directory**: `web`
   - **Framework Preset**: Next.js (auto-detected)
   - **Install Command**: `pnpm install` (auto)
   - **Build Command**: `pnpm build` (auto)
   - No env vars required.

2. **Connect the custom domain**
   - In the project's **Settings → Domains**, add `pvm.sungjin.dev`.
   - Vercel will show a CNAME record. In your DNS provider for `sungjin.dev`, add:

     ```
     Type:  CNAME
     Name:  pvm
     Value: cname.vercel-dns.com
     TTL:   3600 (or default)
     ```

   - Wait 1–5 minutes for propagation. Vercel will issue a Let's Encrypt cert automatically.

3. **Promote production branch**
   - Default is `main`. Until merged, the `feat/web-docs-landing` branch will deploy to a preview URL like `pvm-git-feat-web-docs-landing-<scope>.vercel.app`.

## Local production preview

```bash
cd web
pnpm build
pnpm start
```

Then visit http://localhost:3000.

## What gets deployed

- Static landing page at `/`
- MDX docs under `/docs/*` (statically generated at build)
- Orama-powered search at `/api/search`
- Per-page OG images at `/og/docs/[...slug]`
- Sitemap at `/sitemap.xml`, robots at `/robots.txt`
- LLM-friendly mirrors at `/llms.txt`, `/llms-full.txt`, `/llms.mdx/docs/...`

## Notes

- Dark mode is forced site-wide; there is no theme toggle.
- The site is currently English only. See the comment in `source.config.ts` for adding additional locales.
- Brand assets (favicon, root OG image) are generated dynamically by Next.js from `app/icon.tsx` and `app/opengraph-image.tsx` — no static `.ico`/`.png` files to maintain.
