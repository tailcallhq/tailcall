const fetch = (url, req) =>
  new Promise((res, rej) => {
    __fetch__({...req, url}, (err, resp) => {
      if (err) return rej(err)
      res(resp)
    })
  })

function onEvent(request, cb) {
  main(request)
    .then((resp) => cb(null, resp))
    .catch((err) => cb(err))
}

async function main(request) {
  return await fetch("https://jsonplaceholder.typicode.com/posts", request)
}
