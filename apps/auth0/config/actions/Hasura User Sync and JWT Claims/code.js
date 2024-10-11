const fetch = require("node-fetch");

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

  const graphql_endpoint = event.secrets.HASURA_GRAPHQL_ENDPOINT;
  const admin_secret = event.secrets.HASURA_GRAPHQL_ADMIN_SECRET;

  // Sync your user here

  if (event.authorization) {
    const userId = "<user_id_obtained_from_db>";
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

/**
 * Handler that will be invoked when this action is resuming after an external redirect. If your
 * onExecutePostLogin function does not perform a redirect, this function can be safely ignored.
 *
 * @param {Event} event - Details about the user and the context in which they are logging in.
 * @param {PostLoginAPI} api - Interface whose methods can be used to change the behavior of the login.
 */
// exports.onContinuePostLogin = async (event, api) => {
// };
