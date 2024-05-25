import json
import re

def parse_wrk_output(file_path):
    data = {}
    with open(file_path, 'r') as file:
        content = file.readlines()
        for line in content:
            if "Latency" in line:
                # Example: Latency 88.0ms, 87.42ms, 88.88ms
                match = re.search(r"Latency (\d+\.\d+)ms, (\d+\.\d+)ms, (\d+\.\d+)ms", line)
                if match:
                    data['latency'] = float(match.group(1))
                    data['latency_lower'] = float(match.group(2))
                    data['latency_upper'] = float(match.group(3))
    return data

def generate_bmf_json(data):
    bmf_json = {
        "my_benchmark": {
            "latency": {
                "value": data["latency"],
                "lower_value": data["latency_lower"],
                "upper_value": data["latency_upper"]
            }
        }
    }
    return bmf_json
# Example usage
wrk_data = parse_wrk_output('wrk_output.txt')
bmf_data = generate_bmf_json(wrk_data)
with open('results.json', 'w') as f:
    json.dump(bmf_data, f, indent=4)
