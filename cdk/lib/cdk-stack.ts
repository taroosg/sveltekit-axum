import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as cognito from 'aws-cdk-lib/aws-cognito';
import { RemovalPolicy } from 'aws-cdk-lib';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as path from 'path';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as rds from 'aws-cdk-lib/aws-rds';
import { Duration } from 'aws-cdk-lib';

export class CdkStack extends cdk.Stack {
  public readonly userPool: cognito.UserPool;
  public readonly userPoolClient: cognito.UserPoolClient;
  public readonly myLambda: lambda.Function;
  public readonly functionUrl: lambda.FunctionUrl;

  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // --- 1. VPC 作成 (Lambda と RDS を同じネットワークに置く) ---
    const vpc = new ec2.Vpc(this, 'MyVpc', {
      maxAzs: 2, // 小規模例
    });

    // --- 2. RDS (PostgreSQL) インスタンス ---
    const dbInstance = new rds.DatabaseInstance(this, 'MyPostgresDB', {
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_14,
      }),
      vpc,
      vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS },
      instanceType: ec2.InstanceType.of(ec2.InstanceClass.T3, ec2.InstanceSize.MICRO),
      allocatedStorage: 20,
      // ユーザ名を "postgresuser" にし、パスワードはSecrets Managerで自動生成
      credentials: rds.Credentials.fromGeneratedSecret('postgresuser'),
      removalPolicy: RemovalPolicy.DESTROY, // 開発用
      deletionProtection: false,
      publiclyAccessible: false,
    });

    // --- 3. RDS Proxy ---
    const dbProxy = new rds.DatabaseProxy(this, 'MyDbProxy', {
      proxyTarget: rds.ProxyTarget.fromInstance(dbInstance),
      // 下記の secret に "username=postgresuser", "password=xxx" が格納される
      secrets: [dbInstance.secret!],
      vpc,
      iamAuth: false, // 通常のユーザ名・パスワード認証
      securityGroups: [
        new ec2.SecurityGroup(this, 'ProxySG', {
          vpc,
          description: 'SG for RDS Proxy',
          allowAllOutbound: true,
        }),
      ],
    });

    // --- Cognito ユーザープール ---
    this.userPool = new cognito.UserPool(this, 'SvelteAxum', {
      userPoolName: 'svelte-axum-userpool',
      selfSignUpEnabled: true,
      signInAliases: { email: true },
      removalPolicy: RemovalPolicy.DESTROY,
    });
    this.userPoolClient = this.userPool.addClient('SvelteAxumClient', {
      userPoolClientName: 'svelte-axum-appclient',
      generateSecret: false,
      authFlows: {
        userPassword: true,
        userSrp: true,
      },
    });

    // --- 4. Lambda Function (Rust Axum) ---
    this.myLambda = new lambda.Function(this, 'SvelteAxumBackendLambda', {
      functionName: 'svelte-axum-backend-lambda',
      runtime: lambda.Runtime.PROVIDED_AL2023,
      code: lambda.Code.fromAsset(
        path.join(__dirname, '../../backend/target/lambda/backend'),
      ),
      handler: 'bootstrap',
      memorySize: 512,
      timeout: Duration.seconds(10),
      // Lambda も同じ VPC 内に配置し、DB に接続できるように
      vpc,
      vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS },
      environment: {
        COGNITO_USER_POOL_ID: this.userPool.userPoolId,
        COGNITO_USER_POOL_CLIENT_ID: this.userPoolClient.userPoolClientId,
        COGNITO_REGION: this.region,
        DB_PROXY_ENDPOINT: dbProxy.endpoint,
        DB_SECRET_ARN: dbInstance.secret?.secretArn || '',
      },
    });

    // --- Lambda から RDS Proxy に接続を許可 ---
    // 第2引数: "postgresuser" → DB で定義されたユーザ名
    dbProxy.grantConnect(this.myLambda, 'postgresuser');

    // --- 5. Lambda Function URL ---
    this.functionUrl = this.myLambda.addFunctionUrl({
      authType: lambda.FunctionUrlAuthType.NONE,
      cors: {
        allowedOrigins: ['*'],
        allowedMethods: [lambda.HttpMethod.ALL],
      },
    });

    // --- 出力 ---
    new cdk.CfnOutput(this, 'FunctionUrl', {
      value: this.functionUrl.url,
    });
    new cdk.CfnOutput(this, 'DBProxyEndpoint', {
      value: dbProxy.endpoint,
    });
  }
}
