import Link from 'next/link';
import { ArrowRight, BookOpen } from 'lucide-react';
import { Spotlight } from '@/components/aceternity/spotlight';
import { GithubIcon } from '@/components/icons/github';
import { Terminal } from '@/components/landing/terminal';
import { Button } from '@/components/ui/button';
import { gitConfig } from '@/lib/shared';

export function Hero() {
  return (
    <section className="relative isolate overflow-hidden px-6 pt-24 pb-20 sm:pt-32 sm:pb-28">
      <Spotlight />
      <div
        aria-hidden
        className="absolute inset-0 -z-20 bg-[radial-gradient(ellipse_at_top,hsl(240_6%_8%),hsl(240_6%_5%))]"
      />
      <div
        aria-hidden
        className="absolute inset-x-0 top-0 -z-10 h-px bg-gradient-to-r from-transparent via-[hsl(258_90%_66%/0.4)] to-transparent"
      />

      <div className="mx-auto max-w-5xl text-center">
        <Link
          href={`https://github.com/${gitConfig.user}/${gitConfig.repo}`}
          className="inline-flex items-center gap-2 rounded-full border border-fd-border bg-fd-card/60 px-3 py-1 text-xs text-fd-muted-foreground backdrop-blur transition-colors hover:border-[hsl(258_90%_66%/0.5)] hover:text-fd-foreground"
        >
          <span className="brand-gradient-bg size-1.5 rounded-full" />
          Open source · written in Rust
          <ArrowRight className="size-3" />
        </Link>

        <h1 className="mt-6 text-balance text-4xl font-semibold tracking-tight sm:text-6xl">
          One Python.{' '}
          <span className="brand-gradient-text">Every environment.</span>
        </h1>
        <p className="mx-auto mt-5 max-w-2xl text-balance text-base text-fd-muted-foreground sm:text-lg">
          A lightweight Python version & virtual environment manager written in
          Rust. Shared envs, hardlinked deduplication, single 2.6 MB binary —
          Anaconda&apos;s ergonomics, without the bloat.
        </p>

        <div className="mt-8 flex flex-wrap items-center justify-center gap-3">
          <Button asChild size="lg">
            <Link href="/docs">
              <BookOpen className="size-4" />
              Read the docs
            </Link>
          </Button>
          <Button asChild size="lg" variant="outline">
            <Link
              href={`https://github.com/${gitConfig.user}/${gitConfig.repo}`}
            >
              <GithubIcon className="size-4" />
              View on GitHub
            </Link>
          </Button>
        </div>
      </div>

      <div className="relative mx-auto mt-16 max-w-3xl">
        <Terminal />
      </div>
    </section>
  );
}
