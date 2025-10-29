import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as fs from "node:fs";
import { local } from "@pulumi/command";

// util
const NAME_PREFIX: string = `${pulumi.getStack()}-${pulumi.getProject()}`;

export const apiLambdaId: string = `${NAME_PREFIX}-api-lambda`;

const apiLambdaRoleId = `${apiLambdaId}-role`;
const apiLambdaRole = new aws.iam.Role(apiLambdaRoleId, {
  assumeRolePolicy: JSON.stringify({
    Version: "2012-10-17",
    Statement: [
      {
        Action: "sts:AssumeRole",
        Effect: "Allow",
        Principal: {
          Service: "lambda.amazonaws.com",
        },
      },
    ],
  }),
  managedPolicyArns: [],
  name: apiLambdaRoleId,
  tags: {
    Name: apiLambdaRoleId,
    Project: pulumi.getProject(),
    Stack: pulumi.getStack(),
    Environment: pulumi.getStack(),
    ManagedBy: "pulumi",
  },
});

const lambdaBasicExecutionPolicyAttachment = new aws.iam.RolePolicyAttachment(
  `${apiLambdaId}-basic-execution-policy-attachment`,
  {
    role: apiLambdaRole.name,
    policyArn: "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole",
  },
);

const lambdaXrayMonitoringPolicy = new aws.iam.Policy(
  `${apiLambdaId}-xray-monitoring-policy`,
  {
    description: "Policy for Lambda to access X-Ray",
    policy: JSON.stringify({
        Version: "2012-10-17",
        Statement: [
          {
            Action: [
              "xray:PutTraceSegments",
              "xray:PutSpans",
              "xray:PutSpansForIndexing"
            ],
            Effect: "Allow",
            Resource: [
              "*"
            ],
          },
        ],
      }),
  },
);

const lambdaXrayMonitoringPolicyAttachment = new aws.iam.RolePolicyAttachment(
  `${apiLambdaId}-xray-monitoring-policy-attachment`,
  {
    role: apiLambdaRole.name,
    policyArn: lambdaXrayMonitoringPolicy.arn,
  },
);

const API_DIR = "api";
const BIN_PATH = `./${API_DIR}/bin`;

const BUILD_COMMAND = `
cargo zigbuild --release --target aarch64-unknown-linux-musl --features lambda || exit 1
mkdir -p bin || exit 1
cp ./target/aarch64-unknown-linux-musl/release/api ./bin/bootstrap || exit 1
cp ./aws/collector-config.yaml ./bin/ || exit 1
`;

const selfStack = new pulumi.StackReference(
	`organization/${pulumi.getProject()}/${pulumi.getStack()}`,
);

const apiBuildCommand = new local.Command(`${apiLambdaId}-build`, {
  create: BUILD_COMMAND,
  dir: `./${API_DIR}`,
  triggers: [
    new pulumi.asset.FileArchive(`./${API_DIR}/src`),
    new pulumi.asset.FileAsset(`./${API_DIR}/Cargo.toml`),
    new pulumi.asset.FileAsset(`./${API_DIR}/build.rs`),
    new pulumi.asset.FileAsset(`./${API_DIR}/aws/collector-config.yaml`),
  ],
  environment: {
    PULUMI_STACK: pulumi.getStack(),
    API_LAMBDA_ARN: selfStack.getOutput("API_LAMBDA_ARN"),
    PROJECT_NAME: pulumi.getProject(),
    REMOTE_ENDPOINT: selfStack.getOutput("API_LAMBDA_REMOTE_FUNCTION_URL"),
  }
});

export const apiLambda = new aws.lambda.Function(apiLambdaId, {
  architectures: ["arm64"],
  environment: {
    variables: {
      TZ: "Asia/Tokyo",
      OPENTELEMETRY_COLLECTOR_CONFIG_URI: "/var/task/collector-config.yaml",
      RUST_LOG: "info",
    },
  },
  code: fs.existsSync(BIN_PATH) ? apiBuildCommand.stdout.apply((_) => {
    return new pulumi.asset.FileArchive(BIN_PATH);
  }) : local.runOutput({
    command: BUILD_COMMAND,
    dir: `./${API_DIR}`,
    environment: {
      PULUMI_STACK: pulumi.getStack(),
      API_LAMBDA_ARN: selfStack.getOutput("API_LAMBDA_ARN"),
      PROJECT_NAME: pulumi.getProject(),
      REMOTE_ENDPOINT: selfStack.getOutput("API_LAMBDA_REMOTE_FUNCTION_URL"),
    }
  }).apply(_ => {
    return new pulumi.asset.FileArchive(BIN_PATH);
  }),
  ephemeralStorage: {
    size: 512,
  },
  memorySize: 256,
  handler: "bootstrap",
  layers: [
    "arn:aws:lambda:ap-northeast-1:184161586896:layer:opentelemetry-collector-arm64-0_18_0:1",
  ],
  loggingConfig: {
    applicationLogLevel: "INFO",
    logFormat: "JSON",
    logGroup: `/aws/lambda/${apiLambdaId}`,
    systemLogLevel: "WARN",
  },
  name: apiLambdaId,
  packageType: "Zip",
  role: apiLambdaRole.arn,
  runtime: aws.lambda.Runtime.CustomAL2023,
  timeout: 10,
  tags: {
    Name: apiLambdaId,
    Project: pulumi.getProject(),
    Stack: pulumi.getStack(),
    Environment: pulumi.getStack(),
    ManagedBy: "pulumi",
  },
});

export const apiLambdaUrl = new aws.lambda.FunctionUrl(`${apiLambdaId}-url`, {
  authorizationType: "NONE", // AWS_IAM
  functionName: apiLambda.name,
  invokeMode: "BUFFERED",
});

export const API_LAMBDA_FUNCTION_URL = apiLambdaUrl.functionUrl.apply((url: string) => {
  // NOTE: url の末尾の / を消す
  return url.replace(/\/$/, '');
});

export const API_LAMBDA_ROLE_ARN = pulumi.interpolate`${apiLambdaRole.arn}`;
export const API_LAMBDA_ARN = pulumi.interpolate`${apiLambda.arn}`;
