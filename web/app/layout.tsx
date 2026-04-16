import { RootProvider } from 'fumadocs-ui/provider/next';
import { GeistSans } from 'geist/font/sans';
import { GeistMono } from 'geist/font/mono';
import type { Metadata, Viewport } from 'next';
import { appDescription, appName, siteUrl } from '@/lib/shared';
import './global.css';

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    default: `${appName} — Python Version Manager`,
    template: `%s · ${appName}`,
  },
  description: appDescription,
  applicationName: appName,
  keywords: [
    'python',
    'version manager',
    'virtual environment',
    'rust',
    'pvm',
    'venv',
    'pip',
    'anaconda alternative',
  ],
  openGraph: {
    type: 'website',
    url: siteUrl,
    siteName: appName,
    title: `${appName} — Python Version Manager`,
    description: appDescription,
  },
  twitter: {
    card: 'summary_large_image',
    title: `${appName} — Python Version Manager`,
    description: appDescription,
  },
};

export const viewport: Viewport = {
  themeColor: '#09090B',
  colorScheme: 'dark',
};

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <html
      lang="en"
      className={`dark ${GeistSans.variable} ${GeistMono.variable}`}
      suppressHydrationWarning
    >
      <body className="flex flex-col min-h-screen font-sans antialiased bg-fd-background text-fd-foreground">
        <RootProvider
          theme={{
            enabled: false,
            defaultTheme: 'dark',
            forcedTheme: 'dark',
          }}
        >
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
