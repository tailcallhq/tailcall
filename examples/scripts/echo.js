function onRequest({request}) {
  console.log(`${request.method} ${request.uri.path}`)

  return {request}
}
function postsRequestHandler({request}) {
  console.log(`postsRequestHandler : ${request.method} ${request.uri.path}`)

  return {request}
}
