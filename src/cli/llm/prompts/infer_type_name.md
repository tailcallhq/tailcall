Given the sample schema of a GraphQL type, suggest {{count}} meaningful names for it.
The name should be concise and preferably a single word.
In this context, 'references' refer to the number of times the type is used in different fields as output type. For example, if the following type is referenced 12 times in the 'user' field and 13 times in the 'profile' field.
While suggesting the type name, do consider above references information for understanding the type context.

Example Input:
{{input}}

Example Output:
{{output}}

Ensure the output is in valid JSON format.

Do not add any additional text before or after the JSON.
