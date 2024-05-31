const fs = require("fs")

const wrkOutput = fs.readFileSync(process.argv[2], "utf8")

const latencyAvgMatch = wrkOutput.match(/Latency\s+(\d+\.\d+)ms\s+(\d+\.\d+)ms\s+(\d+\.\d+)ms/)
const reqSecMatch = wrkOutput.match(/Req\/Sec\s+(\d+\.\d+k?)\s+(\d+\.\d+)\s+(\d+\.\d+k?)/)

if (!latencyAvgMatch || !reqSecMatch) {
  console.error("Error parsing " + process.argv[2])
  process.exit(1)
}

const convertToNumber = (value) => {
  if (value.endsWith("k")) {
    return parseFloat(value.replace("k", "")) * 1000
  }
  return parseFloat(value)
}

const latency = {
  value: parseFloat(latencyAvgMatch[1]),
  lower_value: parseFloat(latencyAvgMatch[2]),
  upper_value: parseFloat(latencyAvgMatch[3]),
}

const reqSec = {
  value: convertToNumber(reqSecMatch[1]),
  lower_value: parseFloat(reqSecMatch[2]),
  upper_value: convertToNumber(reqSecMatch[3]),
}

const resultJson = {
  benchmark_name: {
    latency: latency,
    req_sec: reqSec,
  },
}

console.log(JSON.stringify(resultJson, null, 2))
