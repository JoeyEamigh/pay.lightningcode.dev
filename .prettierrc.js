/** @type {import("prettier").Config} */
export default {
  bracketSpacing: true,
  bracketSameLine: true,
  singleQuote: true,
  trailingComma: 'all',
  arrowParens: 'avoid',
  semi: true,
  plugins: ['prettier-plugin-organize-imports', 'prettier-plugin-astro', 'prettier-plugin-tailwindcss'],
  overrides: [
    {
      files: '*.astro',
      options: {
        parser: 'astro',
      },
    },
    {
      files: ['*.ts', '*.js', '*.tsx', '*.jsx', '*.cjs', '*.mjs', '*.astro'],
      options: {
        printWidth: 120,
      },
    },
    {
      files: ['*.html'],
      options: {
        printWidth: 100,
      },
    },
  ],
  tailwindConfig: './frontend/tailwind.config.js',
};
