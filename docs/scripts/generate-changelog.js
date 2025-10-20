#!/usr/bin/env node

/**
 * Generate changelog from Git commits
 * 
 * This script fetches recent git commits and formats them into a changelog.
 * Run this before building the docs to get the latest changes.
 * 
 * Usage: node scripts/generate-changelog.js
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const OUTPUT_FILE = path.join(__dirname, '../docs/CHANGELOG.md');
const REPO_URL = 'https://github.com/Machine-Labz/cloak';

// Get git log with formatted output
function getGitLog(count = 50) {
  try {
    // Check if git is available
    try {
      execSync('git --version', { stdio: 'ignore' });
    } catch {
      console.warn('⚠️  Git is not available. Skipping changelog generation.');
      return [];
    }

    // Check if we're in a git repository
    try {
      execSync('git rev-parse --git-dir', { 
        stdio: 'ignore', 
        cwd: path.join(__dirname, '../..') 
      });
    } catch {
      console.warn('⚠️  Not a git repository. Skipping changelog generation.');
      return [];
    }

    const log = execSync(
      `git log -${count} --pretty=format:"%H|%an|%ad|%s" --date=short`,
      { encoding: 'utf-8', cwd: path.join(__dirname, '../..') }
    );
    
    if (!log || log.trim() === '') {
      console.warn('⚠️  No git history found. This might be a shallow clone.');
      return [];
    }
    
    return log.split('\n').map(line => {
      const [hash, author, date, message] = line.split('|');
      return { hash, author, date, message };
    }).filter(commit => commit.hash && commit.message);
  } catch (error) {
    console.error('Error fetching git log:', error.message);
    console.warn('⚠️  Falling back to template changelog.');
    return [];
  }
}

// Group commits by month
function groupByMonth(commits) {
  const grouped = {};
  
  commits.forEach(commit => {
    const [year, month] = commit.date.split('-');
    const key = `${year}-${month}`;
    
    if (!grouped[key]) {
      grouped[key] = [];
    }
    grouped[key].push(commit);
  });
  
  return grouped;
}

// Format date for display
function formatMonth(yearMonth) {
  const [year, month] = yearMonth.split('-');
  const date = new Date(year, month - 1);
  return date.toLocaleDateString('en-US', { year: 'numeric', month: 'long' });
}

// Categorize commit by type
function categorizeCommit(message) {
  const lower = message.toLowerCase();
  
  if (lower.startsWith('feat:') || lower.includes('add') || lower.includes('implement')) {
    return 'features';
  }
  if (lower.startsWith('fix:') || lower.includes('fix')) {
    return 'fixes';
  }
  if (lower.startsWith('docs:') || lower.includes('doc')) {
    return 'documentation';
  }
  if (lower.startsWith('refactor:') || lower.includes('refactor')) {
    return 'refactoring';
  }
  if (lower.startsWith('chore:') || lower.includes('update')) {
    return 'maintenance';
  }
  
  return 'other';
}

// Generate markdown content
function generateMarkdown(commits) {
  const grouped = groupByMonth(commits);
  
  let markdown = `---
title: Changelog
description: Recent updates and changes to the Cloak project
---

# Changelog

This changelog is automatically generated from Git commit history.

View the complete history on [GitHub](${REPO_URL}/commits/master).

---

`;

  // Sort months in descending order
  const sortedMonths = Object.keys(grouped).sort().reverse();
  
  sortedMonths.forEach(yearMonth => {
    const monthCommits = grouped[yearMonth];
    markdown += `## ${formatMonth(yearMonth)}\n\n`;
    
    // Categorize commits
    const categorized = {
      features: [],
      fixes: [],
      documentation: [],
      refactoring: [],
      maintenance: [],
      other: []
    };
    
    monthCommits.forEach(commit => {
      const category = categorizeCommit(commit.message);
      // Clean up conventional commit prefixes
      const cleanMessage = commit.message
        .replace(/^(feat|fix|docs|refactor|chore|test|style):\s*/i, '')
        .trim();
      
      categorized[category].push({
        ...commit,
        cleanMessage
      });
    });
    
    // Output by category
    if (categorized.features.length > 0) {
      markdown += `### ✨ Features\n\n`;
      categorized.features.forEach(c => {
        markdown += `- ${c.cleanMessage} ([${c.hash.substring(0, 7)}](${REPO_URL}/commit/${c.hash}))\n`;
      });
      markdown += '\n';
    }
    
    if (categorized.fixes.length > 0) {
      markdown += `### 🐛 Bug Fixes\n\n`;
      categorized.fixes.forEach(c => {
        markdown += `- ${c.cleanMessage} ([${c.hash.substring(0, 7)}](${REPO_URL}/commit/${c.hash}))\n`;
      });
      markdown += '\n';
    }
    
    if (categorized.documentation.length > 0) {
      markdown += `### 📚 Documentation\n\n`;
      categorized.documentation.forEach(c => {
        markdown += `- ${c.cleanMessage} ([${c.hash.substring(0, 7)}](${REPO_URL}/commit/${c.hash}))\n`;
      });
      markdown += '\n';
    }
    
    if (categorized.refactoring.length > 0) {
      markdown += `### ♻️ Refactoring\n\n`;
      categorized.refactoring.forEach(c => {
        markdown += `- ${c.cleanMessage} ([${c.hash.substring(0, 7)}](${REPO_URL}/commit/${c.hash}))\n`;
      });
      markdown += '\n';
    }
    
    if (categorized.maintenance.length > 0 && categorized.maintenance.length <= 5) {
      markdown += `### 🔧 Maintenance\n\n`;
      categorized.maintenance.forEach(c => {
        markdown += `- ${c.cleanMessage} ([${c.hash.substring(0, 7)}](${REPO_URL}/commit/${c.hash}))\n`;
      });
      markdown += '\n';
    }
    
    markdown += '---\n\n';
  });
  
  markdown += `
## Contributing

To keep this changelog useful:

1. Write clear, descriptive commit messages
2. Use conventional commit format:
   - \`feat:\` for new features
   - \`fix:\` for bug fixes
   - \`docs:\` for documentation changes
   - \`refactor:\` for code refactoring
   - \`chore:\` for maintenance tasks

The changelog is automatically regenerated before each documentation build.
`;
  
  return markdown;
}

