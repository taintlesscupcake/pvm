'use client';

import { Check, Copy } from 'lucide-react';
import { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { cn } from '@/lib/cn';

const installCommands: Record<string, string> = {
  macos: 'curl -fsSL https://pvm.sungjin.dev/install.sh | bash',
  linux: 'curl -fsSL https://pvm.sungjin.dev/install.sh | bash',
  source:
    'git clone https://github.com/taintlesscupcake/pvm.git && cd pvm && cargo build --release && ./scripts/install.sh',
};

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      onClick={() => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 1600);
      }}
      className="absolute right-3 top-3 inline-flex size-7 items-center justify-center rounded-md border border-fd-border bg-fd-card/60 text-fd-muted-foreground transition-colors hover:text-fd-foreground"
      aria-label="Copy command"
    >
      {copied ? (
        <Check className="size-3.5 text-emerald-400" />
      ) : (
        <Copy className="size-3.5" />
      )}
    </button>
  );
}

function Code({ children, className }: { children: string; className?: string }) {
  return (
    <div className={cn('relative', className)}>
      <pre className="overflow-x-auto rounded-lg border border-fd-border bg-fd-card/60 px-4 py-4 pr-12 font-mono text-[13px] leading-6 text-fd-foreground">
        <code>{children}</code>
      </pre>
      <CopyButton text={children} />
    </div>
  );
}

export function Install() {
  return (
    <section id="install" className="px-6 py-20 sm:py-28">
      <div className="mx-auto max-w-3xl">
        <div className="text-center">
          <p className="text-xs font-medium uppercase tracking-widest text-[hsl(189_94%_55%)]">
            Get started
          </p>
          <h2 className="mt-3 text-balance text-3xl font-semibold tracking-tight sm:text-4xl">
            One line. Then you&apos;re done.
          </h2>
          <p className="mx-auto mt-3 max-w-xl text-fd-muted-foreground">
            Installs to <code className="font-mono text-fd-foreground">~/.local/bin/pvm</code> and stores state in <code className="font-mono text-fd-foreground">~/.pvm/</code>.
          </p>
        </div>

        <Tabs defaultValue="macos" className="mt-10">
          <div className="flex justify-center">
            <TabsList>
              <TabsTrigger value="macos">macOS</TabsTrigger>
              <TabsTrigger value="linux">Linux</TabsTrigger>
              <TabsTrigger value="source">Build from source</TabsTrigger>
            </TabsList>
          </div>
          <TabsContent value="macos">
            <Code>{installCommands.macos}</Code>
          </TabsContent>
          <TabsContent value="linux">
            <Code>{installCommands.linux}</Code>
          </TabsContent>
          <TabsContent value="source">
            <Code>{installCommands.source}</Code>
          </TabsContent>
        </Tabs>

        <div className="mt-6">
          <p className="mb-2 text-center text-xs uppercase tracking-widest text-fd-muted-foreground">
            Then enable shell integration
          </p>
          <Code>{`echo 'eval "$(pvm init zsh)"' >> ~/.zshrc && eval "$(pvm init zsh)"`}</Code>
        </div>
      </div>
    </section>
  );
}
