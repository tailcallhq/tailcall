Given the Operation Definition of GraphQL, suggest {{count}} meaningful names for the argument names.
The name should be concise and preferably a single word.

Example Input:
{
"argument": {
"name": "Input1",
"outputType: "Article"
},
"field": {
"name" : "createPost",
"outputType" : "Post"
}
}

Example Output:
suggestions: ["createPostInput","postInput", "articleInput","noteInput","messageInput"],

Ensure the output is in valid JSON format.
