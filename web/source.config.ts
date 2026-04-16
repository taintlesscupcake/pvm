import { defineConfig, defineDocs } from 'fumadocs-mdx/config';
import { metaSchema, pageSchema } from 'fumadocs-core/source/schema';

// You can customise Zod schemas for frontmatter and `meta.json` here
// see https://fumadocs.dev/docs/mdx/collections
//
// i18n note: docs are currently English-only. To add a locale (e.g. Korean):
//   1. Move existing files into `content/docs/en/...`
//   2. Add translated tree under `content/docs/ko/...`
//   3. Pass an i18n config to `loader()` in `lib/source.ts`
//   4. Wrap routes with the `[lang]` dynamic segment
//   See https://fumadocs.dev/docs/headless/internationalization
export const docs = defineDocs({
  dir: 'content/docs',
  docs: {
    schema: pageSchema,
    postprocess: {
      includeProcessedMarkdown: true,
    },
  },
  meta: {
    schema: metaSchema,
  },
});

export default defineConfig({
  mdxOptions: {
    // MDX options
  },
});
