const fetch = (url, req) =>
  new Promise((res, rej) => {
    __fetch__({...req, url}, (err, resp) => {
      if (err) return rej(err)
      res({...res, json: async () => resp.body})
    })
  })

function onEvent(request, cb) {
  main(request)
    .then((resp) => cb(null, resp))
    .catch((err) => cb(err))
}

////
//// USER LAND CODE
////
async function main(request) {
  const resp = await fetch(request.url, request)
  const body = await resp.json()
  return {...resp, body: body}
}
