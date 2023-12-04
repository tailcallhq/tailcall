These folders contain examples of subgraphs using the following federation directives. Each folder contains the individual Apollo subgraph configuration, and the `rover` composed schema. For each directive, the corresponding http_spec with the corresponding tailcall config is listed if it exists

`shareable`  
Example of a field that can be resolved by multiple subgraphs.  
The `name` field in `Location` is resolvable by both the `Locations` and `Reviews` graph.  
**http spec:** tests/http/federation-router-shareable.yml

`override`  
Example of a field that is now resolved by a particular subgraph instead of another subgraph where it is also defined.  
The `description` field in type `Location` is overridden to be resolved from the `locations` subgraph.
**http spec:** tests/http/federation-router-override-field.yml

`inaccessible`  
Example of a field marked `inaccessible`, which means that it does not appear in the output schema.  
The `description` field in type `Location` in the `locations` subgraph is marked `inaccessible`.
**http spec:** **TODO**

`external` with `provides`  
Example of a field that usually cannot resolve a particular field, but it can at a particular query path.  
The `inventory` subgraph cannot resolve the `name` field in type `Product`, except at the `outOfStockProducts` query path.
**http spec:** **TODO**

`external` with `requires`  
Example of a field that depends on the values of other entity fields that are resolved by **other** subgraphs.  
The `delivery` field in type `Product` in the `inventory` subgraph depends on the `dimensions` field that is resolved by the `products` subgraph.
\*http spec:\*\* **TODO**
