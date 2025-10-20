import type { Config } from '@docusaurus/types';

const config: Config = {
  title: 'Cloak Documentation',
  tagline: 'Privacy-preserving exit router for Solana',
  url: 'https://cloak-labz.github.io',
  baseUrl: '/',
  trailingSlash: false,
  favicon: 'img/favicon.svg',
  organizationName: 'cloak-labz',
  projectName: 'cloak',
  onBrokenLinks: 'warn',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  presets: [
    [
      'classic',
      {
        docs: {
          path: '../docs',
          routeBasePath: 'docs',
          sidebarPath: require.resolve('./sidebars.ts'),
          editUrl: 'https://github.com/cloak-labz/cloak/edit/main/docs',
          showLastUpdateAuthor: true,
          showLastUpdateTime: true,
        },
        blog: false,
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
  themeConfig: {
    navbar: {
      // title: 'Cloak Docs',
      logo: {
        alt: 'Cloak Logo',
        src: 'img/cloaklogo.svg',
      },
      items: [
        { to: '/docs/overview/introduction', label: 'Documentation', position: 'left' },
        { href: 'https://github.com/cloak-labz/cloak', label: 'GitHub', position: 'right' },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Project',
          items: [
            { label: 'Repository', href: 'https://github.com/cloak-labz/cloak' },
            { label: 'Roadmap', to: '/docs/roadmap' },
          ],
        },
        {
          title: 'Resources',
          items: [
            { label: 'Architecture', to: '/docs/overview/system-architecture' },
            { label: 'Zero-Knowledge', to: '/docs/zk/' },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Cloak Labz`,
    },
  },
};

export default config;
