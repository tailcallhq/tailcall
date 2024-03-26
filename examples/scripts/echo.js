function onRequest({request}) {
  return {request}
}
function bar({request}) {
  console.log("its a bar")
  return {request}
}

function foo({request}) {
  console.log("its a foo")
  return {request}
}
