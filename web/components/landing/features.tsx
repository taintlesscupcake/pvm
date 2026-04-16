import {
  Layers,
  Link2,
  Package,
  Terminal as TerminalIcon,
  Zap,
  ShieldCheck,
} from 'lucide-react';
import { BentoCard, BentoGrid } from '@/components/magicui/bento-grid';

export function Features() {
  return (
    <section className="px-6 py-20 sm:py-28">
      <div className="mx-auto max-w-6xl">
        <div className="text-center">
          <p className="text-xs font-medium uppercase tracking-widest text-[hsl(189_94%_55%)]">
            Built for real workflows
          </p>
          <h2 className="mt-3 text-balance text-3xl font-semibold tracking-tight sm:text-4xl">
            Fast where it matters. Boring where it should be.
          </h2>
          <p className="mx-auto mt-3 max-w-2xl text-fd-muted-foreground">
            pvm keeps Python out of your way: shared environments, deduplicated
            packages, and a single binary you can drop into any machine.
          </p>
        </div>

        <BentoGrid className="mt-12">
          <BentoCard
            className="md:col-span-2"
            Icon={Layers}
            title="Shared environments, used everywhere"
            description="Create an environment once and reuse it across any project. No more .venv per repo."
          >
            <div className="flex h-full items-end gap-2 pb-6">
              {['data-science', 'web', 'ml-prod', 'scratch'].map((env) => (
                <div
                  key={env}
                  className="rounded-md border border-fd-border bg-fd-background/50 px-3 py-1 font-mono text-xs text-fd-foreground"
                >
                  {env}
                </div>
              ))}
            </div>
          </BentoCard>

          <BentoCard
            Icon={Link2}
            title="Hardlink deduplication"
            description="Identical packages stored once, hardlinked into every env. Pandas, NumPy, PyTorch — paid for once."
          >
            <div className="flex h-full items-center justify-center pb-4">
              <div className="font-mono text-2xl font-semibold brand-gradient-text">
                ~287 MB
              </div>
            </div>
          </BentoCard>

          <BentoCard
            Icon={Package}
            title="2.6 MB binary"
            description="No Python needed to install. No Conda runtime. Just one Rust binary."
          />
          <BentoCard
            Icon={TerminalIcon}
            title="Drop-in pip"
            description="`pip install` is auto-wrapped while activated — deduplicates without changing your habits."
          />
          <BentoCard
            Icon={Zap}
            title="Instant activation"
            description="No JIT, no Conda init. Activation is a shell function."
          />
          <BentoCard
            className="md:col-span-2"
            Icon={ShieldCheck}
            title="Standalone Python builds"
            description="Powered by python-build-standalone — the same prebuilt Pythons used by uv. Reproducible across machines."
          >
            <div className="mt-4 flex flex-wrap gap-1.5 pb-2">
              {['3.8', '3.9', '3.10', '3.11', '3.12', '3.13', '3.14'].map(
                (v) => (
                  <span
                    key={v}
                    className="rounded-md border border-fd-border bg-fd-background/40 px-2 py-0.5 font-mono text-[11px] text-fd-muted-foreground"
                  >
                    py {v}
                  </span>
                ),
              )}
            </div>
          </BentoCard>
        </BentoGrid>
      </div>
    </section>
  );
}
