import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
// import * as native from "@pulumi/aws-native";

// util
const NAME_PREFIX: string = `${pulumi.getStack()}-${pulumi.getProject()}`;
const AWS_ACCOUNT_ID = pulumi.output(aws.getCallerIdentity().then(result => result.accountId));
const AWS_REGION = pulumi.output(aws.getRegion().then(result => result.region));

const transactionSearchAccessPolicy = new aws.cloudwatch.LogResourcePolicy(
  `${NAME_PREFIX}-transaction-search-access-policy`,
  {
    policyName: `${NAME_PREFIX}-transaction-search-access-policy`,
    policyDocument: pulumi.all([AWS_ACCOUNT_ID, AWS_REGION]).apply(([accountId, region]) =>
      JSON.stringify({
        Version: "2012-10-17",
        Statement: [
          {
            Action: [
              "logs:PutLogEvents"
            ],
            Principal: {
              Service: "xray.amazonaws.com"
            },
            Effect: "Allow",
            Resource: [
              `arn:aws:logs:${region}:${accountId}:log-group:aws/spans:*`,
              `arn:aws:logs:${region}:${accountId}:log-group:/aws/application-signals/data:*`
            ],
            Condition: {
              ArnLike: {
                "aws:SourceArn": `arn:aws:xray:${region}:${accountId}:*`
              },
              StringEquals: {
                "aws:SourceAccount": accountId
              }
            }
          }
        ]
      })
    )
  }
);

/*
const transactionSearchConfig = new native.xray.TransactionSearchConfig(
  `${NAME_PREFIX}-transaction-search-config`,
  {
    indexingPercentage: 100
  },
  {
    dependsOn: [transactionSearchAccessPolicy],
  }
);
*/
