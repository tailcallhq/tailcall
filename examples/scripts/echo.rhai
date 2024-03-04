fn onRequest(event) {
    let request = event.request();
    return request(request)
}