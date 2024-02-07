let i = 0
const asyncId = () => {
  i++
  return i
}
function onEvent(event) {
  if (event.message.request) {
    event.id = asyncId()
  }

  return event
}

// function onEvent(event) {
//   console.log(event)
//   return {continue: event.request}
// }
