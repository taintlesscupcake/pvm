'use client';

import { motion, useReducedMotion } from 'motion/react';
import { cn } from '@/lib/cn';

interface Line {
  prompt?: string;
  promptClass?: string;
  text: string;
  className?: string;
}

const lines: Line[] = [
  { prompt: '$', text: 'pvm python install 3.12' },
  {
    text: '↓ Downloaded Python 3.12.4 · 12.4 MB',
    className: 'text-fd-muted-foreground',
  },
  { prompt: '$', text: 'pvm env create web 3.12' },
  { text: '✓ Created environment web', className: 'text-emerald-400' },
  { prompt: '$', text: 'pvm env activate web' },
  {
    prompt: '(web) $',
    promptClass: 'text-[hsl(258_90%_72%)]',
    text: 'pip install pandas numpy',
  },
  {
    text: '✓ Installed 12 packages · deduplicated 9 · saved 287 MB',
    className: 'text-emerald-400',
  },
];

export function Terminal({ className }: { className?: string }) {
  const reduceMotion = useReducedMotion();

  return (
    <div
      className={cn(
        'relative overflow-hidden rounded-xl border border-fd-border bg-fd-card/70 shadow-[0_30px_120px_-40px_hsl(258_90%_66%/0.45)] backdrop-blur',
        className,
      )}
    >
      <div className="flex items-center gap-2 border-b border-fd-border bg-fd-card/60 px-4 py-3">
        <span className="size-3 rounded-full bg-red-500/70" />
        <span className="size-3 rounded-full bg-yellow-500/70" />
        <span className="size-3 rounded-full bg-emerald-500/70" />
        <span className="ml-3 font-mono text-xs text-fd-muted-foreground">
          ~/projects · zsh
        </span>
      </div>
      <div className="relative px-5 py-4 font-mono text-[13px] leading-6">
        <div className="pointer-events-none absolute inset-0 bg-[linear-gradient(180deg,transparent_0%,hsl(240_6%_5%/0.5)_100%)]" />
        {lines.map((line, i) => (
          <motion.div
            key={i}
            initial={reduceMotion ? false : { opacity: 0, y: 6 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-80px' }}
            transition={{
              delay: reduceMotion ? 0 : i * 0.18,
              duration: 0.35,
              ease: 'easeOut',
            }}
            className="flex"
          >
            {line.prompt ? (
              <>
                <span
                  className={cn(
                    'mr-2 select-none text-[hsl(189_94%_55%)]',
                    line.promptClass,
                  )}
                >
                  {line.prompt}
                </span>
                <span className={cn('text-fd-foreground', line.className)}>
                  {line.text}
                </span>
              </>
            ) : (
              <span className={cn('text-fd-muted-foreground', line.className)}>
                {line.text}
              </span>
            )}
          </motion.div>
        ))}
      </div>
    </div>
  );
}
