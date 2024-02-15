import { serve } from "bun";

let i = 0
const asyncId = () => {
  i++
  return i
}

const CACHE = new Map();

serve({
  port: 3000,
  reusePort: true,
  async fetch(req) {
    // Only accept POST requests
    if (req.method !== "POST") {
      return new Response("Method Not Allowed", { status: 405 });
    }

    try {
      let event = await req.json();

      // console.log(event);
      if (event.message.request) {
        event.id = asyncId();

        const url = event.message.request.url;
        let json;

        if (CACHE.has(url)) {
          json = CACHE.get(url);
        } else {
          let request = await fetch(url);
          json = await request.json();
          CACHE.set(url, json);
        }

        event.message = {
          response: {
            status: 200,
            body: json.map(post => {
              return {
                ...post,
                title: post.title.toUpperCase()
              };
            })
          }
        };
      }

      // Return the modified object as JSON
      return new Response(JSON.stringify(event), {
        headers: { "Content-Type": "application/json" },
      });
    } catch(error) {

      console.error(error);
      // Handle potential errors, such as invalid JSON
      return new Response("Bad Request", { status: 400 });
    }
  }
});

console.log("Server running on http://localhost:3000");
