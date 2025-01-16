import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as cognito from 'aws-cdk-lib/aws-cognito';
import { RemovalPolicy } from 'aws-cdk-lib';

export class CdkStack extends cdk.Stack {
  public readonly userPool: cognito.UserPool;
  public readonly userPoolClient: cognito.UserPoolClient;

  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // --- Cognito User Pool ---
    this.userPool = new cognito.UserPool(this, 'SvelteAxum', {
      userPoolName: 'svelte-axum-userpool',
      selfSignUpEnabled: true,
      signInAliases: {
        email: true
      },
      removalPolicy: RemovalPolicy.DESTROY,
    });

    // --- アプリクライアント ---
    this.userPoolClient = this.userPool.addClient('SvelteAxumClient', {
      userPoolClientName: 'svelte-axum-appclient',
      generateSecret: false,
      authFlows: {
        userPassword: true,
        userSrp: true
      },
      // ↑ ID/PW or SRP でログイン
      // refreshTokenValidity: cdk.Duration.days(30),
    });
  }
}
