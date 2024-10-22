Basic structure for setting up serverless Rust APIs on AWS API Gateway.

This set-up requires the use of SAM, which will build each rust binary into an AWS lambda, which can then be used to set up an API Gateway serverless API with SAM. This library handles parsing the API Gateway request, as well as basic authentication, and returning a JSON response to be parsed by front-end logic.

> NOTE:
>
> This library currently hard-codes CORS headers (only marking requests from "https://fractic.io" as allowed). If access to the API is needed from a web application, these headers should be adjusted in src/response.rs to match the web app's domain.
>
> The domain set in the CORS headers does not mean the API will not respond to requests outside that domain, just that modern browsers will block the response from being read by the front-end code.

This code is provided as-is. For the time being, attention will not be given to backwards compatibility or clear documentation. It is open-sourced mainly for the chance that snippets may be useful to others looking to do similar tasks. Eventually, this may become a real library productionized and documented for external use.
