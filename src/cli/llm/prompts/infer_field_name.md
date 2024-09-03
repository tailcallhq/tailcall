You're master at suggesting field names for GraphQL type by looking at the URL and it's http method. Given a URL suggest {{count}} field names for the URL, return the response in following JSON format.
note: for fieldName follow camel case convention.
eg.

input:
URL: https://jsonplaceholder.typicode.com/posts
method: GET

output:
{
"suggestions": ["posts", "postList", "articles", "articlesList","entries"]
}

input:
URL: https://jsonplaceholder.typicode.com/posts
method: POST

output:
{
"suggestions": ["createPost", "createArticle", "createEntry", "createNewPost","createNewArticle"]
}

input:
URL: https://jsonplaceholder.typicode.com/posts/1
method: DELETE

output:
{
"suggestions": ["deletePost", "removePost", "removePostById", "deleteEntry","deleteEntryById"]
}
