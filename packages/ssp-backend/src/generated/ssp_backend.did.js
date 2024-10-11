export const idlFactory = ({ IDL }) => {
  const Timestamp = IDL.Nat64;
  const CreateCertificateContentRequest = IDL.Record({
    'issued_at' : Timestamp,
    'name' : IDL.Text,
    'file_uri' : IDL.Opt(IDL.Text),
    'issuer_club_name' : IDL.Opt(IDL.Text),
    'issuer_full_name' : IDL.Opt(IDL.Text),
    'notes' : IDL.Opt(IDL.Text),
    'external_id' : IDL.Opt(IDL.Text),
    'sport_category' : IDL.Text,
  });
  const CreateCertificateRequest = IDL.Record({
    'content' : CreateCertificateContentRequest,
    'managed_user_db_id' : IDL.Opt(IDL.Text),
    'user_db_id' : IDL.Text,
  });
  const CreateCertificateResponse = IDL.Record({ 'id' : IDL.Text });
  const CertificateWithId = IDL.Record({
    'id' : IDL.Text,
    'certificate_cbor_hex' : IDL.Text,
  });
  const GetCertificateResponse = IDL.Record({
    'certificate' : CertificateWithId,
    'ic_certificate' : IDL.Vec(IDL.Nat8),
    'ic_certificate_witness' : IDL.Vec(IDL.Nat8),
  });
  const Config = IDL.Record({ 'backend_principal' : IDL.Opt(IDL.Principal) });
  const Signature = IDL.Vec(IDL.Nat8);
  const PublicKey = IDL.Vec(IDL.Nat8);
  const Delegation = IDL.Record({
    'pubkey' : PublicKey,
    'targets' : IDL.Opt(IDL.Vec(IDL.Principal)),
    'expiration' : Timestamp,
  });
  const SignedDelegation = IDL.Record({
    'signature' : Signature,
    'delegation' : Delegation,
  });
  const GetDelegationResponse = IDL.Variant({
    'no_such_delegation' : IDL.Null,
    'signed_delegation' : SignedDelegation,
  });
  const Auth0JWK = IDL.Record({
    'e' : IDL.Text,
    'n' : IDL.Text,
    'alg' : IDL.Text,
    'kid' : IDL.Text,
    'kty' : IDL.Text,
    'use' : IDL.Text,
    'x5c' : IDL.Vec(IDL.Text),
    'x5t' : IDL.Text,
  });
  const Auth0JWKS = IDL.Record({ 'keys' : IDL.Vec(Auth0JWK) });
  const User = IDL.Record({
    'sub' : IDL.Text,
    'created_at' : IDL.Text,
    'db_id' : IDL.Text,
  });
  const GetUserCertificatesRequest = IDL.Record({
    'user_principal' : IDL.Opt(IDL.Principal),
    'user_db_id' : IDL.Opt(IDL.Text),
  });
  const CertificatePreviewWithId = IDL.Record({
    'id' : IDL.Text,
    'name' : IDL.Text,
  });
  const GetUserCertificatesResponse = IDL.Record({
    'certificates' : IDL.Vec(CertificatePreviewWithId),
  });
  const UserKey = PublicKey;
  const PrepareDelegationResponse = IDL.Record({
    'user_key' : UserKey,
    'expiration' : Timestamp,
  });
  return IDL.Service({
    'create_certificate' : IDL.Func(
        [CreateCertificateRequest],
        [CreateCertificateResponse],
        [],
      ),
    'get_certificate' : IDL.Func(
        [IDL.Text],
        [GetCertificateResponse],
        ['query'],
      ),
    'get_config' : IDL.Func([], [Config], ['query']),
    'get_delegation' : IDL.Func(
        [IDL.Text, Timestamp],
        [GetDelegationResponse],
        ['query'],
      ),
    'get_jwks' : IDL.Func([], [IDL.Opt(Auth0JWKS)], ['query']),
    'get_my_user' : IDL.Func([], [User], ['query']),
    'get_user_certificates' : IDL.Func(
        [GetUserCertificatesRequest],
        [GetUserCertificatesResponse],
        ['query'],
      ),
    'prepare_delegation' : IDL.Func(
        [IDL.Text],
        [PrepareDelegationResponse],
        [],
      ),
    'set_backend_principal' : IDL.Func([IDL.Principal], [], []),
    'set_jwks' : IDL.Func([Auth0JWKS], [], []),
    'sync_jwks' : IDL.Func([], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
