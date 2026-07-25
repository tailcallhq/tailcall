#!/usr/bin/env node

/**
 * analyze.js
 *
 * A JavaScript reimplementation of the analyze.sh script from the graphql-benchmarks project.
 * This script analyzes benchmark results and generates summary statistics and plots.
 *
 * Usage:
 *   node analyze.js [options]
 *
 * Options:
 *   --input <file>    Input benchmark results file (default: benchmark_results.txt)
 *   --output <dir>    Output directory for generated files (default: ./output)
 *
 * This script requires Node.js and the 'canvas' package for plotting.
 */

import fs from 'fs';
import path from 'path';
import { createCanvas } from 'canvas';

const DEFAULT_INPUT = 'benchmark_results.txt';
const DEFAULT_OUTPUT_DIR = './output';

function parseBenchmarkLine(line) {
  try {
    return JSON.parse(line);
  } catch {
    return null;
  }
}

function loadBenchmarks(filePath) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const lines = content.split('\n').filter(Boolean);
  return lines.map(parseBenchmarkLine).filter(Boolean);
}

function summarizeBenchmarks(benchmarks) {
  const summary = {};
  for (const bench of benchmarks) {
    if (!summary[bench.id]) {
      summary[bench.id] = [];
    }
    summary[bench.id].push(bench.typical.estimate);
  }
  return summary;
}

function mean(arr) {
  if (arr.length === 0) return 0;
  return arr.reduce((a, b) => a + b, 0) / arr.length;
}

function stddev(arr) {
  if (arr.length === 0) return 0;
  const m = mean(arr);
  const variance = arr.reduce((a, b) => a + (b - m) ** 2, 0) / arr.length;
  return Math.sqrt(variance);
}

function generateMarkdownSummary(summary) {
  let md = '# Benchmark Summary\n\n';
  md += '| Benchmark | Mean (ns) | Std Dev (ns) |\n';
  md += '|-----------|-----------|--------------|\n';
  for (const [id, values] of Object.entries(summary)) {
    md += `| ${id} | ${mean(values).toFixed(2)} | ${stddev(values).toFixed(2)} |\n`;
  }
  return md;
}

function saveMarkdown(outputDir, content) {
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }
  const filePath = path.join(outputDir, 'summary.md');
  fs.writeFileSync(filePath, content, 'utf-8');
  console.log(`Summary saved to ${filePath}`);
}

function plotHistogram(values, title, outputPath) {
  const width = 800;
  const height = 400;
  const canvas = createCanvas(width, height);
  const ctx = canvas.getContext('2d');

  // Clear background
  ctx.fillStyle = '#fff';
  ctx.fillRect(0, 0, width, height);

  // Title
  ctx.fillStyle = '#000';
  ctx.font = '20px Arial';
  ctx.fillText(title, 10, 30);

  // Compute histogram bins
  const bins = 30;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const binWidth = (max - min) / bins;
  const counts = new Array(bins).fill(0);
  for (const v of values) {
    const bin = Math.min(Math.floor((v - min) / binWidth), bins - 1);
    counts[bin]++;
  }
  const maxCount = Math.max(...counts);

  // Draw histogram bars
  const margin = 50;
  const barWidth = (width - 2 * margin) / bins;
  ctx.fillStyle = '#007bff';
  for (let i = 0; i < bins; i++) {
    const barHeight = (counts[i] / maxCount) * (height - 2 * margin);
    ctx.fillRect(
      margin + i * barWidth,
      height - margin - barHeight,
      barWidth - 2,
      barHeight
    );
  }

  // Axes
  ctx.strokeStyle = '#000';
  ctx.beginPath();
  ctx.moveTo(margin, margin);
  ctx.lineTo(margin, height - margin);
  ctx.lineTo(width - margin, height - margin);
  ctx.stroke();

  // Save to file
  const buffer = canvas.toBuffer('image/png');
  fs.writeFileSync(outputPath, buffer);
  console.log(`Histogram saved to ${outputPath}`);
}

function main() {
  const args = process.argv.slice(2);
  let inputFile = DEFAULT_INPUT;
  let outputDir = DEFAULT_OUTPUT_DIR;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--input' && i + 1 < args.length) {
      inputFile = args[i + 1];
      i++;
    } else if (args[i] === '--output' && i + 1 < args.length) {
      outputDir = args[i + 1];
      i++;
    }
  }

  if (!fs.existsSync(inputFile)) {
    console.error(`Input file not found: ${inputFile}`);
    process.exit(1);
  }

  const benchmarks = loadBenchmarks(inputFile);
  if (benchmarks.length === 0) {
    console.error('No valid benchmark data found.');
    process.exit(1);
  }

  const summary = summarizeBenchmarks(benchmarks);
  const md = generateMarkdownSummary(summary);
  saveMarkdown(outputDir, md);

  // Plot histograms for each benchmark
  for (const [id, values] of Object.entries(summary)) {
    const safeId = id.replace(/[^\w]/g, '_');
    const outputPath = path.join(outputDir, `${safeId}_histogram.png`);
    plotHistogram(values, `Histogram for ${id}`, outputPath);
  }
}

if (require.main === module) {
  main();
}
