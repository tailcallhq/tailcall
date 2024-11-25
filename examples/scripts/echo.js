function onRequest({request}) {
  return {request}
}

function hello(val) {
  let json = JSON.parse(val)
  return JSON.stringify(json.id)
}

function onResponse(response) {
  const parsedValue = JSON.parse(response)
  parsedValue.name += " - modified by JS"
  return JSON.stringify(parsedValue)
}
