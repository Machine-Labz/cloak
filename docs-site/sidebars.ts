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
      items: [
        'zk/README',
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
      label: 'Proof-of-Work & Miner',
      items: [
        'pow/overview',
        'pow-architecture',
        'pow-scrambler-gate',
        'pow-implementation-status',
        'POW_ARCHITECTURE_FIXED',
        'POW_DOC_UPDATES_SUMMARY',
        'POW_QUICK_REFERENCE',
        'POW_WILDCARD_IMPLEMENTATION',
        'POW_REFACTOR_SUMMARY',
        'POW_INTEGRATION_GUIDE',
        'POW_INTEGRATION_COMPLETE',
      ],
    },
    {
      type: 'category',
      label: 'On-Chain Programs',
      items: [
        'onchain/shield-pool',
        'onchain/shield-pool-upstream',
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
      label: 'Roadmap & Status',
      items: [
        'roadmap',
        'CHANGELOG',
        'overview/status',
        'COMPLETE_FLOW_STATUS',
      ],
    },
  ],
};

export default sidebars;
