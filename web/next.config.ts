import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Static export: the daemon serves the dashboard, there is no Next.js runtime.
  output: "export",
  reactStrictMode: true,
  images: { unoptimized: true },
};

export default nextConfig;
