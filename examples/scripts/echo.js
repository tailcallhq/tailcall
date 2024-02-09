let i = 0
const asyncId = () => {
  i++
  return i
}

function str2ab(str) {
  var buf = new ArrayBuffer(str.length) // 2 bytes for each char
  var bufView = new Uint8Array(buf)
  for (var i = 0, strLen = str.length; i < strLen; i++) {
    bufView[i] = str.charCodeAt(i)
  }
  return buf
}

export async function onEvent(event) {
  await timerPromises.setTimeout(200)

  if (event.message.request) {
    event.id = asyncId()

    let [posts, users] = await Promise.all([
      fetch("https://jsonplaceholder.typicode.com/posts").then(resp => resp.json()),
      fetch("https://jsonplaceholder.typicode.com/users").then(resp => resp.json()),
    ]);

    posts = posts.map(post => {
      const user = users[post.userId - 1]
      return {...post, title: `${user.name}: ${post.title}`}
    })

    return {message: {response: {status: 200, body: str2ab(JSON.stringify(posts))}}}
  }

  if (event.message.response) {
    let body = event.message.response.body
    const text = String.fromCharCode.apply(null, new Uint8Array(body))
    let json = JSON.parse(text)
    json = json.map((post) => ({...post, title: post.title.toUpperCase()}))
    event.message.response.body = str2ab(JSON.stringify(json))
    return event
  }

  return event
}
