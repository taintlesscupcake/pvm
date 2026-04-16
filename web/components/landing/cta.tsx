import Link from 'next/link';
import { ArrowRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { GithubIcon } from '@/components/icons/github';
import { gitConfig } from '@/lib/shared';

export function CTA() {
  return (
    <section className="px-6 py-24">
      <div className="relative mx-auto max-w-4xl overflow-hidden rounded-2xl border border-fd-border bg-fd-card/40 px-8 py-16 text-center backdrop-blur">
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(circle_at_50%_0%,hsl(258_90%_66%/0.18),transparent_60%)]"
        />
        <div
          aria-hidden
          className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-[hsl(258_90%_66%/0.6)] to-transparent"
        />
        <h2 className="text-balance text-3xl font-semibold tracking-tight sm:text-4xl">
          Ready to thin out your{' '}
          <span className="brand-gradient-text">.venv graveyard</span>?
        </h2>
        <p className="mx-auto mt-4 max-w-xl text-fd-muted-foreground">
          Install pvm in 30 seconds and reclaim a few gigabytes of disk you
          didn&apos;t know you were spending.
        </p>
        <div className="mt-8 flex flex-wrap items-center justify-center gap-3">
          <Button asChild size="lg">
            <Link href="/docs/installation">
              Install now
              <ArrowRight className="size-4" />
            </Link>
          </Button>
          <Button asChild size="lg" variant="ghost">
            <Link
              href={`https://github.com/${gitConfig.user}/${gitConfig.repo}`}
            >
              <GithubIcon className="size-4" />
              Star on GitHub
            </Link>
          </Button>
        </div>
      </div>
    </section>
  );
}
