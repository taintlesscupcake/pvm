import { cn } from '@/lib/cn';

export interface MarqueeProps {
  children: React.ReactNode;
  pauseOnHover?: boolean;
  className?: string;
  vertical?: boolean;
  reverse?: boolean;
  repeat?: number;
}

export function Marquee({
  children,
  pauseOnHover = false,
  className,
  vertical = false,
  reverse = false,
  repeat = 4,
}: MarqueeProps) {
  return (
    <div
      className={cn(
        'group flex overflow-hidden p-2 [--duration:40s] [--gap:1rem] [gap:var(--gap)]',
        vertical ? 'flex-col' : 'flex-row',
        className,
      )}
    >
      {Array.from({ length: repeat }).map((_, i) => (
        <div
          key={i}
          className={cn(
            'flex shrink-0 justify-around [gap:var(--gap)]',
            vertical
              ? 'animate-marquee-vertical flex-col'
              : 'animate-marquee flex-row',
            pauseOnHover && 'group-hover:[animation-play-state:paused]',
            reverse && '[animation-direction:reverse]',
          )}
        >
          {children}
        </div>
      ))}
    </div>
  );
}
