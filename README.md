# XGS platform monorepo

This [Turborepo](https://turbo.build/repo) contains the source code for the XGS platform.

## Requirements

- [Node.js](https://nodejs.org/)
- [pnpm](https://pnpm.io/) package manager
- [Expo's requirements](https://docs.expo.dev/get-started/installation/#requirements) and [local development prerequisites](https://docs.expo.dev/guides/local-app-development/#prerequisites)
- [Rust](https://www.rust-lang.org/) with the `wasm32-unknown-unknown` target:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- [dfx](https://internetcomputer.org/docs/current/developer-docs/getting-started/install/) (better if installed with the dfx version manager - [`dfxvm`](https://github.com/dfinity/dfxvm))
- an [Auth0](https://auth0.com) account
- an Android/iOS device or simulator

## Setup

### Configure Auth0

Follow these steps to configure Auth0 for this project:

1. [Create a Tenant](https://auth0.com/docs/get-started/auth0-overview/create-tenants) and get your Auth0 Tenant domain, which looks like `<TENANT_NAME>.<TENANT_REGION>.auth0.com`

#### Create a Native Application

The Auth0 Native Application is used to authenticated users from the [Mobile app](#mobile-app). Follow these steps:

1. [Create a Native Application](https://auth0.com/docs/get-started/auth0-overview/create-applications/native-apps)
2. In the _Dashboard > Applications > YOUR_APP > Settings_ tab, set the **Allowed Callback URLs** and **Allowed Logout URLs** to:

   - `io.icp0.jwtauthdemo.auth0://<YOUR_AUTH0_TENANT_DOMAIN>/ios/io.icp0.jwtauthdemo/callback`
   - `io.icp0.jwtauthdemo.auth0://<YOUR_AUTH0_TENANT_DOMAIN>/android/io.icp0.jwtauthdemo/callback`

   Where `<YOUR_AUTH0_TENANT_DOMAIN>` is the Auth0 Tenant domain and `io.icp0.jwtauthdemo` is both the **Android Package Name** and **iOS Bundle Identifier**, as configured in the [app.config.js](./src/app/app.config.js) file.

3. In the _Dashboard > Applications > YOUR_APP > Credentials_ tab, set the **Authentication Method** to **None** (instead of **Client Secret (Post)**)

The 1st step of the Auth0 React Native [Quickstart interactive guide](https://auth0.com/docs/quickstart/native/react-native-expo/interactive) can be helpful too.

#### Create a Web Application

The Auth0 Web Application is used to authenticated users from the [Web app](#web-app). Follow these steps:

1. [Create a Web Application](https://auth0.com/docs/get-started/auth0-overview/create-applications/web-apps)
2. In the _Dashboard > Applications > YOUR_APP > Settings_ tab:

   - set the **Allowed Callback URLs** to `http://localhost:3000/api/auth/callback`
   - set the **Allowed Logout URLs** to `http://localhost:3000`

   When you'll deploy the web app on a public domain, you will have to adjust these URLs accordingly.

3. Configure the **Login Flow** in order to add custom JWT claims to the token minted by Auth0 and sync the user with Hasura. See [this guide](https://hasura.io/learn/graphql/hasura-authentication/integrations/auth0/#customjwtclaims) for more details.
   Write the following **Login / Post Login Custom Action** (select the _Node.js 18_ runtime):

   **Hasura Sync User and JWT Claims**

   ```javascript
   const fetch = require("node-fetch");

   const HASURA_GRAPHQL_URL =
     "https://alive-glowworm-caring.ngrok-free.app//v1/graphql";

   /**
    * Handler that will be called during the execution of a PostLogin flow.
    *
    * @param {Event} event - Details about the user and the context in which they are logging in.
    * @param {PostLoginAPI} api - Interface whose methods can be used to change the behavior of the login.
    */
   exports.onExecutePostLogin = async (event, api) => {
     const auth0Id = event.user.user_id;
     const email = event.user.email ?? null;
     const lastLoginAt = event.session?.authenticated_at ?? new Date();

     const admin_secret = event.secrets.HASURA_GRAPHQL_ADMIN_SECRET;

     const query = `mutation UpsertUser($auth0Id: String!, $email: citext!, $lastLoginAt: timestamptz!, $isPending: Boolean!) {
          insert_users_one(
            object: {auth0_id: $auth0Id, email: $email, last_login_at: $lastLoginAt, is_pending: $isPending},
            on_conflict: {constraint: users_email_key, update_columns: [auth0_id, is_pending, last_login_at]}
          ) {
            id
          }
        }
      `;

     const variables = { auth0Id, email, lastLoginAt, isPending: false };

     const res = await fetch(HASURA_GRAPHQL_URL, {
       method: "POST",
       body: JSON.stringify({
         query: query,
         variables: variables,
       }),
       headers: {
         "content-type": "application/json",
         "x-hasura-admin-secret": admin_secret,
       },
     });

     const graphqlRes = await res.json();

     console.log("GraphQL Response:", graphqlRes);

     if (graphqlRes.errors && graphqlRes.errors.length > 0) {
       const err = graphqlRes.errors[0];
       api.access.deny(err?.message || "GraphQL API returned an error.");
       return;
     }

     if (!graphqlRes.data) {
       api.access.deny("No data from GraphQL API.");
       return;
     }

     if (event.authorization) {
       const userId = graphqlRes.data.insert_users_one.id;
       const roles = ["user"];
       const customClaimKey = "https://hasura.io/jwt/claims";
       const customClaimValue = {
         "x-hasura-default-role": "user",
         "x-hasura-allowed-roles": roles,
         "x-hasura-user-id": userId,
       };

       api.idToken.setCustomClaim(customClaimKey, customClaimValue);
       api.accessToken.setCustomClaim(customClaimKey, customClaimValue);
     }
   };
   ```

   Finally, add this action to the Login Flow in the Auth0 Dashboard.

   In order for the Auth0 Login Action to access the local Hasura API, you need to run a tunnel like ngrok. See the [backend's README tunnel](./apps/backend/README.md#tunnel) for more details.

The Next.js Quickstart [guide](https://auth0.com/docs/quickstart/webapp/nextjs) can be helpful too.

#### Test users

Create the following test users in the Auth0 Dashboard:

| Username                    | Password       |
| --------------------------- | -------------- |
| `testmanagerlugano@test.io` | `Thisisatest!` |
| `testparent@test.io`        | `Thisisatest!` |

### Install dependencies

To install the dependencies, run:

```bash
pnpm install
```

This will install the dependencies for all the apps and packages in the monorepo.

### Configure environment variables

In every app, there's a `.env.example` file that you can use as reference to create the relative `.env` file.

## Usage

### SwissSportPass backend canister

The backend canister is located in the [`apps/ssp_backend`](./apps/ssp_backend/) folder.

To deploy the backend canister on the local dfx replica, run the following commands:

1. Start the local replica **from the `apps/ssp_backend` folder**:

   ```bash
   pnpm dev
   ```

   Keep this terminal open.

2. In another terminal, deploy the `ssp_backend` canister **from the root of the monorepo**:

   ```bash
   ENV_FILE_PATH=apps/ssp_backend/.env && pnpm run deploy --filter=ssp_backend
   ```

### Mobile app

The mobile app is located in the [`apps/mobile`](./apps/mobile/) folder, and is built with [Expo](https://expo.dev/).

#### Run the mobile app in dev mode

To start the mobile app in dev mode, run the following commands **from the `apps/mobile` folder**:

1. Make sure you've deployed the backend canister, see the [SwissSportPass backend canister](#swisssportpass-backend-canister) section. This generates a `.env` file in the [`packages/config`](./packages/config/) folder with the necessary environment variables to connect to the backend canister from the mobile app.

2. [Prebuild](https://docs.expo.dev/workflow/prebuild/) the mobile app:

   ```bash
   pnpm expo prebuild
   ```

   > Note: you should only do this the first time you build the mobile app.

3. Start the Expo development server:

   ```bash
   pnpm dev
   ```

   More info on how to use Expo dev server on their docs: https://docs.expo.dev/more/expo-cli/#develop.

### Web app

The web app is located in the [`apps/web`](./apps/web/) folder, and is build with [Next.js](https://nextjs.org/).

#### Run the web app in dev mode

To start the web app in dev mode, run the following command **from the `apps/web` folder**:

```bash
pnpm dev
```

#### Build the web app

To build the web app, run the following command **from the `apps/web` folder**:

```bash
pnpm build
```

### Off-chain Backend

Head over to the off-chain backend's [README](./apps/backend/README.md) for more information.

## Seed data

Seed data is located in the [`apps/backend/hasura/seeds`](./apps/backend/hasura/seeds) folder. To apply the seed data, run:

```bash
pnpm hasura:seed-apply
```

## Tests

Run the tests in all packages with:

```bash
pnpm test
```

This script uses Turborepo's dependencies to run the `pretest` script in all packages before running the tests. Use the `pretest` script to prepare the environment before running the tests (e.g. building the binaries, etc.).

If you want to skip Turborepo's cache, run:

```bash
pnpm test -- --force
```

## Linting

Run the linters in all packages with:

```bash
pnpm lint
```

## Formatting

Run the formatters in all packages with:

```bash
pnpm format
```

## Turborepo and Rust

Turborepo doesn't officially support Rust yet, but we have a workaround to make it work with the cache system of Turborepo.

When you want to add a Rust **app** to the monorepo:

1. Add the Rust app in the `apps` folder
2. Add a `package.json` file with the `<app-name>` into the app folder
3. Add any script needed for the app to build, lint, etc. in the `package.json` file

When you want to add a Rust **package** to the monorepo:

1. Add the Rust package in the `packages` folder
2. Add a `package.json` file with the `<package-name>` name into the package folder
3. Add any script needed for the package to build, lint, etc. in the `package.json` file
4. Import the Rust package in the app's `Cargo.toml` file where needed
5. Import the workspace package in the package.json of the app, so that Turbo recognizes it as a dependency and handles cache properly

See [`apps/ssp_backend`](./apps/ssp_backend/) and [`packages/ssp_backend_types`](./packages/ssp_backend_types/) for reference.
