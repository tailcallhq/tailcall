terraform {
    required_providers {
        aws = {
            source  = "hashicorp/aws"
            version = "3.62.0"
        }
    }
}

provider "aws" {}

data "aws_iam_policy_document" "assume_role" {
    statement {
        effect = "Allow"

        principals {
            type        = "Service"
            identifiers = ["lambda.amazonaws.com"]
        }

        actions = ["sts:AssumeRole"]
    }
}

resource "aws_iam_role" "iam_for_lambda" {
    name               = "iam_for_lambda"
    assume_role_policy = data.aws_iam_policy_document.assume_role.json
}

resource "local_sensitive_file" "bootstrap" {
    content_base64 = filebase64("../../target/lambda/aws-lambda/bootstrap")
    filename       = "tmp/bootstrap"
}

resource "local_sensitive_file" "config" {
    content_base64 = filebase64("config.graphql")
    filename = "tmp/config.graphql"
}

data "archive_file" "tailcall" {

    depends_on = [
        local_sensitive_file.bootstrap,
        local_sensitive_file.config
    ]
    type        = "zip"
    source_dir  = "tmp/"
    output_path = "tailcall.zip"
}

resource "aws_lambda_function" "tailcall" {
    depends_on = [
        data.archive_file.tailcall
    ]

    role = aws_iam_role.iam_for_lambda.arn
    function_name    = "tailcall"
    runtime          = "provided.al2"
    architectures    = ["x86_64"]
    handler          = "bootstrap"
    filename         = data.archive_file.tailcall.output_path
    source_code_hash = data.archive_file.tailcall.output_base64sha256
}

resource "aws_api_gateway_rest_api" "tailcall" {
    name        = "tailcall"
}

resource "aws_api_gateway_resource" "proxy" {
    rest_api_id = "${aws_api_gateway_rest_api.tailcall.id}"
    parent_id   = "${aws_api_gateway_rest_api.tailcall.root_resource_id}"
    path_part   = "{proxy+}"
}

resource "aws_api_gateway_method" "proxy" {
    rest_api_id   = "${aws_api_gateway_rest_api.tailcall.id}"
    resource_id   = "${aws_api_gateway_resource.proxy.id}"
    http_method   = "ANY"
    authorization = "NONE"
}

resource "aws_api_gateway_integration" "lambda" {
    rest_api_id = "${aws_api_gateway_rest_api.tailcall.id}"
    resource_id = "${aws_api_gateway_method.proxy.resource_id}"
    http_method = "${aws_api_gateway_method.proxy.http_method}"

    integration_http_method = "POST"
    type                    = "AWS_PROXY"
    uri                     = "${aws_lambda_function.tailcall.invoke_arn}"
}

resource "aws_api_gateway_method" "proxy_root" {
    rest_api_id   = "${aws_api_gateway_rest_api.tailcall.id}"
    resource_id   = "${aws_api_gateway_rest_api.tailcall.root_resource_id}"
    http_method   = "ANY"
    authorization = "NONE"
}

resource "aws_api_gateway_integration" "lambda_root" {
    rest_api_id = "${aws_api_gateway_rest_api.tailcall.id}"
    resource_id = "${aws_api_gateway_method.proxy_root.resource_id}"
    http_method = "${aws_api_gateway_method.proxy_root.http_method}"

    integration_http_method = "POST"
    type                    = "AWS_PROXY"
    uri                     = "${aws_lambda_function.tailcall.invoke_arn}"
}

resource "aws_api_gateway_deployment" "tailcall" {
    depends_on = [
        aws_api_gateway_integration.lambda,
        aws_api_gateway_integration.lambda_root,
    ]

    rest_api_id = "${aws_api_gateway_rest_api.tailcall.id}"
}

resource "aws_lambda_permission" "apigw" {
    statement_id  = "AllowAPIGatewayInvoke"
    action        = "lambda:InvokeFunction"
    function_name = "${aws_lambda_function.tailcall.function_name}"
    principal     = "apigateway.amazonaws.com"

    # The /*/* portion grants access from any method on any resource
    # within the API Gateway "REST API".
    source_arn = "${aws_api_gateway_rest_api.tailcall.execution_arn}/*/*"
}

output "base_url" {
    value = "${aws_api_gateway_deployment.tailcall.invoke_url}"
}
