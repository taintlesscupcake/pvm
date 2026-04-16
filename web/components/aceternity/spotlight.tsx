import { cn } from '@/lib/cn';

export function Spotlight({ className }: { className?: string }) {
  return (
    <div
      aria-hidden
      className={cn(
        'pointer-events-none absolute inset-x-0 -top-20 -z-10 mx-auto h-[40rem] w-[min(88rem,100%)] brand-spotlight blur-3xl',
        className,
      )}
    />
  );
}
