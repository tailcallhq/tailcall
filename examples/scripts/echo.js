function onEvent(event) {
  // console.log(event)
  if (event.response) {
    return {response: event.response[0]}
  }

  return {request: [event.request]}
}

// function onEvent(event) {
//   console.log(event)
//   return {continue: event.request}
// }
