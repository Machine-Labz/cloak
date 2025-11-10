import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docs: [
    {
      type: 'category',
      label: 'Overview',
      collapsed: false,
      items: [
        'overview/introduction',
        'overview/quickstart',
        'overview/system-architecture',
        'overview/visual-flow',
        'overview/tech-stack',
        'overview/status',
        'glossary',
      ],
    },
    {
      type: 'category',
      label: 'Workflows',
      items: [
        'workflows/deposit',
        'workflows/withdraw',
        'workflows/pow-withdraw',
      ],
    },
    {
      type: 'category',
      label: 'Zero-Knowledge Layer',
      link: {
        type: 'doc',
        id: 'zk/README',
      },
      items: [
        'zk/design',
        'zk/circuit-withdraw',
        'zk/encoding',
        'zk/merkle',
        'zk/prover-sp1',
        'zk/onchain-verifier',
        'zk/api-contracts',
        'zk/testing',
        'zk/threat-model',
      ],
    },
    {
      type: 'category',
      label: 'Wildcard Mining',
      items: [
        'pow/overview',
        'POW_QUICK_REFERENCE',
        'POW_INTEGRATION_GUIDE',
      ],
    },
    {
      type: 'category',
      label: 'On-Chain Programs',
      items: [
        'onchain/shield-pool',
        'onchain/scramble-registry',
      ],
    },
    {
      type: 'category',
      label: 'Services',
      items: [
        'offchain/indexer',
        'offchain/relay',
        'offchain/web-app',
      ],
    },
    {
      type: 'category',
      label: 'Packages & Tooling',
      items: [
        'packages/cloak-miner',
        'packages/zk-guest-sp1',
        'packages/vkey-generator',
        'packages/cloak-proof-extract',
        'packages/zk-verifier-program',
        'packages/tooling-test',
      ],
    },
    {
      type: 'category',
      label: 'Operations',
      items: [
        'operations/runbook',
        'operations/metrics-guide',
      ],
    },
    {
      type: 'category',
      label: 'APIs',
      items: [
        'api/indexer',
        'api/relay',
        'api/validator-agent',
      ],
    },
    {
      type: 'category',
      label: 'Project Status',
      items: [
        'roadmap',
        'overview/status',
        'COMPLETE_FLOW_STATUS',
        'CHANGELOG',
      ],
    },
  ],
};

export default sidebars;
