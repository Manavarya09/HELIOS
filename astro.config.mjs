// @ts-check
import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  site: 'https://manavarya09.github.io',
  base: '/HELIOS',
  vite: {
    plugins: [tailwindcss()]
  }
});