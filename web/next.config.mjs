import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();

const GITHUB_RAW_BASE =
  'https://raw.githubusercontent.com/taintlesscupcake/pvm/main/scripts';

/** @type {import('next').NextConfig} */
const config = {
  reactStrictMode: true,
  async rewrites() {
    return [
      { source: '/install.sh', destination: `${GITHUB_RAW_BASE}/install.sh` },
      { source: '/pvm.sh', destination: `${GITHUB_RAW_BASE}/pvm.sh` },
    ];
  },
};

export default withMDX(config);