// Generate fallback changelog when git is not available
function generateFallbackChangelog() {
  return `---
title: Changelog
description: Recent updates and changes to the Cloak project
---

# Changelog

View the complete commit history on [GitHub](${REPO_URL}/commits/master).

## Latest Updates

This documentation site is automatically updated from the latest Git commits.

### Core Features

- **Zero-Knowledge Proofs**: SP1-powered Groth16 proofs verified on-chain
- **Privacy-Preserving Withdrawals**: Complete deposit and withdraw flow with ZK proofs
- **Wildcard Mining System**: Proof-of-work claims for prioritized transaction processing
- **Multi-Network Support**: Full support for localnet, testnet, and devnet

### Recent Improvements

- Documentation restructure and cleanup
- Updated homepage design
- Fixed broken links and references
- Improved navigation structure

---

For detailed commit history and updates, visit the [GitHub repository](${REPO_URL}/commits/master).

## Contributing

To contribute to this project:

1. Write clear, descriptive commit messages
2. Use conventional commit format when possible:
   - \`feat:\` for new features
   - \`fix:\` for bug fixes
   - \`docs:\` for documentation changes
   - \`refactor:\` for code refactoring
   - \`chore:\` for maintenance tasks

See [CONTRIBUTING.md](${REPO_URL}/blob/master/CONTRIBUTING.md) for more details.
`;
}

// Main execution
function main() {
  console.log('📝 Generating changelog from Git history...');
  
  const commits = getGitLog(100);
  
  let markdown;
  
  if (commits.length === 0) {
    console.warn('⚠️  No commits found. Using fallback changelog.');
    console.warn('💡 The changelog will be generated from git history on next deployment with full clone.');
    markdown = generateFallbackChangelog();
  } else {
    markdown = generateMarkdown(commits);
  }
  
  // Ensure docs directory exists
  const docsDir = path.dirname(OUTPUT_FILE);
  if (!fs.existsSync(docsDir)) {
    fs.mkdirSync(docsDir, { recursive: true });
  }
  
  fs.writeFileSync(OUTPUT_FILE, markdown, 'utf-8');
  
  if (commits.length === 0) {
    console.log(`✅ Fallback changelog generated at ${OUTPUT_FILE}`);
  } else {
    console.log(`✅ Changelog generated successfully at ${OUTPUT_FILE}`);
    console.log(`   Total commits processed: ${commits.length}`);
  }
}

// Run if called directly
if (require.main === module) {
  main();
}

module.exports = { getGitLog, generateMarkdown };

