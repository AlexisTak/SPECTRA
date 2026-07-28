import type { NextConfig } from 'next'

const nextConfig: NextConfig = {
  output: 'export',
  trailingSlash: true,
  images: { unoptimized: true },
  env: {
    NEXT_PUBLIC_OLLAMA_URL:
      process.env.NEXT_PUBLIC_OLLAMA_URL ?? 'http://localhost:11434',
  },
}

export default nextConfig
