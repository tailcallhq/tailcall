#!/usr/bin/env node

/**
 * analyze.js
 *
 * Reimplementation of analyze.sh in JavaScript.
 * This script analyzes benchmark result files and generates summary statistics.
 *
 * Usage:
 *   node analyze.js <benchmark_file1> <benchmark_file2> ...
 *
 * The input files should be newline-delimited JSON benchmark results.
 */

import fs from 'fs';
import path from 'path';

function parseBenchmarkLine(line) {
  try {
    return JSON.parse(line);
  } catch (e) {
    return null;
  }
}

function formatValue(value, unit) {
  if (unit === 'ns' && value >= 1000) {
    return `${(value / 1000).toFixed(2)} μs`;
  }
  if (unit === 'μs' && value >= 1000) {
    return `${(value / 1000).toFixed(2)} ms`;
  }
  if (unit === 'ms' && value >= 1000) {
    return `${(value / 1000).toFixed(2)} s`;
  }
  return `${value.toFixed(2)} ${unit}`;
}

function calculatePercentageChange(oldValue, newValue) {
  if (oldValue === 0) return 0;
  return ((newValue - oldValue) / oldValue) * 100;
}

function generateComparisonTableMarkdown(oldBenchmarks, newBenchmarks) {
  let table = '';
  table += '| Benchmark | Base | Change | Percentage Change |\n';
  table += '|-----------|------|--------|-------------------|\n';

  for (let i = 0; i < oldBenchmarks.length; i++) {
    const old = oldBenchmarks[i];
    const newB = newBenchmarks[i];
    const oldEstimate = formatValue(old.typical.estimate, old.typical.unit);
    const newEstimate = formatValue(newB.typical.estimate, newB.typical.unit);
    const percentageChange = calculatePercentageChange(old.typical.estimate, newB.typical.estimate);

    table += `| ${old.id} | ${oldEstimate} | ${newEstimate} | ${percentageChange.toFixed(2)}% |\n`;
  }
  return table;
}

function performCheck(oldBenchmarks, newBenchmarks) {
  const threshold = 10.0;
  const exceeding = [];

  for (let i = 0; i < oldBenchmarks.length; i++) {
    const old = oldBenchmarks[i];
    const newB = newBenchmarks[i];
    const change = calculatePercentageChange(old.typical.estimate, newB.typical.estimate);
    if (change > threshold) {
      exceeding.push({ id: old.id, change });
    }
  }

  if (exceeding.length > 0) {
    console.error('Benchmarks exceeding the 10% change threshold:');
    for (const bench of exceeding) {
      console.error(`  \x1b[31m${bench.id}: ${bench.change.toFixed(2)}%\x1b[0m`);
    }
    process.exit(1);
  }
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length < 2 || args.length > 3) {
    console.error('Usage: analyze.js <old_file> <new_file> [check|table]');
    process.exit(1);
  }

  const oldFile = args[0];
  const newFile = args[1];
  const mode = args[2] || null;

  if (!fs.existsSync(oldFile)) {
    console.error(`Old file not found: ${oldFile}`);
    process.exit(1);
  }
  if (!fs.existsSync(newFile)) {
    console.error(`New file not found: ${newFile}`);
    process.exit(1);
  }

  const oldContent = fs.readFileSync(oldFile, 'utf-8');
  const newContent = fs.readFileSync(newFile, 'utf-8');

  const oldBenchmarks = oldContent
    .split('\n')
    .map(parseBenchmarkLine)
    .filter(Boolean);

  const newBenchmarks = newContent
    .split('\n')
    .map(parseBenchmarkLine)
    .filter(Boolean);

  if (oldBenchmarks.length !== newBenchmarks.length) {
    console.error('Mismatch in the number of benchmarks between old and new files');
    process.exit(1);
  }

  if (mode === 'check') {
    performCheck(oldBenchmarks, newBenchmarks);
  } else {
    const table = generateComparisonTableMarkdown(oldBenchmarks, newBenchmarks);
    const outputFile = path.join('benches', 'benchmark.md');
    fs.writeFileSync(outputFile, table);
    console.log(table);
    if (mode === null) {
      performCheck(oldBenchmarks, newBenchmarks);
    }
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
