import { ImageResponse } from 'next/og';
import { appDescription, appName } from '@/lib/shared';

export const alt = `${appName} — Python Version Manager`;
export const size = { width: 1200, height: 630 };
export const contentType = 'image/png';

export default function OG() {
  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          padding: '72px 80px',
          background: 'hsl(240 6% 5%)',
          color: 'white',
          fontFamily: 'sans-serif',
          backgroundImage: `
            radial-gradient(80% 60% at 50% 0%, hsla(258, 90%, 66%, 0.30), transparent 60%),
            radial-gradient(60% 40% at 80% 30%, hsla(189, 94%, 43%, 0.18), transparent 60%),
            linear-gradient(180deg, hsl(240 6% 8%) 0%, hsl(240 6% 5%) 100%)
          `,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
          <div
            style={{
              width: 28,
              height: 28,
              borderRadius: 7,
              backgroundImage:
                'linear-gradient(135deg, hsl(258 90% 66%) 0%, hsl(189 94% 43%) 100%)',
            }}
          />
          <span
            style={{
              fontSize: 28,
              fontWeight: 600,
              letterSpacing: '-0.02em',
            }}
          >
            {appName}
          </span>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              fontSize: 88,
              fontWeight: 600,
              letterSpacing: '-0.04em',
              lineHeight: 1.05,
            }}
          >
            <span style={{ color: 'white' }}>One Python.</span>
            <span
              style={{
                backgroundImage:
                  'linear-gradient(135deg, hsl(258 90% 70%) 0%, hsl(189 94% 60%) 100%)',
                backgroundClip: 'text',
                color: 'transparent',
              }}
            >
              Every environment.
            </span>
          </div>
          <p
            style={{
              margin: 0,
              fontSize: 28,
              color: 'hsl(240 4% 70%)',
              maxWidth: 900,
              lineHeight: 1.4,
            }}
          >
            {appDescription}
          </p>
        </div>

        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            color: 'hsl(240 4% 60%)',
            fontSize: 22,
          }}
        >
          <span style={{ fontFamily: 'monospace' }}>
            curl -fsSL .../install.sh | bash
          </span>
          <span>pvm.sungjin.dev</span>
        </div>
      </div>
    ),
    { ...size },
  );
}
