import { CTA } from '@/components/landing/cta';
import { Comparison } from '@/components/landing/comparison';
import { Features } from '@/components/landing/features';
import { Footer } from '@/components/landing/footer';
import { Hero } from '@/components/landing/hero';
import { HowItWorks } from '@/components/landing/how-it-works';
import { Install } from '@/components/landing/install';

export default function HomePage() {
  return (
    <>
      <Hero />
      <Comparison />
      <Features />
      <HowItWorks />
      <Install />
      <CTA />
      <Footer />
    </>
  );
}
