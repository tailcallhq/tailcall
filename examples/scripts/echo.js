function onRequest(request) {
    return {
        response: {
            status: 200,
            headers: {
                "Content-Type": "text/plain"
            },
            body: JSON.stringify([{ title: request.url }])
        }
    };
}
