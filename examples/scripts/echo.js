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
  const response = await fetch("https://jsonplaceholder.typicode.com/posts", request)
  const date = new Date();
  console.log({date})
  const body = await response.json()
  const newBody = body.map((post) => ({...post, title: "Hello Shashi!!!"}))
  return {...response, body: newBody}
}
