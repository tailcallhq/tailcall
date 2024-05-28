function onRequest({request}) {
  return {request}
}

function hello(val) {
  let json = JSON.parse(val)
  return JSON.stringify(json.id)
}
