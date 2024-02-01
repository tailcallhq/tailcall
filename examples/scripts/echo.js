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
const TEMPLATE = "http://upstream";
const UPSTREAM = "http://jsonplaceholder.typicode.com";

async function main(request) {
  // console.log("Start...", request.url);
  const url = request.url.replace(TEMPLATE, UPSTREAM);

  if (url.endsWith("/posts")) {
    const resp = await fetch(url, request);
    const body = await resp.json();

    // console.log("End...");

    return {...resp, body: body.map(e => ({...e, title: e.title.toUpperCase()}))};
  }

  const resp = await fetch(url, request);
  const body = await resp.json();

  return {...resp, body};
}
