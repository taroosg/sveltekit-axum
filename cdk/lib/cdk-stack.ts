import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as cognito from 'aws-cdk-lib/aws-cognito';
import { RemovalPolicy } from 'aws-cdk-lib';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as path from 'path';
import * as apigw from 'aws-cdk-lib/aws-apigatewayv2';
import * as integrations from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import {
  HttpJwtAuthorizer,
} from 'aws-cdk-lib/aws-apigatewayv2-authorizers';
export class CdkStack extends cdk.Stack {
  public readonly userPool: cognito.UserPool;
  public readonly userPoolClient: cognito.UserPoolClient;
  public readonly myLambda: lambda.Function;
  public readonly httpApi: apigw.HttpApi;

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

    // --- APIGateway HTTP API ---
    this.httpApi = new apigw.HttpApi(this, 'MyHttpApi', {
      apiName: 'myproject-httpapi',
      createDefaultStage: true,
    });

    // Lambda インテグレーション
    const lambdaIntegration = new integrations.HttpLambdaIntegration(
      'MyLambdaIntegration',
      this.myLambda
    );

    // Cognito JWT オーソライザー
    const issuer = `https://cognito-idp.${this.region}.amazonaws.com/${this.userPool.userPoolId}`;
    const audience = [this.userPoolClient.userPoolClientId];

    const jwtAuthorizer = new HttpJwtAuthorizer(
      'MyCognitoAuthorizer',
      issuer,
      {
        jwtAudience: audience,
        authorizerName: 'MyCognitoAuthorizer',
      }
    );

    // -- ルートにオーソライザーを設定して保護 --
    this.httpApi.addRoutes({
      path: '/protected',
      methods: [apigw.HttpMethod.ANY],
      integration: lambdaIntegration,
      authorizer: jwtAuthorizer,
    });

    this.httpApi.addRoutes({
      path: '/fuga',
      methods: [apigw.HttpMethod.ANY],
      integration: lambdaIntegration,
      authorizer: jwtAuthorizer,
    });

    // -- パブリックなルート "/" はオーソライザーなし --
    this.httpApi.addRoutes({
      path: '/',
      methods: [apigw.HttpMethod.ANY],
      integration: lambdaIntegration,
    });

    this.httpApi.addRoutes({
      path: '/hoge',
      methods: [apigw.HttpMethod.ANY],
      integration: lambdaIntegration,
    });

  }
}
