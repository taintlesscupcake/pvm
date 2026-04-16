import { ImageResponse } from 'next/og';

export const size = { width: 32, height: 32 };
export const contentType = 'image/png';

export default function Icon() {
  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'hsl(240 6% 5%)',
          color: 'white',
          fontFamily: 'sans-serif',
          fontWeight: 800,
          fontSize: 18,
          letterSpacing: '-0.04em',
          backgroundImage:
            'linear-gradient(135deg, hsl(258 90% 66%) 0%, hsl(189 94% 43%) 100%)',
          borderRadius: 7,
        }}
      >
        p
      </div>
    ),
    { ...size },
  );
}
