import { Check, Minus } from 'lucide-react';
import { cn } from '@/lib/cn';

const rows = [
  { feature: 'Shared environments', pvm: true, uv: false, anaconda: true },
  { feature: 'Package deduplication', pvm: true, uv: true, anaconda: false },
  { feature: 'No external dependencies', pvm: true, uv: false, anaconda: false },
  { feature: 'Single binary', pvm: '2.6 MB', uv: true, anaconda: false },
  { feature: 'Fast', pvm: true, uv: true, anaconda: false },
] as const;

type ColKey = 'pvm' | 'uv' | 'anaconda';
const cols: readonly { key: ColKey; label: string; highlight?: boolean }[] = [
  { key: 'pvm', label: 'pvm', highlight: true },
  { key: 'uv', label: 'uv / mise' },
  { key: 'anaconda', label: 'Anaconda' },
];

function Cell({ value }: { value: boolean | string }) {
  if (value === true)
    return (
      <Check className="mx-auto size-4 text-[hsl(258_90%_72%)]" />
    );
  if (value === false)
    return <Minus className="mx-auto size-4 text-fd-muted-foreground/50" />;
  return (
    <span className="font-mono text-xs text-fd-foreground">{value}</span>
  );
}

export function Comparison() {
  return (
    <section className="px-6 py-20 sm:py-28">
      <div className="mx-auto max-w-5xl">
        <div className="text-center">
          <p className="text-xs font-medium uppercase tracking-widest text-[hsl(189_94%_55%)]">
            At a glance
          </p>
          <h2 className="mt-3 text-balance text-3xl font-semibold tracking-tight sm:text-4xl">
            How pvm compares
          </h2>
        </div>

        <div className="mt-10 overflow-hidden rounded-xl border border-fd-border bg-fd-card/40 backdrop-blur">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-fd-border">
                <th className="px-6 py-4 text-left text-xs font-medium uppercase tracking-wider text-fd-muted-foreground">
                  Feature
                </th>
                {cols.map((col) => (
                  <th
                    key={col.key}
                    className={cn(
                      'px-6 py-4 text-center text-sm font-semibold',
                      col.highlight
                        ? 'brand-gradient-text'
                        : 'text-fd-muted-foreground',
                    )}
                  >
                    {col.label}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((row, i) => (
                <tr
                  key={row.feature}
                  className={cn(
                    i !== rows.length - 1 && 'border-b border-fd-border/60',
                  )}
                >
                  <td className="px-6 py-4 text-fd-foreground">
                    {row.feature}
                  </td>
                  {cols.map((col) => (
                    <td
                      key={col.key}
                      className={cn(
                        'px-6 py-4 text-center',
                        col.highlight && 'bg-[hsl(258_90%_66%/0.04)]',
                      )}
                    >
                      <Cell value={row[col.key]} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
