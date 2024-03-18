function onRequest({request}) {
  console.log(`${request.method} ${request.uri.path}`)

  return {request}
}