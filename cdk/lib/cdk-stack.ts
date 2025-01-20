import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as cognito from 'aws-cdk-lib/aws-cognito';
import { RemovalPolicy } from 'aws-cdk-lib';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as path from 'path';
export class CdkStack extends cdk.Stack {
  public readonly userPool: cognito.UserPool;
  public readonly userPoolClient: cognito.UserPoolClient;
  public readonly myLambda: lambda.Function;
  public readonly functionUrl: lambda.FunctionUrl;

  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // --- Cognito User Pool ---
    this.userPool = new cognito.UserPool(this, 'SvelteAxum', {
      userPoolName: 'svelte-axum-userpool',
      selfSignUpEnabled: true,
      signInAliases: {
        email: true,
      },
      removalPolicy: RemovalPolicy.DESTROY,
    });

    // --- アプリクライアント ---
    this.userPoolClient = this.userPool.addClient('SvelteAxumClient', {
      userPoolClientName: 'svelte-axum-appclient',
      generateSecret: false,
      authFlows: {
        userPassword: true,
        userSrp: true,
      },
    });

    // --- Lambda Function (Rust Axum) ---
    this.myLambda = new lambda.Function(this, 'SvelteAxumBackendLambda', {
      functionName: 'svelte-axum-backend-lambda',
      runtime: lambda.Runtime.PROVIDED_AL2023,
      code: lambda.Code.fromAsset(
        path.join(__dirname, '../../backend/target/lambda/backend')
      ),
      handler: 'bootstrap',
      memorySize: 512,
      timeout: cdk.Duration.seconds(10),
      environment: {
        USER_POOL_ID: this.userPool.userPoolId,
        USER_POOL_CLIENT_ID: this.userPoolClient.userPoolClientId,
      },
    });

        // --- Lambda Function URL ---
    this.functionUrl = this.myLambda.addFunctionUrl({
      authType: lambda.FunctionUrlAuthType.NONE,
      cors: {
        allowedOrigins: ['*'],
        allowedMethods: [
          lambda.HttpMethod.ALL,
        ],
      },
    });

    // Function URL を出力
    new cdk.CfnOutput(this, 'FunctionUrl', {
      value: this.functionUrl.url,
    });

  }
}
