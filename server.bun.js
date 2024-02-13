import { serve } from "bun";

let i = 0
const asyncId = () => {
  i++
  return i
}

serve({
  port: 3000,
  reusePort: true,
  fetch(req) {
    // Only accept POST requests
    if (req.method !== "POST") {
      return new Response("Method Not Allowed", { status: 405 });
    }

    // Handle the POST request
    return req.json().then(event => {
      // console.log(event);
      if (event.message.request) {
        event.id = asyncId();
      }

      if (event.message.response) {
        let body = event.message.response.body;
        event.message.response.body = body.map(post => {
          return {
            ...post,
            title: post.title.toUpperCase()
          };
        });
      }

      // Return the modified object as JSON
      return new Response(JSON.stringify(event), {
        headers: { "Content-Type": "application/json" },
      });
    }).catch(error => {
      console.error(error);
      // Handle potential errors, such as invalid JSON
      return new Response("Bad Request", { status: 400 });
    });
  }
});

console.log("Server running on http://localhost:3000");
