import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as fs from "node:fs";
import { local } from "@pulumi/command";

// util
const NAME_PREFIX: string = `${pulumi.getStack()}-${pulumi.getProject()}`;

export const apiLambdaRemoteId: string = `${NAME_PREFIX}-api-lambda-remote`;

const apiLambdaRemoteRoleId = `${apiLambdaRemoteId}-role`;
const apiLambdaRemoteRole = new aws.iam.Role(apiLambdaRemoteRoleId, {
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
  name: apiLambdaRemoteRoleId,
  tags: {
    Name: apiLambdaRemoteRoleId,
    Project: pulumi.getProject(),
    Stack: pulumi.getStack(),
    Environment: pulumi.getStack(),
    ManagedBy: "pulumi",
  },
});

const lambdaBasicExecutionPolicyAttachment = new aws.iam.RolePolicyAttachment(
  `${apiLambdaRemoteId}-basic-execution-policy-attachment`,
  {
    role: apiLambdaRemoteRole.name,
    policyArn: "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole",
  },
);

const lambdaXrayMonitoringPolicy = new aws.iam.Policy(
  `${apiLambdaRemoteId}-xray-monitoring-policy`,
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
  `${apiLambdaRemoteId}-xray-monitoring-policy-attachment`,
  {
    role: apiLambdaRemoteRole.name,
    policyArn: lambdaXrayMonitoringPolicy.arn,
  },
);

const API_DIR = "api";
const BIN_PATH = `./${API_DIR}/bin`;

// TODO: linux 環境依存のコマンドを修正したい
// TODO: feature の切り替えを環境変数や pulumi config で制御したい
const BUILD_COMMAND = `
cargo zigbuild --release --target aarch64-unknown-linux-musl --features lambda || exit 1
mkdir -p bin || exit 1
cp ./target/aarch64-unknown-linux-musl/release/api ./bin/bootstrap || exit 1
cp ./aws/collector-config.yaml ./bin/ || exit 1
`;

const apiBuildCommand = new local.Command(`${apiLambdaRemoteId}-build`, {
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
  }
});

export const apiLambdaRemote = new aws.lambda.Function(apiLambdaRemoteId, {
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
    logGroup: `/aws/lambda/${apiLambdaRemoteId}`,
    systemLogLevel: "WARN",
  },
  name: apiLambdaRemoteId,
  packageType: "Zip",
  role: apiLambdaRemoteRole.arn,
  runtime: aws.lambda.Runtime.CustomAL2023,
  timeout: 10,
  tags: {
    Name: apiLambdaRemoteId,
    Project: pulumi.getProject(),
    Stack: pulumi.getStack(),
    Environment: pulumi.getStack(),
    ManagedBy: "pulumi",
  },
});

export const apiLambdaRemoteUrl = new aws.lambda.FunctionUrl(`${apiLambdaRemoteId}-url`, {
  authorizationType: "NONE", // AWS_IAM
  functionName: apiLambdaRemote.name,
  invokeMode: "BUFFERED",
});

export const API_LAMBDA_REMOTE_FUNCTION_URL = apiLambdaRemoteUrl.functionUrl.apply((url: string) => {
  // NOTE: url の末尾の / を消す
  return url.replace(/\/$/, '');
});

export const API_LAMBDA_REMOTE_ROLE_ARN = pulumi.interpolate`${apiLambdaRemoteRole.arn}`;
