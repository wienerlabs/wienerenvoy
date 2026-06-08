import type { Metadata } from "next";
import { Funnel_Display } from "next/font/google";

import { AppShell } from "@/components/app-shell";
import { ThemeInit } from "@/components/theme-init";

import "./globals.css";

const funnel = Funnel_Display({
  subsets: ["latin"],
  variable: "--font-funnel",
  weight: ["300", "400", "500", "600", "700", "800"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "WienerEnvoy",
  description:
    "Tailnet-native control plane for your Mac mini home server: presence, power, and live telemetry.",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" className={funnel.variable} suppressHydrationWarning>
      <head>
        <ThemeInit />
      </head>
      <body>
        <AppShell>{children}</AppShell>
      </body>
    </html>
  );
}
