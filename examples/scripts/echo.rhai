fn onRequest(event) {
    let request = event.request();
    return request(request)
}

fn onResponse(event) {
    let response = event.response();
    let body = response.body_json();
    let newBody = [];
    for item in body {
        item["title"] = item["title"].to_upper();
        newBody.push(item);

    }
    response.set_body(newBody);
    return response(response)
}