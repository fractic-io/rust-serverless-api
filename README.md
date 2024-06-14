Basic structure for setting up serverless Rust APIs for AWS API Gateway.

This set-up requires the use of SAM, which will build each rust binary into an AWS lambda, which can then be used to set up an API Gateway serverless API with SAM. This library handles parsing the API Gateway request, as well as basic authentication, and returning a JSON response to be parsed by front-end logic.

This code is provided as-is. For the time being, attention will not be given to backwards compatibility or clear documentation. It is open-sourced mainly for the chance that snippets may be useful to others looking to do similar tasks. Eventually, this may become a real library productionized and documented for external use.
