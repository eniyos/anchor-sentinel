import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Anchor Sentinel — Static Security Analysis for Solana Programs",
  description:
    "Detect critical Solana smart contract vulnerabilities before deployment. 14 security rules covering signer checks, PDA misconfigurations, reinit risks, and more.",
  keywords: [
    "solana security",
    "solana security tool",
    "anchor security",
    "solana smart contract audit",
    "sealevel attacks",
    "solana vulnerability scanner",
    "anchor sentinel",
    "solana static analysis",
  ],
  authors: [{ name: "eniyos" }],
  openGraph: {
    title: "Anchor Sentinel",
    description:
      "Detect Solana vulnerabilities before deployment. 14 rules, static analysis, CI-ready.",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Anchor Sentinel",
    description:
      "Detect Solana vulnerabilities before deployment. Fast, deterministic, CI-friendly.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${geistSans.variable} ${geistMono.variable} antialiased`}>
      <body>{children}</body>
    </html>
  );
}
