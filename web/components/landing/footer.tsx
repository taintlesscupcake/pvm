import Link from 'next/link';
import { appName, gitConfig } from '@/lib/shared';

export function Footer() {
  return (
    <footer className="border-t border-fd-border px-6 py-10">
      <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 text-sm text-fd-muted-foreground sm:flex-row">
        <div className="flex items-center gap-2">
          <span aria-hidden className="brand-gradient-bg size-2 rounded-[2px]" />
          <span className="font-semibold tracking-tight text-fd-foreground">
            {appName}
          </span>
          <span>· MIT licensed</span>
        </div>
        <nav className="flex items-center gap-5">
          <Link href="/docs" className="hover:text-fd-foreground">
            Docs
          </Link>
          <Link
            href={`https://github.com/${gitConfig.user}/${gitConfig.repo}`}
            className="hover:text-fd-foreground"
          >
            GitHub
          </Link>
          <Link
            href={`https://github.com/${gitConfig.user}/${gitConfig.repo}/issues`}
            className="hover:text-fd-foreground"
          >
            Issues
          </Link>
        </nav>
      </div>
    </footer>
  );
}
