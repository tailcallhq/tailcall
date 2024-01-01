# tailcall

[![Continuous Integration](https://github.com/tailcallhq/tailcall/actions/workflows/ci.yml/badge.svg)](https://github.com/tailcallhq/tailcall/actions/workflows/ci.yml)

API Orchestration for Scale

Tailcall is solving the problem of API composition in microservices architecture. In a microservices architecture, different services are developed and deployed separately. However, when you are building a rich client application, it needs to access multiple services. The backend for frontend (BFF) layer becomes necessary to provide an abstraction layer that can aggregate data from multiple services. This layer is critical for providing a good user experience, but building it manually can be expensive, time-consuming, and error-prone.

Tailcall provides a solution for this problem by providing a programmable API gateway that allows developers to compose APIs in a modular way. This allows developers to focus on building individual services without worrying about how they will be aggregated for client applications. Tailcall's solution also includes performance optimizations, first class support for canary releases, versioning, resiliency, and other features that make it easier for developers to manage their APIs.

By providing a solution for API composition, Tailcall is addressing a critical pain point for organizations that use microservices architecture. The lack of a generic solution for API composition makes it difficult for organizations to scale and provide a good user experience. We believe our solution can help organizations reduce costs, improve performance, and provide a better user experience.


## Getting Started

To use tailcall, you'll need to have Scala, SBT, and Java 11 or above installed on your machine.

### Installation

1. Clone the repository: `git clone https://github.com/tailcallhq/tailcall.git`
2. Navigate to the project directory: `cd tailcall`
3. Open the SBT console: `sbt`
4. To start the server, run the following command inside the SBT console: `~ tc-server`
5. Open a browser and go to `http://localhost:8080/`
6. To communicate with the server use the `tc` command from the SBT console.
   

### Contributing

Contributions are welcome! Please fork the repository and submit a pull request.

### License

This project is licensed under the MIT License. 
