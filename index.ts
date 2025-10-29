import "./api/aws/lambda.ts";

export {
  API_LAMBDA_FUNCTION_URL,
  API_LAMBDA_ROLE_ARN,
  API_LAMBDA_ARN,
} from "./api/aws/lambda.ts";

import "./api/aws/lambda-remote.ts";

export {
  API_LAMBDA_REMOTE_FUNCTION_URL,
  API_LAMBDA_REMOTE_ROLE_ARN,
} from "./api/aws/lambda-remote.ts";

import "./monitoring/aws/transaction-search.ts";
