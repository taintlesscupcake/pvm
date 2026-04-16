import { ArrowRight } from 'lucide-react';

const steps = [
  {
    n: '01',
    title: 'Download standalone Python',
    body: 'Prebuilt Python from python-build-standalone — the same source uv uses.',
  },
  {
    n: '02',
    title: 'Create environments centrally',
    body: 'Environments live under ~/.pvm/envs and can be activated from any directory.',
  },
  {
    n: '03',
    title: 'Pip installs go through dedup',
    body: 'A pip wrapper hardlinks identical packages from a global content-addressable store.',
  },
  {
    n: '04',
    title: 'Activate with a shell function',
    body: 'No subprocess, no init step — activation is just sourcing the env script.',
  },
];

export function HowItWorks() {
  return (
    <section className="px-6 py-20 sm:py-28">
      <div className="mx-auto max-w-5xl">
        <div className="text-center">
          <p className="text-xs font-medium uppercase tracking-widest text-[hsl(189_94%_55%)]">
            Under the hood
          </p>
          <h2 className="mt-3 text-balance text-3xl font-semibold tracking-tight sm:text-4xl">
            How it works
          </h2>
        </div>

        <ol className="mt-12 grid gap-4 sm:grid-cols-2">
          {steps.map((s) => (
            <li
              key={s.n}
              className="group relative overflow-hidden rounded-xl border border-fd-border bg-fd-card/40 p-6 transition-colors hover:border-[hsl(258_90%_66%/0.4)]"
            >
              <div className="flex items-center gap-3">
                <span className="font-mono text-xs text-fd-muted-foreground">
                  {s.n}
                </span>
                <span className="h-px flex-1 bg-fd-border" />
                <ArrowRight className="size-3.5 -translate-x-2 text-fd-muted-foreground opacity-0 transition-all group-hover:translate-x-0 group-hover:text-[hsl(258_90%_72%)] group-hover:opacity-100" />
              </div>
              <h3 className="mt-4 text-lg font-semibold tracking-tight">
                {s.title}
              </h3>
              <p className="mt-2 text-sm text-fd-muted-foreground">{s.body}</p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
