const fs = require("fs")

const separator = "|"
let markdown = ""

function isTableRow(line) {
  return line.trim().startsWith("Latency") || line.trim().startsWith("Req/Sec")
}

function isTableHeader(line) {
  return line.trim().startsWith("Thread Stats")
}

function isTotalRequestLine(line) {
  return line.includes("requests in")
}

function getHeader() {
  return "|Thread Stats|Avg|Stdev|Max|+/- Stdev|\n|---|---|---|---|---|\n"
}

function getRow(line) {
  const words = line.trim().split(/[\s]+/)
  let row = separator
  words.forEach((word) => {
    const trimmedWord = word.trim()
    if (trimmedWord.length > 0) {
      row = row + trimmedWord + separator
    }
  })
  return row + "\n"
}

const lines = fs.readFileSync(process.argv[2], "utf8")
lines.split("\n").forEach((line) => {
  if (isTableHeader(line)) {
    markdown = markdown + getHeader()
  } else if (isTableRow(line)) {
    markdown = markdown + getRow(line)
  } else {
    if (isTotalRequestLine(line)) {
      markdown = markdown + "\n"
    }
    markdown = markdown + line.trim() + "\n\n"
  }
})

console.log(markdown)
