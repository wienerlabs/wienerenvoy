import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Static export: the daemon serves the dashboard, there is no Next.js runtime.
  output: "export",
  // Emit each route as a folder with index.html (/system/index.html), so a
  // plain file server resolves deep links and refreshes correctly.
  trailingSlash: true,
  reactStrictMode: true,
  images: { unoptimized: true },
};

export default nextConfig;
