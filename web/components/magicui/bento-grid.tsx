import { cn } from '@/lib/cn';
import type { ComponentType, ReactNode } from 'react';

export function BentoGrid({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        'grid auto-rows-[18rem] grid-cols-1 gap-4 md:grid-cols-3',
        className,
      )}
    >
      {children}
    </div>
  );
}

export function BentoCard({
  className,
  title,
  description,
  Icon,
  children,
}: {
  className?: string;
  title: string;
  description: string;
  Icon?: ComponentType<{ className?: string }>;
  children?: ReactNode;
}) {
  return (
    <div
      className={cn(
        'group relative flex flex-col justify-between overflow-hidden rounded-xl border border-fd-border bg-fd-card/40 p-6 transition-all hover:border-[hsl(258_90%_66%/0.4)]',
        className,
      )}
    >
      <div className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(circle_at_30%_0%,hsl(258_90%_66%/0.08),transparent_60%)] opacity-0 transition-opacity duration-500 group-hover:opacity-100" />
      <div className="flex-1">{children}</div>
      <div className="space-y-1">
        {Icon && (
          <Icon className="size-5 text-fd-muted-foreground transition-colors group-hover:text-[hsl(258_90%_72%)]" />
        )}
        <h3 className="text-lg font-semibold tracking-tight">{title}</h3>
        <p className="text-sm text-fd-muted-foreground">{description}</p>
      </div>
    </div>
  );
}
